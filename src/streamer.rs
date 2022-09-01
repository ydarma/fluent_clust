use std::{error::Error, ops::Deref};

use crate::{algorithm::Algo, model::{Model, NormalData}};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Map, Value};

pub struct Streamer<In, Out>
where
    In: Iterator<Item = Result<String, std::io::Error>>,
    Out: Fn(String),
{
    points: In,
    write: Out,
}

impl<In, Out> Streamer<In, Out>
where
    In: Iterator<Item = Result<String, std::io::Error>>,
    Out: Fn(String),
{
    pub fn new(points: In, write: Out) -> Self {
        Self { points, write }
    }

    pub fn run<Point: PartialEq + Serialize + DeserializeOwned + 'static>(
        streamer: Streamer<In, Out>,
        algo: Algo<Point>,
        model: &mut Model<Point>,
    ) -> Result<(), Box<dyn Error>> {
        for input in streamer.points {
            let point_str = input?;
            let point: Point = serde_json::from_str(&point_str)?;
            algo.fit(model, point);
            let components = serialize_model(model);
            let output = serde_json::to_string(&components)?;
            (streamer.write)(output);
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
    data: impl Deref<Target = NormalData<Point>>,
) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("mu".into(), json!(data.mu()));
    map.insert("sigma".into(), json!(data.sigma()));
    map.insert("weight".into(), json!(data.weight()));
    map
}
