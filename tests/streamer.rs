use fluent_data::{algorithm::Algo, model::Model, space, streamer::*};
use rand;
use rand_distr::{Distribution, Normal};
use regex::Regex;
use serde_json::json;

const OUT_PATTERN: &str =
    r#"^\[(\{"mu":\[[-0-9.]*\],"sigma":(null|[0-9.]*),"weight":[0-9.]*\},?)*\]$"#;

#[test]
fn test_streamer() {
    let algo = Algo::new(space::euclid_dist, space::real_combine);
    let mut model = Model::new(space::euclid_dist);
    let normal = Normal::new(2.0, 3.0).unwrap();
    let points = (1..10000).map(|_i| -> Result<_, _> {
        let v = normal.sample(&mut rand::thread_rng());
        Ok(json!(vec![v]).to_string())
    });
    let mut result: Vec<String> = vec![];
    let write = |model: String| Ok(result.push(model));
    let streamer = Streamer::new(points, write);
    match Streamer::run(streamer, algo, &mut model) {
        Ok(()) => {
            let re = Regex::new(OUT_PATTERN).unwrap();
            assert!(result.iter().all(|r| re.is_match(r)));
        }
        Err(_) => panic!(),
    };
}
