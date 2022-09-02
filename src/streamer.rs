use std::{
    error::Error,
    io,
    ops::Deref,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    algorithm::Algo,
    model::{GaussianData, Model},
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
    Out: FnMut(String) -> Result<(), Box<dyn Error>>,
{
    points: In,
    write: Out,
}

impl<In, Out> Streamer<In, Out>
where
    In: Iterator<Item = Result<String, Box<dyn Error>>>,
    Out: FnMut(String) -> Result<(), Box<dyn Error>>,
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

/// Returns point iterator / model writer that use standard in out.
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

/// Returns point iterator / model writer that use mpsc channels.
pub fn channels(
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

#[cfg(test)]
mod tests {

    use crate::streamer::*;

    #[test]
    fn test_serialize_component() {
        let obj = serialize_component(&GaussianData::new(vec![3., 5.1], 4.7, 0.999));
        let json = serde_json::to_string(&obj).unwrap();
        assert_eq!(r#"{"mu":[3.0,5.1],"sigma":4.7,"weight":0.999}"#, json);
    }
}
