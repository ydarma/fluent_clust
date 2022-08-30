use std::ops::Deref;

use crate::{
    graph::{Neighbor, Vertex},
    neighbors::{GetNeighborhood, Neighborhood},
};

/// Parameters of a normal component.
#[derive(Clone, Copy, Debug, PartialEq)]
struct NormalData<Point> {
    /// mean
    mu: Point,
    /// variance
    sigma: f64,
    /// weight
    weight: f64,
}

impl<Point> NormalData<Point> {
    /// Builds a new normal component.
    fn new(mu: Point, sigma: f64, weight: f64) -> Self {
        NormalData { mu, sigma, weight }
    }
}

type NormalNode<Point> = Vertex<NormalData<Point>>;

/// Represents a mixed model.
struct Model<Point> {
    dist: Box<dyn Fn(&Point, &NormalData<Point>) -> f64>,
    graph: Vec<NormalNode<Point>>,
}

impl<Point: 'static> Model<Point> {
    /// Builds a new model.
    pub(crate) fn new<Dist>(space_dist: Dist) -> Self
    where
        Dist: Fn(&Point, &Point) -> f64 + 'static,
    {
        Self {
            dist: Box::new(Model::normalize_dist(space_dist)),
            graph: vec![],
        }
    }

    /// Normalizes the given distance function by dividing by the variance.
    fn normalize_dist<Dist>(space_dist: Dist) -> impl Fn(&Point, &NormalData<Point>) -> f64
    where
        Dist: Fn(&Point, &Point) -> f64,
    {
        Box::new(move |p1: &Point, p2: &NormalData<Point>| space_dist(p1, &p2.mu) / p2.sigma)
    }

    /// Get the components which the given points most probably belongs to.
    pub(crate) fn get_neighborhood(
        &self,
        point: &Point,
    ) -> Neighborhood<NormalNode<Point>, impl Deref<Target = NormalNode<Point>> + '_>
    {
        self.graph
            .iter()
            .get_neighborhood(point, |p, m| (self.dist)(p, &*m.as_data()))
    }

    /// Extracts `Neighbor` instance for a `Neighborhood`
    pub(crate) fn get_neighbors<RefNode>(
        neighborhood: Neighborhood<NormalNode<Point>, RefNode>,
    ) -> Vec<Neighbor<NormalData<Point>>>
    where
        RefNode: Deref<Target = NormalNode<Point>>,
    {
        let mut neighbors = vec![];
        match neighborhood.0 {
            Some(n1) => {
                neighbors.push(n1.coord().as_neighbor());
                match neighborhood.1 {
                    Some(n2) => neighbors.push(n2.coord().as_neighbor()),
                    _ => {}
                }
            }
            _ => {}
        }
        neighbors
    }

    /// Add a new component to the mixed model.
    /// Components neighbors are generally already known,
    /// thus in order to avoid unecessary calls to `Self.get_neighborhood` they are also passed.
    pub(crate) fn add_component(
        &mut self,
        component: NormalData<Point>,
        neighbors: Vec<Neighbor<NormalData<Point>>>,
    ) {
        let i = self.graph.len();
        self.graph.push(Vertex::new(component));
        self.graph[i].set_neighbors(neighbors);
    }

    /// Gets an iterator over the model components.
    pub(crate) fn iter_components(&self) -> impl Iterator<Item = impl Deref<Target = NormalData<Point>> + '_> {
        self.graph.iter().map(|v| v.as_data())
    }

    /// Mutate the model components in sequence. The closure should return `true` to retain the components or `false` to discard it.
    pub(crate) fn iter_components_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut NormalData<Point>) -> bool,
    {
        self.graph.retain(|v| f(&mut *v.as_data_mut()))
    }
}

#[cfg(test)]
mod tests {
    use std::f64::INFINITY;

    use crate::{model::*, space};

    #[test]
    fn test_build_norm_data() {
        let norm = NormalData::new(0., 1., 11.1);
        assert_eq!(norm.mu, 0.);
        assert_eq!(norm.sigma, 1.);
        assert_eq!(norm.weight, 11.1);
    }

    #[test]
    fn test_model_dist() {
        let dist = Model::normalize_dist(space::euclid_dist);
        let norm = NormalData::new(vec![0.], 4., 11.1);
        let point = vec![4.];
        let d = dist(&point, &norm);
        assert_eq!(4., d);
    }

    #[test]
    fn test_model_find_neighbors() {
        let components = vec![
            NormalData::new(vec![1.], 4., 11.),
            NormalData::new(vec![2.], 2., 1.),
            NormalData::new(vec![6.], 1., 7.),
        ];
        let point = vec![4.];
        let dist = Model::normalize_dist(space::euclid_dist);
        let neighbors = components.iter().get_neighborhood(&point, dist);
        let neighbor1 = neighbors.0.unwrap();
        let neighbor2 = neighbors.1.unwrap();
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

    #[test]
    fn test_model_update_component() {
        let (mut model, n1, n2) = build_model();
        model.iter_components_mut(|component| {
            component.weight *= 0.95;
            true
        });
        let mut components = model.iter_components();
        let c1 = &*components.next().unwrap();
        assert_eq!(n1.weight * 0.95, c1.weight);
        let c2 = &*components.next().unwrap();
        assert_eq!(n2.weight * 0.95, c2.weight);
    }

    #[test]
    fn test_model_remove_component() {
        let (mut model, _n1, n2) = build_model();
        model.iter_components_mut(|component| component.weight != 0.);
        let mut components = model.iter_components();
        let c1 = &*components.next().unwrap();
        assert_eq!(&n2, c1);
        let c2 = components.next();
        assert!(c2.is_none());
        assert_eq!(1, model.graph.len());
        assert_eq!(&n2, &*model.graph[0].as_data());
        assert!(model.graph[0].iter_neighbors().next().is_none());
    }

    fn build_model() -> (Model<Vec<f64>>, NormalData<Vec<f64>>, NormalData<Vec<f64>>) {
        let mut model = Model::new(space::euclid_dist);
        let n1 = NormalData::new(vec![4.], INFINITY, 0.);
        model.add_component(n1.clone(), vec![]);
        let p2 = vec![3.];
        let neighborhood = model.get_neighborhood(&p2);
        let neighbors = Model::get_neighbors(neighborhood);
        let n2 = NormalData::new(p2, 3., 1.);
        model.add_component(n2.clone(), neighbors);
        (model, n1, n2)
    }
}
