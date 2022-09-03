use std::thread;
use fluent_data::{algorithm::Algo, model::Model, service::service, space, streamer::*};
use tungstenite::{connect, Message};
use url::Url;

#[path = "./utilities.rs"]
mod utilities;
use utilities::{get_point_iter, assert_results};

#[test]
fn test_streamer() {
    thread::spawn(|| {
        let algo = Algo::new(space::euclid_dist, space::real_combine);
        let mut model = Model::new(space::euclid_dist);
        let (points, write) = service::<Vec<f64>>();
        let streamer = Streamer::new(points, write);
        Streamer::run(streamer, algo, &mut model).unwrap();
    });
    thread::spawn(|| {
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
    });
    let models_url = "ws://localhost:9001/ws/models";
    let (mut models_socket, _resp) =
        connect(Url::parse(models_url).unwrap()).expect("Can't connect");
    let mut result: Vec<String> = vec![];
    for _i in 0..10000 {
        let m = models_socket.read_message().unwrap();
        result.push(m.into_text().unwrap());
    }
    assert_results(result);
    models_socket.close(None).unwrap();
}
