use std::{
    error::Error,
    sync::{
        mpsc::{self, Receiver, Sender},
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

use crate::streamer;

pub fn service<Point: PartialEq + Serialize + DeserializeOwned + 'static>() -> (
    impl Iterator<Item = Result<String, Box<dyn Error>>>,
    impl FnMut(String) -> Result<(), Box<dyn Error>>,
) {
    let (point_producer, point_receiver) = mpsc::channel::<String>();
    let (model_producer, model_receiver) = mpsc::channel::<String>();
    spawn(move || start_server(point_producer, model_receiver));
    streamer::channels(point_receiver, model_producer)
}

fn start_server(point_producer: Sender<String>, model_receiver: Receiver<String>) {
    let peers: Arc<Mutex<Vec<Websocket>>> = Arc::new(Mutex::new(vec![]));
    start_dispatcher(peers.clone(), model_receiver);
    start_http(peers.clone(), point_producer);
}

fn start_http(peers: Arc<Mutex<Vec<Websocket>>>, point_producer: Sender<String>) {
    let point_producer = Arc::new(Mutex::new(point_producer));
    rouille::start_server("0.0.0.0:8080", move |request: &Request| -> Response {
        if request.url().ends_with("/ws/points") {
            handle_point_receiver(&request, point_producer.lock().unwrap().clone())
        } else if request.url().ends_with("/ws/model") {
            handle_model_producer(&request, peers.clone())
        } else {
            Response::empty_404()
        }
    });
}

fn handle_model_producer(request: &Request, peers: Arc<Mutex<Vec<Websocket>>>) -> Response {
    let (response, websocket) = try_or_400!(websocket::start(&request, Some("OK")));
    match websocket.recv() {
        Ok(websocket) => register_receiver(peers, websocket),
        Err(reason) => eprintln!("{}", reason),
    }
    response
}

fn register_receiver(peers: Arc<Mutex<Vec<Websocket>>>, websocket: Websocket) {
    let mut peers = peers.lock().unwrap();
    peers.push(websocket);
}

fn handle_point_receiver(request: &Request, point_producer: Sender<String>) -> Response {
    let (response, websocket) = try_or_400!(websocket::start(&request, Some("OK")));
    spawn(move || {
        match websocket.recv() {
            Ok(mut websocket) => {
                while let Some(message) = websocket.next() {
                    read_point(message, &point_producer);
                }
            }
            Err(reason) => eprint!("{}", reason),
        };
    });
    response
}

fn read_point(message: websocket::Message, point_producer: &Sender<String>) {
    match message {
        websocket::Message::Text(txt) => match point_producer.send(txt) {
            Err(reason) => eprintln!("{:#?}", reason),
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
            let mut peers = peers.lock().unwrap();
            peers.retain_mut(|peer| send_model(peer, &msg));
        }
    });
}

fn send_model(peer: &mut Websocket, msg: &String) -> bool {
    if !peer.is_closed() {
        match peer.send_text(&msg) {
            Err(reason) => eprintln!("{:#?}", reason),
            _ => {}
        };
        true
    } else {
        false
    }
}
