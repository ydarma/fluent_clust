use std::thread;

use approx_eq::assert_approx_eq;
use fluent_data::{algorithm::Algo, model::Model, service::service, space, streamer::*};
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use regex::Regex;
use serde_json::{json, Value};
use tungstenite::{connect, Message};
use url::Url;

const OUT_PATTERN: &str =
    r#"^\[(\{"mu":\[[-0-9.]*\],"sigma":(null|[0-9.]*),"weight":[0-9.]*\},?)*\]$"#;

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
        let normal = Normal::new(2.0, 3.0).unwrap();
        let mut rng = rand::rngs::StdRng::seed_from_u64(9787043385113690);
        for _i in 1..10000 {
            let p = vec![normal.sample(&mut rng)];
            points_socket
                .write_message(Message::Text(json!(p).to_string()))
                .unwrap();
        }
        points_socket.close(None).unwrap();
    });
    let models_url = "ws://localhost:9001/ws/models";
    let (mut models_socket, _resp) =
        connect(Url::parse(models_url).unwrap()).expect("Can't connect");
    let re = Regex::new(OUT_PATTERN).unwrap();
    for _i in 1..9999 {
        let m = models_socket.read_message().unwrap();
        assert!(re.is_match(&m.into_text().unwrap()));
    }
    let m = models_socket.read_message().unwrap();
    let result: Vec<Value> = serde_json::from_str(&m.into_text().unwrap()).unwrap();
    let mu = result[0]["mu"].as_array().unwrap()[0].as_f64().unwrap();
    let sigma = result[0]["sigma"].as_f64().unwrap();
    let weight = result[0]["weight"].as_f64().unwrap();
    assert_approx_eq!(mu, 2.0, 5E-2);
    assert_approx_eq!(sigma, 9.0, 5E-2);
    assert_approx_eq!(weight, 10000., 1E-1);
    models_socket.close(None).unwrap();
}
