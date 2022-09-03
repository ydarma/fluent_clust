//! The [Algo] struct implements the algorithm that fits mixed Gaussian models from data point streams.

use std::{marker::PhantomData, ops::DerefMut};

use crate::model::{GetNeighbors, Model, GaussianData, GaussianNode};

const EXTRA_THRESHOLD: f64 = 25.;
const INTRA_THRESHOLD: f64 = 16.;
const MERGE_THRESHOLD: f64 = 1.;
const DECAY_FACTOR: f64 = 0.95;
const DECAY_THRESHOLD: f64 = 1E-2;
const MAX_NEIGHBORS: usize = 2;

/// The algorithm that fits online incoming points to a gaussian mixture model.
///
/// The algorithm can fit any kind of points in a space that:
///  - defines the distance between two points,
///  - defines the weighted center of two points.
///  ```
/// use fluent_data::algorithm::Algo;
/// use fluent_data::model::Model;
/// use fluent_data::space;
///
/// let dataset = vec![
///         vec![5., -1.],
///         vec![1., 1.],
///         vec![11., -9.],
///     ];
/// let algo = Algo::new(space::euclid_dist, space::real_combine);
/// let mut model = Model::new(space::euclid_dist);
/// for i in 0..3 {
///     algo.fit(&mut model, dataset[i].clone());
/// }
/// let mut components = model.iter_components();
/// let first = components.next().unwrap();
/// assert_eq!(&vec![6., -4.], first.mu());
/// assert_eq!(110., first.sigma());
/// assert!(first.weight() < 2.001 && first.weight() > 1.999);
/// ```
pub struct Algo<Point: PartialEq + 'static> {
    dist: Box<dyn Fn(&Point, &Point) -> f64>,
    combine: Box<dyn Fn(&Point, f64, &Point, f64) -> Point>,
    phantom: PhantomData<Point>,
}

impl<Point: PartialEq + 'static> Algo<Point> {
    /// Creates a new algorithm for the given distance and combination functions.
    pub fn new<Dist, Combine>(dist: Dist, combine: Combine) -> Self
    where
        Dist: Fn(&Point, &Point) -> f64 + 'static,
        Combine: Fn(&Point, f64, &Point, f64) -> Point + 'static,
    {
        Self {
            dist: Box::new(dist),
            combine: Box::new(combine),
            phantom: PhantomData,
        }
    }

    /// Fits the incoming points to the given mixture model.
    pub fn fit<'a>(&'a self, model: &'a mut Model<Point>, point: Point) {
        let neighborhood = model.get_neighborhood(&point);
        match neighborhood.first() {
            None =>  {
                self.init(model, point);
            }
            Some(candidate) => {
                let(vertex, maybe_neighbor) = self.update(model, candidate, point, &neighborhood);
                if let Some(maybe_neighbor) = maybe_neighbor {
                    self.update_local_graph(candidate, maybe_neighbor);
                };
                self.decay(model, vertex);
            }
        }
    }

    /// Initializes the model for the first incoming point.
    /// It creates a first components with an infinite variance and a zero weight.
    /// The second point will be merged into this component and the variance updated to the distance between the two points.
    fn init(&self, model: &mut Model<Point>, point: Point) -> GaussianNode<Point> {
        let component = GaussianData::new(point, f64::INFINITY, 0.);
        model.add_component(component, vec![])
    }

    /// Updates the model for all points after the first.
    /// If the new point is "far" from its neighbors, a new component is created
    /// otherwise it is merged into the closest one.
    /// In both case variance is calculated or updated using the distance between the point and its closest component.
    fn update(
        &self,
        model: &mut Model<Point>,
        vertex: &GaussianNode<Point>,
        point: Point,
        neighborhood: &Vec<GaussianNode<Point>>,
    ) -> (GaussianNode<Point>, Option<GaussianNode<Point>>) {
        let mut closest = vertex.deref_data_mut();
        let d = (self.dist)(&closest.mu, &point);
        if d < INTRA_THRESHOLD * closest.sigma {
            self.update_component(&mut closest, point, d);
            (vertex.clone(), neighborhood.get(1).map(|v| v.clone()))
        } else {
            let component = self.split_component(point, d, &closest);
            let vertex = model.add_component(component, neighborhood.get_neighbors());
            (vertex.clone(), Some(vertex))
        }
    }

    /// Updates the gaussian component when the given point is merged.
    /// The center is updated to the weighted center of point ansd the component.
    /// The variance is updated using the distance between the point and the component center.
    fn update_component(
        &self,
        component: &mut impl DerefMut<Target = GaussianData<Point>>,
        point: Point,
        dist: f64,
    ) {
        component.mu = self.update_mu(component, point);
        component.sigma = self.update_sigma(component, dist);
        component.weight += 1.;
    }

    /// Updates the component center to the weighted center of point ansd the component.
    fn update_mu(
        &self,
        component: &impl DerefMut<Target = GaussianData<Point>>,
        point: Point,
    ) -> Point {
        (self.combine)(&component.mu, component.weight, &point, 1.)
    }

    /// Updates the component variance using the distance between the point and the component center.
    fn update_sigma(
        &self,
        component: &impl DerefMut<Target = GaussianData<Point>>,
        dist: f64,
    ) -> f64 {
        if component.weight == 0. {
            dist
        } else {
            (component.sigma * component.weight + dist) / (component.weight + 1.)
        }
    }

    /// Creates a new component for the point.
    /// The center and the variance are calculated using the distance to its closest neighbor.
    fn split_component(
        &self,
        point: Point,
        d: f64,
        neighbor: &impl DerefMut<Target = GaussianData<Point>>,
    ) -> GaussianData<Point> {
        let sigma = d / EXTRA_THRESHOLD;
        let mu = (self.combine)(&neighbor.mu, -1., &point, 5.);
        GaussianData::new(mu, sigma, 1.)
    }

    /// Updates the neighborhood of a component with the candidate component if it is closer than its current neighbors.
    /// Then merges the component with its closest neighbor if close enough.
    fn update_local_graph(&self, vertex: &GaussianNode<Point>, candidate: GaussianNode<Point>) {
        let neighborhood: Vec<GaussianNode<Point>> = vertex.iter_neighbors().collect();
        let neighborhood = self.rebuild_neighborhood(vertex, neighborhood, candidate);
        let mut neighborhood = self.rebuild_merge(vertex, neighborhood);
        if neighborhood.len() > MAX_NEIGHBORS {
            neighborhood.pop();
        }
        vertex.set_neighbors(neighborhood.get_neighbors());
    }

    /// Updates the neighborhood of a component with the candidate component if it is closer than its current neighbors.
    fn rebuild_neighborhood(
        &self,
        vertex: &GaussianNode<Point>,
        mut neighborhood: Vec<GaussianNode<Point>>,
        candidate: GaussianNode<Point>,
    ) -> Vec<GaussianNode<Point>> {
        let current_point = &vertex.deref_data().mu;
        let dist_to_current = |p: &GaussianNode<Point>| (self.dist)(&p.deref_data().mu, &current_point);

        let candidate_dist = dist_to_current(&candidate);
        for i in 0..MAX_NEIGHBORS {
            // not enough known neighbors: push candidate
            if i == neighborhood.len() {
                neighborhood.push(candidate);
                break;
            }
            // candidate is already a known neighbor: keep known neighbors
            if neighborhood[i].eq(&candidate) {
                break;
            }
            // candidate is closer than known neighbor: insert candidate
            if dist_to_current(&neighborhood[i]) > candidate_dist {
                neighborhood.insert(i, candidate);
                break;
            }
        }
        neighborhood
    }

    /// Merges a component to its closest neighbor if it is close enough.
    fn rebuild_merge(
        &self,
        vertex: &GaussianNode<Point>,
        mut neighborhood: Vec<GaussianNode<Point>>,
    ) -> Vec<GaussianNode<Point>> {
        let (should_merge, d) = self.should_merge(vertex, &neighborhood[0]);
        if should_merge {
            self.merge_components(vertex, &neighborhood[0], d);
            neighborhood.remove(0);
        }
        neighborhood
    }

    /// Decides if two components are close enough to merge.
    fn should_merge(&self, first: &GaussianNode<Point>, second: &GaussianNode<Point>) -> (bool, f64) {
        let current_data = first.deref_data();
        let neighbor_data = second.deref_data();
        let d = (self.dist)(&current_data.mu, &neighbor_data.mu);
        let should_merge = d < (current_data.sigma + neighbor_data.sigma) * MERGE_THRESHOLD;
        (should_merge, d)
    }

    /// Merge two components.
    /// The new center is the weighted center of the component centers
    /// and the new variance is the weighted average of the components variances.
    fn merge_components(&self, vertex: &GaussianNode<Point>, neighbor: &GaussianNode<Point>, d: f64) {
        let mut current_data = vertex.deref_data_mut();
        let mut neighbor_data = neighbor.deref_data_mut();
        current_data.mu = (self.combine)(
            &current_data.mu,
            current_data.weight,
            &neighbor_data.mu,
            neighbor_data.weight,
        );
        current_data.sigma = d
            + (current_data.sigma * current_data.weight
                + neighbor_data.sigma * neighbor_data.weight)
                / (current_data.weight + neighbor_data.weight);
        current_data.weight = current_data.weight + neighbor_data.weight;
        neighbor_data.weight = 0.;
    }

    /// Decrease the weight of all components by applying decay factor.
    /// Remove components which weight is too low.
    fn decay(&self, model: &mut Model<Point>, vertex: GaussianNode<Point>) {
        vertex.deref_data_mut().weight /= DECAY_FACTOR;
        model.iter_components_mut(|v| {
            v.weight *= DECAY_FACTOR;
            v.weight > DECAY_THRESHOLD
        })
    }
}

#[cfg(test)]
mod tests {
    use approx_eq::assert_approx_eq;

    use crate::algorithm::*;
    use crate::space;

    #[test]
    fn test_init() {
        let (dataset, model) = build_model(1);
        let mut components = model.iter_components();
        let first = components.next().unwrap();
        assert_eq!(dataset[0], first.mu);
        assert_eq!(f64::INFINITY, first.sigma);
        assert_eq!(0., first.weight);
    }

    #[test]
    fn test_update() {
        let (dataset, model) = build_model(2);
        let mut components = model.iter_components();
        let first = components.next().unwrap();
        assert_eq!(dataset[1], first.mu);
        assert_eq!(20., first.sigma);
        assert_approx_eq!(1., first.weight);
    }

    #[test]
    fn test_new() {
        let (dataset, model) = build_model(3);
        let mut components = model.iter_components();
        let first = components.next().unwrap();
        assert_eq!(dataset[1], first.mu);
        assert_eq!(20., first.sigma);
        assert_approx_eq!(DECAY_FACTOR, first.weight);
        let second = components.next().unwrap();
        assert_eq!(vec![18.5, -16.5], second.mu);
        assert_eq!(15.68, second.sigma);
        assert_approx_eq!(1., second.weight);
    }

    #[test]
    fn test_neighborhood_init() {
        let (_dataset, model) = build_model(3);
        let mut components = model.iter_components();
        let first = components.next().unwrap();
        let second = components.next().unwrap();
        let mut n1 = model.graph[0].iter_neighbors();
        assert_eq!(second.mu, n1.next().unwrap().deref_data().mu);
        assert!(n1.next().is_none());
        let mut n2 = model.graph[1].iter_neighbors();
        assert_eq!(first.mu, n2.next().unwrap().deref_data().mu);
        assert!(n2.next().is_none());
    }

    #[test]
    fn test_neighborhood_refine_append() {
        let (_dataset, model) = build_model(4);
        let mut components = model.iter_components();
        let first = components.next().unwrap();
        let second = components.next().unwrap();
        let third = components.next().unwrap();
        let mut n1 = model.graph[0].iter_neighbors();
        assert_eq!(second.mu, n1.next().unwrap().deref_data().mu);
        assert_eq!(third.mu, n1.next().unwrap().deref_data().mu); // appended during refinement
        let mut n2 = model.graph[1].iter_neighbors();
        assert_eq!(first.mu, n2.next().unwrap().deref_data().mu);
        assert!(n2.next().is_none()); // not up to date for now
        let mut n3 = model.graph[2].iter_neighbors();
        assert_eq!(first.mu, n3.next().unwrap().deref_data().mu);
        assert_eq!(second.mu, n3.next().unwrap().deref_data().mu);
    }

    #[test]
    fn test_neighborhood_refine_prepend() {
        let (_dataset, model) = build_model(5);
        let mut components = model.iter_components();
        let first = components.next().unwrap();
        let second = components.next().unwrap();
        let third = components.next().unwrap();
        let fourth = components.next().unwrap();
        let mut n1 = model.graph[0].iter_neighbors();
        assert_eq!(second.mu, n1.next().unwrap().deref_data().mu);
        assert_eq!(third.mu, n1.next().unwrap().deref_data().mu);
        let mut n2 = model.graph[1].iter_neighbors();
        assert_eq!(fourth.mu, n2.next().unwrap().deref_data().mu); // prepended during refinement
        assert_eq!(first.mu, n2.next().unwrap().deref_data().mu);
        let mut n3 = model.graph[2].iter_neighbors();
        assert_eq!(first.mu, n3.next().unwrap().deref_data().mu);
        assert_eq!(second.mu, n3.next().unwrap().deref_data().mu);
    }

    #[test]
    fn test_merge() {
        let (_dataset, model) = build_model(8);
        let mut components = model.iter_components();
        let first = components.next().unwrap();
        let second = components.next().unwrap();
        let third = components.next().unwrap();
        assert!(components.next().is_none());
        assert!(first.weight > 3.);
        assert!(second.weight < 1.);
        assert!(third.weight < 1.);
        assert!(first.mu[0] < 10.);
        assert!(first.mu[1] < 0.);
        assert!(second.mu[0] > 10.);
        assert!(second.mu[1] > 0.);
        assert!(third.mu[0] > 10.);
        assert!(third.mu[1] > 0.);
        let mut n1 = model.graph[0].iter_neighbors();
        assert_eq!(third.mu, n1.next().unwrap().deref_data().mu);
        assert!(n1.next().is_none());
    }

    fn build_model(count: usize) -> (Vec<Vec<f64>>, Model<Vec<f64>>) {
        let dataset = build_sample();
        let algo = Algo::new(space::euclid_dist, space::real_combine);
        let mut model = Model::new(space::euclid_dist);
        for i in 0..count {
            algo.fit(&mut model, dataset[i].clone());
        }
        (dataset, model)
    }

    fn build_sample() -> Vec<Vec<f64>> {
        vec![
            vec![5., -1.],
            vec![1., 1.],
            vec![15., -13.],
            vec![11., 23.],
            vec![31., -3.],
            vec![10., -9.],
            vec![6., -4.],
            vec![-2., -5.]
        ]
    }
}
