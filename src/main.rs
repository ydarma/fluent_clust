use std::{error::Error, io};

use fluent_data::{algorithm::Algo, model::Model, space, streamer::Streamer};

fn main() -> Result<(), Box<dyn Error>> {
    let algo = Algo::new(space::euclid_dist, space::real_combine);
    let mut model = Model::new(space::euclid_dist);
    let points = io::stdin().lines();
    let write = |model: String| println!("{}", model);
    let streamer = Streamer::new(points, write);
    Streamer::run(streamer, algo, &mut model)?;
    Ok(())
}
