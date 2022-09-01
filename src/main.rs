use std::{
    error::Error,
    io,
    ops::Deref,
};

use fluent_data::{
    algorithm::Algo,
    model::{Model, NormalData},
    space,
};

use serde_json::{json, Map, Value};

fn main() -> Result<(), Box<dyn Error>> {
    let algo = Algo::new(space::euclid_dist, space::real_combine);
    let mut model = Model::new(space::euclid_dist);
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let point: Vec<f64> = serde_json::from_str(&input)?;
        algo.fit(&mut model, point);
        let components = serialize_model(&model);
        let output = serde_json::to_string(&components)?;
        println!("{}", output);
    }
}

fn serialize_model(model: &Model<Vec<f64>>) -> Vec<Map<String, Value>> {
    let components: Vec<_> = model
        .iter_components()
        .map(|data| serialize_component(data))
        .collect();
    components
}

fn serialize_component(data: impl Deref<Target = NormalData<Vec<f64>>>) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("mu".into(), json!(data.mu()));
    map.insert("sigma".into(), json!(data.sigma()));
    map.insert("weight".into(), json!(data.weight()));
    map
}
