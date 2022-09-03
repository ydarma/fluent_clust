use std::error::Error;

use approx_eq::assert_approx_eq;
use fluent_data::{algorithm::Algo, model::Model, space, streamer::*};
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use regex::Regex;
use serde_json::{json, Value};

const OUT_PATTERN: &str =
    r#"^\[(\{"mu":\[[-0-9.]*\],"sigma":(null|[0-9.]*),"weight":[0-9.]*\},?)*\]$"#;

#[test]
fn test_streamer() {
    let algo = Algo::new(space::euclid_dist, space::real_combine);
    let mut model = Model::new(space::euclid_dist);
    let points = get_point_iter();
    let mut result: Vec<String> = vec![];
    let write = |model: String| Ok(result.push(model));
    let streamer = Streamer::new(points, write);
    match Streamer::run(streamer, algo, &mut model) {
        Ok(()) => {
            let re = Regex::new(OUT_PATTERN).unwrap();
            assert!(result.iter().all(|r| re.is_match(r)));
            let m = &result[9998];
            let result: Vec<Value> = serde_json::from_str(m).unwrap();
            let mu = result[0]["mu"].as_array().unwrap()[0].as_f64().unwrap();
            let sigma = result[0]["sigma"].as_f64().unwrap();
            let weight = result[0]["weight"].as_f64().unwrap();
            assert_approx_eq!(mu, 2.0, 5E-2);
            assert_approx_eq!(sigma, 9.0, 5E-2);
            assert_approx_eq!(weight, 10000., 1E-1);
        }
        Err(_) => panic!(),
    };
}

fn get_point_iter() -> impl Iterator<Item = Result<String, Box<dyn Error>>> {
    let normal = Normal::new(2.0, 3.0).unwrap();
    let mut rng = rand::rngs::StdRng::seed_from_u64(9787043385113690);
    (1..10000).map(move |_i| {
        let v = normal.sample(&mut rng);
        Ok(json!(vec![v]).to_string())
    })
}
