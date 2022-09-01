use std::{error::Error};

use fluent_data::{algorithm::Algo, model::Model, space, streamer::{Streamer, self}};

fn main() -> Result<(), Box<dyn Error>> {
    let (algo, mut model) = get_algo_model();
    let (points, write) = streamer::stdio();
    let streamer = Streamer::new(points, write);
    Streamer::run(streamer, algo, &mut model)?;
    Ok(())
}

fn get_algo_model() -> (Algo<Vec<f64>>, Model<Vec<f64>>) {
    let algo = Algo::new(space::euclid_dist, space::real_combine);
    let model = Model::new(space::euclid_dist);
    (algo, model)
}
