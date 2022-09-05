//! The [Streamer] continuously consumes data points and produces models.
//!
//! This module also provides the [stdio] function that builds
//! a point iterator which reads the standard input and a
//! write closure that writes to the standard output.

use std::{
    error::Error,
    io,
    ops::Deref,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    algorithm::Algo,
    model::{Ball, Model},
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Map, Value};

/// Reads data from `In` and writes model to `Out`.
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
            let balls = serialize_model(model);
            let output = serde_json::to_string(&balls)?;
            (streamer.write)(output)?;
        }
        Ok(())
    }
}

fn serialize_model<Point: PartialEq + Serialize + 'static>(
    model: &Model<Point>,
) -> Vec<Map<String, Value>> {
    let balls: Vec<_> = model
        .iter_balls()
        .map(|data| serialize_ball(data))
        .collect();
    balls
}

fn serialize_ball<Point: PartialEq + Serialize>(
    data: impl Deref<Target = Ball<Point>>,
) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("center".into(), json!(data.center()));
    map.insert("radius".into(), json!(data.radius()));
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

    use std::sync::mpsc;

    use crate::{space, streamer::*};

    #[test]
    fn test_serialize_ball() {
        let obj = serialize_ball(&Ball::new(vec![3., 5.1], 4.7, 0.999));
        let json = serde_json::to_string(&obj).unwrap();
        assert_eq!(
            r#"{"center":[3.0,5.1],"radius":2.16794833886788,"weight":0.999}"#,
            json
        );
    }

    #[test]
    fn test_serialize_model() {
        let mut model = Model::new(space::euclid_dist);
        let v = model.add_ball(Ball::new(vec![3., 5.1], 4.7, 0.999), vec![]);
        model.add_ball(Ball::new(vec![1.2, 6.], 1.3, 3.998), vec![v.as_neighbor()]);
        let obj = serialize_model(&model);
        let json = serde_json::to_string(&obj).unwrap();
        assert_eq!(
            r#"[{"center":[3.0,5.1],"radius":2.16794833886788,"weight":0.999},{"center":[1.2,6.0],"radius":1.140175425099138,"weight":3.998}]"#,
            json
        );
    }

    #[test]
    fn test_streamer() {
        let algo = Algo::new(space::euclid_dist, space::real_combine);
        let mut model = Model::new(space::euclid_dist);
        let points = vec![Ok(String::from("[1.0,1.0]"))].into_iter();
        let mut result = String::new();
        let write = |s| {
            result = s;
            Ok(())
        };
        let streamer = Streamer::new(points, write);
        match Streamer::run(streamer, algo, &mut model) {
            Ok(()) => assert_eq!(
                r#"[{"center":[1.0,1.0],"radius":null,"weight":0.0}]"#,
                result
            ),
            Err(_) => panic!(),
        };
    }

    #[test]
    fn test_channels() {
        let (point_producer, point_receiver) = mpsc::channel();
        let (model_producer, model_receiver) = mpsc::channel();
        let (mut points, mut write) = channels(point_receiver, model_producer);
        point_producer.send(String::from("point")).unwrap();
        let p = points.next().unwrap().unwrap();
        assert_eq!("point", p);
        (write)(String::from("model")).unwrap();
        let m = model_receiver.recv().unwrap();
        assert_eq!("model", m);
    }
}
