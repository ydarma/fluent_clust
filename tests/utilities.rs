
use std::error::Error;

use approx_eq::assert_approx_eq;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use regex::Regex;
use serde_json::{json, Value};

#[allow(unused)]
const OUT_PATTERN: &str =
    r#"^\[(\{"mu":\[[-0-9.]*\],"sigma":(null|[0-9.]*),"weight":[0-9.]*\},?)*\]$"#;

#[allow(unused)]
pub(crate) fn assert_results(result: Vec<String>) {
    let re = Regex::new(OUT_PATTERN).unwrap();
    assert!(result.iter().all(|r| re.is_match(r)));
    assert_final_result(result.last().unwrap());
}

#[allow(unused)]
pub(crate) fn assert_final_result(m: &String) {
    let final_result: Vec<Value> = serde_json::from_str(m).unwrap();
    let mu = final_result[0]["mu"].as_array().unwrap()[0]
        .as_f64()
        .unwrap();
    let sigma = final_result[0]["sigma"].as_f64().unwrap();
    let weight = final_result[0]["weight"].as_f64().unwrap();
    assert_approx_eq!(mu, 2.0, 5E-2);
    assert_approx_eq!(sigma, 9.0, 5E-2);
    assert_approx_eq!(weight, 10000., 1E-1);
}

#[allow(unused)]
pub fn get_point_iter(count: usize) -> impl Iterator<Item = Result<String, Box<dyn Error>>> {
    let normal = Normal::new(2.0, 3.0).unwrap();
    let mut rng = rand::rngs::StdRng::seed_from_u64(9787043385113690);
    (0..count).map(move |_i| {
        let v = normal.sample(&mut rng);
        Ok(json!(vec![v]).to_string())
    })
}
