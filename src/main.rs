use std::error::Error;

use fluent_data::{space, streamer};
use fluent_data::{Algo, Model, Streamer};

fn main() -> Result<(), Box<dyn Error>> {
    let (algo, mut model) = get_algo_model();
    let streamer = get_streamer();
    Streamer::run(streamer, algo, &mut model)?;
    Ok(())
}

fn get_streamer() -> Streamer<
    impl Iterator<Item = Result<String, Box<dyn Error>>>,
    impl FnMut(String) -> Result<(), Box<dyn Error>>,
> {
    let (points, write) = streamer::stdio();
    let streamer = Streamer::new(points, write);
    streamer
}

fn get_algo_model() -> (Algo<Vec<f64>>, Model<Vec<f64>>) {
    let algo = Algo::new(space::euclid_dist, space::real_combine);
    let model = Model::new(space::euclid_dist);
    (algo, model)
}
