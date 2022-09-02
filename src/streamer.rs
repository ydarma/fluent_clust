use std::{error::Error, io, ops::Deref};

use crate::{
    algorithm::Algo,
    model::{Model, GaussianData},
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Map, Value};

/// Reads data form `In` source and write model to `Out` sink.
/// ```
/// use std::{error::Error, io};
///
/// use fluent_data::{algorithm::Algo, model::Model, space, streamer::{Streamer, self}};
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let algo = Algo::new(space::euclid_dist, space::real_combine);
///     let mut model = Model::new(space::euclid_dist);
///     let (points, write) = streamer::stdio();
///     let streamer = Streamer::new(points, write);
///     Streamer::run(streamer, algo, &mut model)?;
///     Ok(())
/// }
/// ```
pub struct Streamer<In, Out>
where
    In: Iterator<Item = Result<String, Box<dyn Error>>>,
    Out: FnMut(String)-> Result<(), Box<dyn Error>>,
{
    points: In,
    write: Out,
}

impl<In, Out> Streamer<In, Out>
where
    In: Iterator<Item = Result<String, Box<dyn Error>>>,
    Out: FnMut(String)-> Result<(), Box<dyn Error>>,
{
    /// builds a new streamer instance.
    pub fn new(points: In, write: Out) -> Self {
        Self { points, write }
    }

    /// Infinitely reads points from `In` source and write model changes to `Out` sink.
    pub fn run<Point: PartialEq + Serialize + DeserializeOwned + 'static>(
        mut streamer: Streamer<In, Out>,
        algo: Algo<Point>,
        model: &mut Model<Point>,
    ) -> Result<(), Box<dyn Error>> {
        for input in streamer.points {
            let point_str = input?;
            let point: Point = serde_json::from_str(&point_str)?;
            algo.fit(model, point);
            let components = serialize_model(model);
            let output = serde_json::to_string(&components)?;
            (streamer.write)(output)?;
        }
        Ok(())
    }
}

fn serialize_model<Point: PartialEq + Serialize + 'static>(
    model: &Model<Point>,
) -> Vec<Map<String, Value>> {
    let components: Vec<_> = model
        .iter_components()
        .map(|data| serialize_component(data))
        .collect();
    components
}

fn serialize_component<Point: PartialEq + Serialize>(
    data: impl Deref<Target = GaussianData<Point>>,
) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("mu".into(), json!(data.mu()));
    map.insert("sigma".into(), json!(data.sigma()));
    map.insert("weight".into(), json!(data.weight()));
    map
}

pub fn stdio() -> (
    impl Iterator<Item = Result<String, Box<dyn Error>>>,
    impl FnMut(String) -> Result<(), Box<dyn Error>>,
) {
    let points = io::stdin()
        .lines()
        .map(|f| -> Result<String, Box<dyn Error>> { Ok(f?) });
    let write = |model| {
        println!("{}", model);
        Ok(())
    };
    (points, write)
}

#[cfg(test)]
mod tests {
    use crate::algorithm::tests::build_sample;
    use crate::{space, streamer::*};
    use regex::Regex;

    const OUT_PATTERN: &str =
        r#"^\[(\{"mu":\[[-0-9.]*,[-0-9.]*\],"sigma":(null|[0-9.]*),"weight":[0-9.]*\},?)*\]$"#;

    #[test]
    fn test_serialize_component() {
        let obj = serialize_component(&GaussianData::new(vec![3., 5.1], 4.7, 0.999));
        let json = serde_json::to_string(&obj).unwrap();
        assert_eq!(r#"{"mu":[3.0,5.1],"sigma":4.7,"weight":0.999}"#, json);
    }

    #[test]
    fn test_streamer() {
        let algo = Algo::new(space::euclid_dist, space::real_combine);
        let mut model = Model::new(space::euclid_dist);
        let dataset = build_sample();
        let points = dataset.iter().map(|v| Ok(json!(v).to_string()));
        let mut result: Vec<String> = vec![];
        let write = |model: String| Ok(result.push(model));
        let streamer = Streamer::new(points, write);
        match Streamer::run(streamer, algo, &mut model) {
            Ok(()) => {
                let re = Regex::new(OUT_PATTERN).unwrap();
                assert!(result.iter().all(|r| re.is_match(r)));
            }
            Err(_) => {
                assert!(false)
            }
        };
    }
}
