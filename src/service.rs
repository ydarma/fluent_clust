use std::{
    error::Error,
    sync::{
        mpsc::{self, Receiver, Sender, SyncSender},
        Arc, Mutex,
    },
    thread::spawn,
};

use rouille::{
    try_or_400,
    websocket::{self, Websocket},
    Request, Response,
};
use serde::{de::DeserializeOwned, Serialize};

pub fn service<Point: PartialEq + Serialize + DeserializeOwned + 'static>() -> (
    impl Iterator<Item = Result<String, Box<dyn Error>>>,
    impl FnMut(String) -> Result<(), Box<dyn Error>>,
) {
    let (point_producer, point_receiver) = mpsc::sync_channel::<String>(1000);
    let (model_producer, model_receiver) = mpsc::channel::<String>();
    spawn(move || start_server(point_producer, model_receiver));
    channels(point_receiver, model_producer)
}

fn channels(
    point_receiver: Receiver<String>,
    model_producer: Sender<String>,
) -> (
    impl Iterator<Item = Result<String, Box<dyn Error>>>,
    impl FnMut(String) -> Result<(), Box<dyn Error>>,
) {
    let points = point_receiver.into_iter().map(|f| Ok(f));
    let write = move |model| {
        model_producer.send(model)?;
        Ok(())
    };
    (points, write)
}

fn start_server(point_producer: SyncSender<String>, model_receiver: Receiver<String>) {
    let peers: Arc<Mutex<Vec<Websocket>>> = Arc::new(Mutex::new(vec![]));
    start_dispatcher(peers.clone(), model_receiver);
    rouille::start_server("0.0.0.0:80", move |request: &Request| -> Response {
        serve(request, peers.clone(), point_producer.clone())
    })
}

fn serve(
    request: &Request,
    peers: Arc<Mutex<Vec<Websocket>>>,
    point_producer: SyncSender<String>,
) -> Response {
    if request.url().ends_with("/ws/points") {
        handle_point_receiver(&request, point_producer)
    } else if request.url().ends_with("/ws/model") {
        handle_model_producer(&request, peers)
    } else {
        Response::empty_404()
    }
}

fn handle_model_producer(request: &Request, peers: Arc<Mutex<Vec<Websocket>>>) -> Response {
    let (response, websocket) = try_or_400!(websocket::start(&request, Some("OK")));
    let ws = websocket.recv().unwrap();
    register_receiver(peers, ws);
    response
}

fn register_receiver(peers: Arc<Mutex<Vec<Websocket>>>, ws: Websocket) {
    match peers.lock() {
        Ok(mut peers) => peers.push(ws),
        Err(str) => eprintln!("{}", str),
    }
}

fn handle_point_receiver(request: &Request, point_producer: SyncSender<String>) -> Response {
    let (response, websocket) = try_or_400!(websocket::start(&request, Some("OK")));
    spawn(move || {
        let ws = websocket.recv().unwrap();
        {
            let mut websocket = ws;
            while let Some(message) = websocket.next() {
                read_point(message, &point_producer);
            }
        };
    });
    response
}

fn read_point(message: websocket::Message, point_producer: &SyncSender<String>) {
    match message {
        websocket::Message::Text(txt) => match point_producer.send(txt) {
            Err(str) => eprintln!("{:#?}", str),
            _ => {}
        },
        websocket::Message::Binary(_) => {
            eprintln!("unsupported binary message.");
        }
    }
}

fn start_dispatcher(peers: Arc<Mutex<Vec<Websocket>>>, model_receiver: Receiver<String>) {
    spawn(move || {
        for msg in model_receiver {
            match peers.lock() {
                Ok(mut peers) => {
                    peers.retain_mut(|peer| send_model(peer, &msg));
                }
                Err(_) => {}
            }
        }
    });
}

fn send_model(peer: &mut Websocket, msg: &String) -> bool {
    if !peer.is_closed() {
        match peer.send_text(&msg) {
            Err(str) => eprintln!("{:#?}", str),
            _ => {}
        };
        true
    } else {
        false
    }
}
