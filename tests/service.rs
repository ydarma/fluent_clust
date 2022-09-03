use fluent_data::{Algo, Model, Streamer, service, space};
use std::thread;
use tungstenite::{connect, Message};
use url::Url;

use utilities::{assert_results, get_point_iter};

#[test]
fn test_streamer() {
    thread::spawn(|| start());
    thread::spawn(|| feed());
    assert_results(collect());
}

fn start() {
    let algo = Algo::new(space::euclid_dist, space::real_combine);
    let mut model = Model::new(space::euclid_dist);
    let (points, write) = service::backend();
    let streamer = Streamer::new(points, write);
    Streamer::run(streamer, algo, &mut model).unwrap();
}

fn feed() {
    let points_url = "ws://localhost:9001/ws/points";
    let (mut points_socket, _resp) =
        connect(Url::parse(points_url).unwrap()).expect("Can't connect");
    let points = get_point_iter(10000);
    for p in points {
        points_socket
            .write_message(Message::Text(p.unwrap()))
            .unwrap();
    }
    points_socket.close(None).unwrap();
}

fn collect() -> Vec<String> {
    let models_url = "ws://localhost:9001/ws/models";
    let (mut models_socket, _resp) =
        connect(Url::parse(models_url).unwrap()).expect("Can't connect");
    let mut results: Vec<String> = vec![];
    for _i in 0..10000 {
        let m = models_socket.read_message().unwrap();
        results.push(m.into_text().unwrap());
    }
    models_socket.close(None).unwrap();
    results
}
