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
    let (point_sender, point_receiver) = mpsc::sync_channel::<String>(1000);
    let (model_sender, model_receiver) = mpsc::channel::<String>();
    spawn(move || start_server(point_sender, model_receiver));
    channels(point_receiver, model_sender)
}

fn channels(
    point_receiver: Receiver<String>,
    model_sender: Sender<String>,
) -> (
    impl Iterator<Item = Result<String, Box<dyn Error>>>,
    impl FnMut(String) -> Result<(), Box<dyn Error>>,
) {
    let points = point_receiver.into_iter().map(|f| Ok(f));
    let write = move |model| {
        model_sender.send(model)?;
        Ok(())
    };
    (points, write)
}

fn start_server(point_sender: SyncSender<String>, model_receiver: Receiver<String>) {
    let peers: Arc<Mutex<Vec<Websocket>>> = Arc::new(Mutex::new(vec![]));
    start_dispatcher(peers.clone(), model_receiver);
    rouille::start_server("0.0.0.0:80", move |request: &Request| -> Response {
        serve(request, peers.clone(), point_sender.clone())
    })
}

fn serve(
    request: &Request,
    peers: Arc<Mutex<Vec<Websocket>>>,
    point_sender: SyncSender<String>,
) -> Response {
    if request.url().ends_with("/ws/points") {
        handle_point_receiver(&request, point_sender)
    } else if request.url().ends_with("/ws/model") {
        handle_model_sender(&request, peers)
    } else {
        Response::empty_404()
    }
}

fn handle_model_sender(request: &Request, peers: Arc<Mutex<Vec<Websocket>>>) -> Response {
    let (response, websocket) = try_or_400!(websocket::start(&request, Some("OK")));
    let ws = websocket.recv().unwrap();
    match peers.lock() {
        Ok(mut peers) => {
            peers.push(ws);
        }
        Err(_) => {}
    }
    response
}

fn handle_point_receiver(request: &Request, point_sender: SyncSender<String>) -> Response {
    let (response, websocket) = try_or_400!(websocket::start(&request, Some("OK")));
    spawn(move || {
        let ws = websocket.recv().unwrap();
        {
            let mut websocket = ws;
            while let Some(message) = websocket.next() {
                match message {
                    websocket::Message::Text(txt) => {
                        match point_sender.send(txt) {
                            Err(str) => eprintln!("{:#?}", str),
                            _ => {}
                        }
                    }
                    websocket::Message::Binary(_) => {
                        eprintln!("unsupported binary message.");
                    }
                }
            }
        };
    });
    response
}

fn start_dispatcher(peers: Arc<Mutex<Vec<Websocket>>>, model_receiver: Receiver<String>) {
    spawn(move || {
        for msg in model_receiver {
            match peers.lock() {
                Ok(mut peers) => {
                    for peer in peers.iter_mut() {
                        match peer.send_text(&msg) {
                            Err(str) => eprintln!("{:#?}", str),
                            _ => {}
                        }
                    }
                }
                Err(_) => {}
            }
        }
    });
}
