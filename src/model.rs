//! The [Model] struct represents the mixed Gaussian model fited by the algorithm.

use std::ops::Deref;

use crate::{
    graph::{Neighbor, Vertex},
    neighborhood::{GetNeighborhood, Neighborhood},
};

/// Parameters of a gaussian component.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GaussianData<Point: PartialEq> {
    pub(crate) mu: Point,
    pub(crate) sigma: f64,
    pub(crate) weight: f64,
}

impl<Point: PartialEq> GaussianData<Point> {
    /// Builds a new gaussian component.
    pub fn new(mu: Point, sigma: f64, weight: f64) -> Self {
        GaussianData { mu, sigma, weight }
    }

    /// Mean.
    pub fn mu(&self) -> &Point {
        &self.mu
    }

    /// Variance.
    pub fn sigma(&self) -> f64 {
        self.sigma
    }

    /// Weight
    pub fn weight(&self) -> f64 {
        self.weight
    }
}

pub type GaussianNode<Point> = Vertex<GaussianData<Point>>;

/// A mixed gaussian model.
pub struct Model<Point: PartialEq> {
    pub(crate) dist: Box<dyn Fn(&Point, &GaussianData<Point>) -> f64>,
    pub(crate) graph: Vec<GaussianNode<Point>>,
}

impl<Point: PartialEq + 'static> Model<Point> {
    /// Builds a new model.
    pub fn new<Dist>(space_dist: Dist) -> Self
    where
        Dist: Fn(&Point, &Point) -> f64 + 'static,
    {
        Self {
            dist: Box::new(Model::gaussianize_dist(space_dist)),
            graph: vec![],
        }
    }

    /// Gaussianizes the given distance function by dividing by the variance.
    fn gaussianize_dist<Dist>(space_dist: Dist) -> impl Fn(&Point, &GaussianData<Point>) -> f64
    where
        Dist: Fn(&Point, &Point) -> f64,
    {
        move |p1: &Point, p2: &GaussianData<Point>| space_dist(p1, &p2.mu) / p2.sigma
    }

    /// Get the components which the given points most probably belongs to.
    pub fn get_neighborhood(&self, point: &Point) -> Vec<GaussianNode<Point>> {
        let neighborhood = self
            .graph
            .iter()
            .get_neighborhood(point, |p, m| (self.dist)(p, &*m.deref_data()));
        Self::get_neighbors(neighborhood)
    }

    /// Extracts `Neighbor` instance for a `Neighborhood`
    fn get_neighbors<RefNode>(
        neighborhood: Neighborhood<GaussianNode<Point>, RefNode>,
    ) -> Vec<GaussianNode<Point>>
    where
        RefNode: Deref<Target = GaussianNode<Point>>,
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

    /// Add a new component to the mixed model.
    /// Components neighbors are generally already known,
    /// thus in order to avoid unecessary calls to `Self.get_neighborhood` they are also passed.
    pub(crate) fn add_component(
        &mut self,
        component: GaussianData<Point>,
        neighbors: Vec<Neighbor<GaussianData<Point>>>,
    ) -> GaussianNode<Point> {
        let vertex = Vertex::new(component);
        vertex.set_neighbors(neighbors);
        self.graph.push(vertex.clone());
        vertex
    }

    /// Gets an iterator over the model components.
    pub fn iter_components(
        &self,
    ) -> impl Iterator<Item = impl Deref<Target = GaussianData<Point>> + '_> {
        self.graph.iter().map(|v| v.deref_data())
    }
}

pub trait GetNeighbors<Point: PartialEq> {
    fn get_neighbors(&self) -> Vec<Neighbor<GaussianData<Point>>>;
}

impl<Point: PartialEq> GetNeighbors<Point> for Vec<GaussianNode<Point>> {
    fn get_neighbors(&self) -> Vec<Neighbor<GaussianData<Point>>> {
        self.iter().map(|n| n.as_neighbor()).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{model::*, space};

    #[test]
    fn test_build_norm_data() {
        let norm = GaussianData::new(0., 1., 11.1);
        assert_eq!(norm.mu, 0.);
        assert_eq!(norm.sigma, 1.);
        assert_eq!(norm.weight, 11.1);
    }

    #[test]
    fn test_model_dist() {
        let dist = Model::gaussianize_dist(space::euclid_dist);
        let norm = GaussianData::new(vec![0.], 4., 11.1);
        let point = vec![4.];
        let d = dist(&point, &norm);
        assert_eq!(4., d);
    }

    #[test]
    fn test_model_find_neighbors() {
        let components = vec![
            GaussianData::new(vec![1.], 4., 11.),
            GaussianData::new(vec![2.], 2., 1.),
            GaussianData::new(vec![6.], 1., 7.),
        ];
        let point = vec![4.];
        let dist = Model::gaussianize_dist(space::euclid_dist);
        let neighbors = components.iter().get_neighborhood(&point, dist);
        let (neighbor1, neighbor2) = if let Neighborhood::Two(neighbor1, neighbor2) = neighbors {
            (neighbor1, neighbor2)
        } else {
            panic!();
        };
        assert_eq!(&components[1], neighbor1.coord());
        assert_eq!(2., neighbor1.dist());
        assert_eq!(&components[0], neighbor2.coord());
        assert_eq!(2.25, neighbor2.dist());
    }

    #[test]
    fn test_model_add_component() {
        let (model, n1, n2) = build_model();
        let mut components = model.iter_components();
        let c1 = &*components.next().unwrap();
        assert_eq!(&n1, c1);
        let c2 = &*components.next().unwrap();
        assert_eq!(&n2, c2);
    }

    fn build_model() -> (
        Model<Vec<f64>>,
        GaussianData<Vec<f64>>,
        GaussianData<Vec<f64>>,
    ) {
        let mut model = Model::new(space::euclid_dist);
        let n1 = GaussianData::new(vec![4.], f64::INFINITY, 0.);
        model.add_component(n1.clone(), vec![]);
        let p2 = vec![3.];
        let neighborhood = model.get_neighborhood(&p2);
        let n2 = GaussianData::new(p2, 3., 1.);
        model.add_component(n2.clone(), neighborhood.get_neighbors());
        (model, n1, n2)
    }
}
