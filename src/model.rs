//! The [Model] struct represents the set of balls model.

use std::ops::Deref;

use crate::{
    graph::{Neighbor, Vertex},
    neighborhood::{GetNeighborhood, Neighborhood},
};

/// A ball in the set of balls model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BallData<Point: PartialEq> {
    pub(crate) center: Point,
    pub(crate) radius: f64,
    pub(crate) weight: f64,
}

impl<Point: PartialEq> BallData<Point> {
    /// Builds a new ball.
    pub fn new(center: Point, radius: f64, weight: f64) -> Self {
        BallData { center, radius, weight }
    }

    /// Ball center.
    pub fn center(&self) -> &Point {
        &self.center
    }

    /// Ball radius.
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// Ball weight.
    pub fn weight(&self) -> f64 {
        self.weight
    }
}

/// A graph node which represents a ball.
pub(crate) type BallNode<Point> = Vertex<BallData<Point>>;

/// A set of balls model.
pub struct Model<Point: PartialEq> {
    pub(crate) dist: Box<dyn Fn(&Point, &BallData<Point>) -> f64>,
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
    pub fn load<Dist>(space_dist: Dist, data: Vec<BallData<Point>>) -> Self
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
            let neighbors = Model::get_neighbors(neighborhood);
            vertex.set_neighbors(neighbors.iter().map(|v| v.as_neighbor()).collect());
        }
        model
    }

    /// Normalize the given distance function by dividing by the radius.
    fn normalize<Dist>(space_dist: Dist) -> impl Fn(&Point, &BallData<Point>) -> f64
    where
        Dist: Fn(&Point, &Point) -> f64,
    {
        move |p1: &Point, p2: &BallData<Point>| space_dist(p1, &p2.center) / p2.radius
    }

    /// Get the balls which the given point most probably belongs to.
    pub fn get_neighborhood(&self, point: &Point) -> Vec<BallNode<Point>> {
        let neighborhood = self
            .graph
            .iter()
            .get_neighborhood(point, |p, m| (self.dist)(p, &*m.deref_data()));
        Self::get_neighbors(neighborhood)
    }

    /// Extracts `Neighbor` instance for a `Neighborhood`
    fn get_neighbors<RefNode>(
        neighborhood: Neighborhood<BallNode<Point>, RefNode>,
    ) -> Vec<BallNode<Point>>
    where
        RefNode: Deref<Target = BallNode<Point>>,
    {
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
    }

    /// Add a new ball or ball to the model.
    /// Balls neighbors are generally already known,
    /// thus in order to avoid unecessary calls to `Self.get_neighborhood` they are also passed.
    pub(crate) fn add_ball(
        &mut self,
        ball: BallData<Point>,
        neighbors: Vec<Neighbor<BallData<Point>>>,
    ) -> BallNode<Point> {
        let vertex = Vertex::new(ball);
        vertex.set_neighbors(neighbors);
        self.graph.push(vertex.clone());
        vertex
    }

    /// Gets an iterator over the balls of this model.
    pub fn iter_balls(
        &self,
    ) -> impl Iterator<Item = impl Deref<Target = BallData<Point>> + '_> {
        self.graph.iter().map(|v| v.deref_data())
    }
}

pub trait GetNeighbors<Point: PartialEq> {
    fn get_neighbors(&self) -> Vec<Neighbor<BallData<Point>>>;
}

impl<Point: PartialEq> GetNeighbors<Point> for Vec<BallNode<Point>> {
    fn get_neighbors(&self) -> Vec<Neighbor<BallData<Point>>> {
        self.iter().map(|n| n.as_neighbor()).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{model::*, space};

    #[test]
    fn test_build_norm_data() {
        let norm = BallData::new(0., 1., 11.1);
        assert_eq!(norm.center, 0.);
        assert_eq!(norm.radius, 1.);
        assert_eq!(norm.weight, 11.1);
    }

    #[test]
    fn test_model_dist() {
        let dist = Model::normalize(space::euclid_dist);
        let norm = BallData::new(vec![0.], 4., 11.1);
        let point = vec![4.];
        let d = dist(&point, &norm);
        assert_eq!(4., d);
    }

    #[test]
    fn test_model_find_neighbors() {
        let balls = vec![
            BallData::new(vec![1.], 4., 11.),
            BallData::new(vec![2.], 2., 1.),
            BallData::new(vec![6.], 1., 7.),
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
            BallData::<Vec<f64>> {
                center: vec![4.],
                radius: 3.,
                weight: 1.,
            },
            BallData::<Vec<f64>> {
                center: vec![5.],
                radius: 2.,
                weight: 2.,
            },
            BallData::<Vec<f64>> {
                center: vec![3.],
                radius: 3.,
                weight: 3.,
            },
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

    fn build_model() -> (
        Model<Vec<f64>>,
        BallData<Vec<f64>>,
        BallData<Vec<f64>>,
    ) {
        let mut model = Model::new(space::euclid_dist);
        let n1 = BallData::new(vec![4.], f64::INFINITY, 0.);
        model.add_ball(n1.clone(), vec![]);
        let p2 = vec![3.];
        let neighborhood = model.get_neighborhood(&p2);
        let n2 = BallData::new(p2, 3., 1.);
        model.add_ball(n2.clone(), neighborhood.get_neighbors());
        (model, n1, n2)
    }
}
