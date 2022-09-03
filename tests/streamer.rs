use fluent_data::{algorithm::Algo, model::Model, space, streamer::*};

#[path = "./utilities.rs"]
mod utilities;
use utilities::{assert_results, get_point_iter};

#[test]
fn test_streamer() {
    let algo = Algo::new(space::euclid_dist, space::real_combine);
    let mut model = Model::new(space::euclid_dist);
    let points = get_point_iter(10000);
    let mut result: Vec<String> = vec![];
    let write = |model: String| Ok(result.push(model));
    let streamer = Streamer::new(points, write);
    match Streamer::run(streamer, algo, &mut model) {
        Ok(()) => assert_results(result),
        Err(_) => panic!(),
    };
}
