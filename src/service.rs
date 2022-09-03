use std::{
    error::Error,
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use serde::{de::DeserializeOwned, Serialize};
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
    Message, WebSocket,
};

use crate::streamer;

type Peers = Arc<Mutex<Vec<WebSocket<TcpStream>>>>;

pub fn service<Point: PartialEq + Serialize + DeserializeOwned + 'static>() -> (
    impl Iterator<Item = Result<String, Box<dyn Error>>>,
    impl FnMut(String) -> Result<(), Box<dyn Error>>,
) {
    let (point_producer, point_receiver) = mpsc::channel::<String>();
    let (model_producer, model_receiver) = mpsc::channel::<String>();
    thread::spawn(move || start_server(point_producer, model_receiver));
    streamer::channels(point_receiver, model_producer)
}

fn start_server(point_producer: Sender<String>, model_receiver: Receiver<String>) {
    let peers: Peers = Arc::new(Mutex::new(vec![]));
    start_dispatcher(peers.clone(), model_receiver);
    start_websockets(peers.clone(), point_producer);
}

fn start_websockets(peers: Peers, point_producer: Sender<String>) {
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();
    for stream in server.incoming() {
        let (path, websocket) = get_websocket(stream);
        if path.ends_with("/ws/points") {
            handle_point_receiver(websocket, point_producer.clone());
        } else if path.ends_with("/ws/models") {
            handle_model_producer(websocket, peers.clone());
        }
    }
}

fn get_websocket(stream: Result<TcpStream, std::io::Error>) -> (String, WebSocket<TcpStream>) {
    let mut path: String = String::new();
    let callback = |req: &Request, response: Response| {
        path = String::from(req.uri().path());
        Ok(response)
    };
    let websocket = accept_hdr(stream.unwrap(), callback).unwrap();
    (path, websocket)
}

fn handle_model_producer(websocket: WebSocket<TcpStream>, peers: Peers) {
    let mut peers = peers.lock().unwrap();
    peers.push(websocket);
}

fn handle_point_receiver(mut websocket: WebSocket<TcpStream>, point_producer: Sender<String>) {
    thread::spawn(move || loop {
        let msg = websocket.read_message();
        match msg {
            Ok(message) => {
                if !read_point(message, &point_producer) {
                    break;
                }
            }
            Err(reason) => {
                eprint!("{}", reason);
                break;
            }
        };
    });
}

fn read_point(message: Message, point_producer: &Sender<String>) -> bool {
    match message {
        Message::Text(txt) => {
            match point_producer.send(txt) {
                Err(reason) => eprintln!("{:#?}", reason),
                _ => {}
            }
            true
        }
        Message::Binary(_) => {
            eprintln!("unsupported binary message.");
            true
        }
        Message::Close(_) => false,
        _ => true,
    }
}

fn start_dispatcher(peers: Peers, model_receiver: Receiver<String>) {
    thread::spawn(move || {
        for msg in model_receiver {
            let mut peers = peers.lock().unwrap();
            peers.retain_mut(|peer| send_model(peer, msg.clone()));
        }
    });
}

fn send_model(peer: &mut WebSocket<TcpStream>, msg: String) -> bool {
    if peer.can_write() {
        match peer.write_message(Message::Text(msg)) {
            Err(reason) => eprintln!("{:#?}", reason),
            _ => {}
        };
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use crate::{algorithm::Algo, model::Model, service::service, space, streamer::*};
    use tungstenite::{connect, Message};
    use url::Url;

    #[test]
    fn test_streamer() {
        thread::spawn(move || {
            let algo = Algo::new(space::euclid_dist, space::real_combine);
            let mut model = Model::new(space::euclid_dist);
            let (points, write) = service::<Vec<f64>>();
            let streamer = Streamer::new(points, write);
            Streamer::run(streamer, algo, &mut model).unwrap();
        });
        let points_url = "ws://localhost:9001/ws/points";
        let (mut points_socket, _resp) =
            connect(Url::parse(points_url).unwrap()).expect("Can't connect");
        let models_url = "ws://localhost:9001/ws/models";
        let (mut models_socket, _resp) =
            connect(Url::parse(models_url).unwrap()).expect("Can't connect");
        points_socket
            .write_message(Message::Text("[1.0,1.0]".into()))
            .unwrap();
        let result = models_socket.read_message().unwrap();
        assert_eq!(
            r#"[{"mu":[1.0,1.0],"sigma":null,"weight":0.0}]"#,
            result.into_text().unwrap()
        );
        models_socket.close(None).unwrap();
        points_socket.close(None).unwrap();
    }
}
