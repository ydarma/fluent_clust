//! This library provides an online algorithm to fit streaming data into a mixed Gaussian model.
//! Components covariances are supposed to be zero, i.e. for a given component dimensions are independant from each other.
//! Theese are very strong hypothesis, thus the algorithm is not suited to all kind of data.
//!
//! The algorithm uses two functions that can be custom :
//!  - a function that computes a distance between points
//!  - a function that computes the wighted center of two points
//!  
//! Theese functions are used to construct the [Algo] and [Model] structs,
//! that represents respectively the algorithm and the mixed Gaussian model.
//!
//! ```
//! use fluent_data::{Model, Algo, space};
//!
//! fn build_algorithm() -> (Model<Vec<f64>>, Algo<Vec<f64>>) {
//!     let algo = Algo::new(space::euclid_dist, space::real_combine);
//!     let model = Model::new(space::euclid_dist);
//!     (model, algo)
//! }
//! ```
//!
//! The [Streamer] enlessly consumes data points and produce models. the streamer needs:
//!  - a point iterator that produces points consumed by the streamer.
//!  - a write closure that consumes models produced by the streamer.
//! The [streamer::stdio] functions builds an iterator that reads standard input
//! and a write closure that writes to standard input.
//! ```
//! use std::error::Error;
//! use fluent_data::{streamer, Streamer};
//!
//! fn build_streamer() -> Streamer<
//!     impl Iterator<Item = Result<String, Box<dyn Error>>>,
//!     impl FnMut(String) -> Result<(), Box<dyn Error>>,
//! >
//! {
//!     let (points, write) = streamer::stdio();
//!     Streamer::new(points, write)
//! }
//! ```
//!
//! Then the [Streamer::run] will run the algorithm and fit the model continuously,
//! consuming data points from standard input and producing models to standard output.
//! ```
//! use std::{error::Error};
//!
//! use fluent_data::{Algo, Model, Streamer};
//! use fluent_data::{ space, streamer};
//!
//! fn main() {
//!     let (algo, mut model) = get_algo_model();
//!     let streamer = get_streamer();
//!     Streamer::run(streamer, algo, &mut model).unwrap();
//! }
//!
//! fn get_algo_model() -> (Algo<Vec<f64>>, Model<Vec<f64>>) {
//!     let algo = Algo::new(space::euclid_dist, space::real_combine);
//!     let model = Model::new(space::euclid_dist);
//!     (algo, model)
//! }
//!
//! fn get_streamer() -> Streamer<
//!     impl Iterator<Item = Result<String, Box<dyn Error>>>,
//!     impl FnMut(String) -> Result<(), Box<dyn Error>>
//! > {
//!     let (points, write) = streamer::stdio();
//!     let streamer = Streamer::new(points, write);
//!     streamer
//! }
//! ```
//! 
//! Alternatively, the library provides a backend that
//! receive data points from websockets and send models to websockets.
//! To achieve this, just replace the point iterator and
//! the model write closure when building the streamer: use those provided by
//! the [service::backend] method.
//! ```
//! use std::error::Error;
//! use fluent_data::{service, Streamer};
//!
//! fn build_streamer() -> Streamer<
//!     impl Iterator<Item = Result<String, Box<dyn Error>>>,
//!     impl FnMut(String) -> Result<(), Box<dyn Error>>,
//! >
//! {
//!     let (points, write) = service::backend();
//!     Streamer::new(points, write)
//! }
//! ```
//! 
//! An executable program is also provided by this crate:
//!  - `fluent_data`
//!    - reads point from standard input and writes models to standard output,
//!  - `fluent_data --service`
//!    - starts a server, receives point from websockets and dispatch models to websockets,
//!  - `fluent_data --help`
//!    - display the executable usage documentation.

pub mod algorithm;
pub mod model;
pub mod service;
pub mod space;
pub mod streamer;

mod graph;
mod neighborhood;

pub use algorithm::Algo;
pub use model::Model;
pub use streamer::Streamer;
