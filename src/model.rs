//! The [Model] struct represents the set of balls model.
//!
//! The model can be loaded with existing balls by the [Model::load] method.
//! It can also be used to predict the balls that most probably contains a given point
//! by using the [Model::predict] method.
use std::ops::Deref;

use crate::{
    graph::{Neighbor, Vertex},
    neighborhood::{GetNeighborhood, Neighborhood},
};

/// A ball in the set of balls model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ball<Point: PartialEq> {
    pub(crate) center: Point,
    pub(crate) radius: f64,
    pub(crate) weight: f64,
}

impl<Point: PartialEq> Ball<Point> {
    /// Builds a new ball.
    pub fn new(center: Point, radius: f64, weight: f64) -> Self {
        Ball {
            center,
            radius,
            weight,
        }
    }

    /// Ball center.
    pub fn center(&self) -> &Point {
        &self.center
    }

    /// Ball radius.
    pub fn radius(&self) -> f64 {
        self.radius.sqrt()
    }

    /// Ball weight.
    pub fn weight(&self) -> f64 {
        self.weight
    }
}

/// A graph node which represents a ball.
pub(crate) type BallNode<Point> = Vertex<Ball<Point>>;

/// A set of balls model.
pub struct Model<Point: PartialEq> {
    pub(crate) dist: Box<dyn Fn(&Point, &Ball<Point>) -> f64>,
    pub(crate) graph: Vec<BallNode<Point>>,
}

impl<Point: PartialEq + 'static> Model<Point> {
    /// Build a new model.
    pub fn new<Dist>(space_dist: Dist) -> Self
    where
        Dist: Fn(&Point, &Point) -> f64 + 'static,
    {
        Self {
            dist: Box::new(Model::normalize(space_dist)),
            graph: vec![],
        }
    }

    /// Load an existing model.
    /// ```
    /// use fluent_data::{Model, model::Ball, space};
    ///
    /// fn main() {
    ///     let data = vec![
    ///         Ball::new(vec![4.], 3., 1.),
    ///         Ball::new(vec![5.], 2., 2.),
    ///         Ball::new(vec![3.], 3., 3.),
    ///     ];
    ///     let model = Model::load(space::euclid_dist, data);
    /// }
    /// ```
    pub fn load<Dist>(space_dist: Dist, data: Vec<Ball<Point>>) -> Self
    where
        Dist: Fn(&Point, &Point) -> f64 + 'static,
    {
        let mut model = Self::new(space_dist);
        for ball in data {
            model.add_ball(ball, vec![]);
        }
        for vertex in model.graph.iter() {
            let neighborhood = model
                .graph
                .iter()
                .filter(|v| v.ne(&vertex))
                .get_neighborhood(&vertex.deref_data().center, |v1, v2| {
                    (model.dist)(v1, &v2.deref_data())
                });
            let neighbors = {
                let mut neighbors = vec![];
                match neighborhood {
                    Neighborhood::Two(n1, n2) => {
                        neighbors.push(Vertex::clone(n1.coord()));
                        neighbors.push(Vertex::clone(n2.coord()));
                    }
                    Neighborhood::One(n1) => {
                        neighbors.push(Vertex::clone(n1.coord()));
                    }
                    Neighborhood::None => {}
                }
                neighbors
            };
            vertex.set_neighbors(neighbors.iter().map(|v| v.as_neighbor()).collect());
        }
        model
    }

    /// Normalize the given distance function by dividing by the radius.
    fn normalize<Dist>(space_dist: Dist) -> impl Fn(&Point, &Ball<Point>) -> f64
    where
        Dist: Fn(&Point, &Point) -> f64,
    {
        move |p1: &Point, p2: &Ball<Point>| space_dist(p1, &p2.center) / p2.radius
    }

    /// Get the vertices associated to balls which the given point most probably belongs to.
    pub(crate) fn get_neighborhood(&self, point: &Point) -> Vec<BallNode<Point>> {
        let mut neighbors = vec![];
        let neighborhood = self
            .graph
            .iter()
            .get_neighborhood(point, |p, m| (self.dist)(p, &*m.deref_data()));

        match neighborhood {
            Neighborhood::Two(n1, n2) => {
                neighbors.push(Vertex::clone(n1.coord()));
                neighbors.push(Vertex::clone(n2.coord()));
            }
            Neighborhood::One(n1) => {
                neighbors.push(Vertex::clone(n1.coord()));
            }
            Neighborhood::None => {}
        }
        neighbors
    }

    /// Add a new ball or ball to the model.
    /// Balls neighbors are generally already known,
    /// thus in order to avoid unecessary calls to `Self.get_neighborhood` they are also passed.
    pub(crate) fn add_ball(
        &mut self,
        ball: Ball<Point>,
        neighbors: Vec<Neighbor<Ball<Point>>>,
    ) -> BallNode<Point> {
        let vertex = Vertex::new(ball);
        vertex.set_neighbors(neighbors);
        self.graph.push(vertex.clone());
        vertex
    }

    /// Gets an iterator over the balls of this model.
    pub fn iter_balls(&self) -> impl Iterator<Item = impl Deref<Target = Ball<Point>> + '_> {
        self.graph.iter().map(|v| v.deref_data())
    }

    /// Gets the balls that most probably include the given point.
    /// ```
    /// use fluent_data::{Model, model::Ball, space, neighborhood::{GetNeighborhood, Neighborhood}};
    ///
    /// fn main() {
    ///     let data = vec![
    ///         Ball::new(vec![4.], 3., 1.),
    ///         Ball::new(vec![5.], 2., 2.),
    ///         Ball::new(vec![3.], 3., 3.),
    ///     ];
    ///     let model = Model::load(space::euclid_dist, data.clone());
    ///     let neighborhood = model.predict(&vec![6.]);
    ///     if let Neighborhood::Two(n1, n2) = neighborhood {
    ///         assert_eq!(&data[1], n1.coord());
    ///         assert_eq!(1./2., n1.dist());
    ///         assert_eq!(&data[0], n2.coord());
    ///         assert_eq!(4./3., n2.dist());
    ///     } else {
    ///         panic!()
    ///     }
    /// }
    /// ```
    pub fn predict(
        &self,
        point: &Point,
    ) -> Neighborhood<Ball<Point>, impl Deref<Target = Ball<Point>> + '_> {
        self.iter_balls()
            .get_neighborhood(point, |p, m| (self.dist)(p, m))
    }
}

pub(crate) trait GetNeighbors<Point: PartialEq> {
    fn get_neighbors(&self) -> Vec<Neighbor<Ball<Point>>>;
}

impl<Point: PartialEq> GetNeighbors<Point> for Vec<BallNode<Point>> {
    fn get_neighbors(&self) -> Vec<Neighbor<Ball<Point>>> {
        self.iter().map(|n| n.as_neighbor()).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{model::*, space};

    #[test]
    fn test_build_norm_data() {
        let norm = Ball::new(0., 1., 11.1);
        assert_eq!(*norm.center(), 0.);
        assert_eq!(norm.radius(), 1.);
        assert_eq!(norm.weight(), 11.1);
    }

    #[test]
    fn test_model_dist() {
        let dist = Model::normalize(space::euclid_dist);
        let norm = Ball::new(vec![0.], 4., 11.1);
        let point = vec![4.];
        let d = dist(&point, &norm);
        assert_eq!(4., d);
    }

    #[test]
    fn test_model_find_neighbors() {
        let balls = vec![
            Ball::new(vec![1.], 4., 11.),
            Ball::new(vec![2.], 2., 1.),
            Ball::new(vec![6.], 1., 7.),
        ];
        let point = vec![4.];
        let dist = Model::normalize(space::euclid_dist);
        let neighbors = balls.iter().get_neighborhood(&point, dist);
        let (neighbor1, neighbor2) = if let Neighborhood::Two(neighbor1, neighbor2) = neighbors {
            (neighbor1, neighbor2)
        } else {
            panic!();
        };
        assert_eq!(&balls[1], neighbor1.coord());
        assert_eq!(2., neighbor1.dist());
        assert_eq!(&balls[0], neighbor2.coord());
        assert_eq!(2.25, neighbor2.dist());
    }

    #[test]
    fn test_model_add_ball() {
        let (model, n1, n2) = build_model();
        let mut balls = model.iter_balls();
        let c1 = &*balls.next().unwrap();
        assert_eq!(&n1, c1);
        let c2 = &*balls.next().unwrap();
        assert_eq!(&n2, c2);
    }

    #[test]
    fn test_load_model() {
        let data = vec![
            Ball::new(vec![4.], 3., 1.),
            Ball::new(vec![5.], 2., 2.),
            Ball::new(vec![3.], 3., 3.),
        ];
        let model = Model::load(space::euclid_dist, data.clone());
        let mut n1 = model.graph[0].iter_neighbors();
        assert!(n1.next().unwrap().deref_data().eq(&data[2]));
        assert!(n1.next().unwrap().deref_data().eq(&data[1]));
        let mut n2 = model.graph[1].iter_neighbors();
        assert!(n2.next().unwrap().deref_data().eq(&data[0]));
        assert!(n2.next().unwrap().deref_data().eq(&data[2]));
        let mut n3 = model.graph[2].iter_neighbors();
        assert!(n3.next().unwrap().deref_data().eq(&data[0]));
        assert!(n3.next().unwrap().deref_data().eq(&data[1]));
    }

    fn build_model() -> (Model<Vec<f64>>, Ball<Vec<f64>>, Ball<Vec<f64>>) {
        let mut model = Model::new(space::euclid_dist);
        let n1 = Ball::new(vec![4.], f64::INFINITY, 0.);
        model.add_ball(n1.clone(), vec![]);
        let p2 = vec![3.];
        let neighborhood = model.get_neighborhood(&p2);
        let n2 = Ball::new(p2, 3., 1.);
        model.add_ball(n2.clone(), neighborhood.get_neighbors());
        (model, n1, n2)
    }

    #[test]
    fn test_predict() {
        let data = vec![
            Ball::new(vec![4.], 3., 1.),
            Ball::new(vec![5.], 2., 2.),
            Ball::new(vec![3.], 3., 3.),
        ];
        let model = Model::load(space::euclid_dist, data.clone());
        let neighborhood = model.predict(&vec![6.]);
        if let Neighborhood::Two(n1, n2) = neighborhood {
            assert_eq!(&data[1], n1.coord());
            assert_eq!(1. / 2., n1.dist());
            assert_eq!(&data[0], n2.coord());
            assert_eq!(4. / 3., n2.dist());
        } else {
            panic!()
        }
    }
}
