use std::error::Error;

use clap::Parser;
use fluent_data::{service, space, streamer};
use fluent_data::{Algo, Model, Streamer};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// starts in service mode.
    #[clap(short, long, value_parser)]
    service: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let (algo, mut model) = get_algo_model();
    let streamer = get_streamer(&args);
    Streamer::run(streamer, algo, &mut model)?;
    Ok(())
}

type BoxedInOut = (
    Box<dyn Iterator<Item = Result<String, Box<dyn Error>>>>,
    Box<dyn FnMut(String) -> Result<(), Box<dyn Error>>>,
);

fn get_streamer(
    args: &Args,
) -> Streamer<
    Box<dyn Iterator<Item = Result<String, Box<dyn Error>>>>,
    Box<dyn FnMut(String) -> Result<(), Box<dyn Error>>>,
> {
    let (points, write): BoxedInOut = if args.service {
        let (points, write) = service::backend();
        (Box::new(points), Box::new(write))
    } else {
        let (points, write) = streamer::stdio();
        (Box::new(points), Box::new(write))
    };
    let streamer = Streamer::new(points, write);
    streamer
}

fn get_algo_model() -> (Algo<Vec<f64>>, Model<Vec<f64>>) {
    let algo = Algo::new(space::euclid_dist, space::real_combine);
    let model = Model::new(space::euclid_dist);
    (algo, model)
}
