use std::{marker::PhantomData, ops::DerefMut};

use crate::model::{GetNeighbors, Model, NormalData, NormalNode};

const EXTRA_THRESHOLD: f64 = 16.;
const INTRA_THRESHOLD: f64 = 9.;
const MERGE_THRESHOLD: f64 = 1.;
const DECAY_FACTOR: f64 = 0.999;
const DECAY_THRESHOLD: f64 = 1E-6;
const MAX_NEIGHBORS: usize = 2;

/// The algorithm that fits incoming point to a normal mixture model.
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
/// assert_eq!(&dataset[1], first.mu());
/// assert_eq!(20., first.sigma());
/// assert_eq!(0.999, first.weight());
/// let second = components.next().unwrap();
/// assert_eq!(&vec![13.5, -11.5], second.mu());
/// assert_eq!(12.5, second.sigma());
/// assert_eq!(1., second.weight());
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
    fn init(&self, model: &mut Model<Point>, point: Point) -> NormalNode<Point> {
        let component = NormalData::new(point, f64::INFINITY, 0.);
        model.add_component(component, vec![])
    }

    /// Updates the model for all points after the first.
    /// If the new point is "far" from its neighbors, a new component is created
    /// otherwise it is merged into the closest one.
    /// In both case variance is calculated or updated using the distance between the point and its closest component.
    fn update(
        &self,
        model: &mut Model<Point>,
        vertex: &NormalNode<Point>,
        point: Point,
        neighborhood: &Vec<NormalNode<Point>>,
    ) -> (NormalNode<Point>, Option<NormalNode<Point>>) {
        let mut closest = vertex.as_data_mut();
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

    /// Updates the normal component when the given point is merged.
    /// The center is updated to the weighted center of point ansd the component.
    /// The variance is updated using the distance between the point and the component center.
    fn update_component(
        &self,
        component: &mut impl DerefMut<Target = NormalData<Point>>,
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
        component: &impl DerefMut<Target = NormalData<Point>>,
        point: Point,
    ) -> Point {
        (self.combine)(&component.mu, component.weight, &point, 1.)
    }

    /// Updates the component variance using the distance between the point and the component center.
    fn update_sigma(
        &self,
        component: &impl DerefMut<Target = NormalData<Point>>,
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
        neighbor: &impl DerefMut<Target = NormalData<Point>>,
    ) -> NormalData<Point> {
        let sigma = d / EXTRA_THRESHOLD;
        let mu = (self.combine)(&neighbor.mu, -1., &point, 5.);
        NormalData::new(mu, sigma, 1.)
    }

    /// Updates the neighborhood of a component with the candidate component if it is closer than its current neighbors.
    /// Then merges the component with its closest neighbor if close enough.
    fn update_local_graph(&self, vertex: &NormalNode<Point>, candidate: NormalNode<Point>) {
        let neighborhood: Vec<NormalNode<Point>> = vertex.iter_neighbors().collect();
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
        vertex: &NormalNode<Point>,
        mut neighborhood: Vec<NormalNode<Point>>,
        candidate: NormalNode<Point>,
    ) -> Vec<NormalNode<Point>> {
        let current_point = &vertex.as_data().mu;
        let candidate_dist = (self.dist)(&candidate.as_data().mu, &current_point);
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
            if (self.dist)(&neighborhood[i].as_data().mu, &current_point) > candidate_dist {
                neighborhood.insert(i, candidate);
                break;
            }
        }
        neighborhood
    }

    /// Merges a component to its closest neighbor if it is close enough.
    fn rebuild_merge(
        &self,
        vertex: &NormalNode<Point>,
        mut neighborhood: Vec<NormalNode<Point>>,
    ) -> Vec<NormalNode<Point>> {
        let (should_merge, d) = self.should_merge(vertex, &neighborhood[0]);
        if should_merge {
            self.merge_components(vertex, &neighborhood[0], d);
            neighborhood.remove(0);
        }
        neighborhood
    }

    /// Decides if two components are close enough to merge.
    fn should_merge(&self, first: &NormalNode<Point>, second: &NormalNode<Point>) -> (bool, f64) {
        let current_data = first.as_data();
        let neighbor_data = second.as_data();
        let d = (self.dist)(&current_data.mu, &neighbor_data.mu);
        let should_merge = d < (current_data.sigma + neighbor_data.sigma) * MERGE_THRESHOLD;
        (should_merge, d)
    }

    /// Merge two components.
    /// The new center is the weighted center of the component centers
    /// and the new variance is the weighted average of the components variances.
    fn merge_components(&self, vertex: &NormalNode<Point>, neighbor: &NormalNode<Point>, d: f64) {
        let mut current_data = vertex.as_data_mut();
        let mut neighbor_data = neighbor.as_data_mut();
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
    fn decay(&self, model: &mut Model<Point>, vertex: NormalNode<Point>) {
        vertex.as_data_mut().weight /= DECAY_FACTOR;
        model.iter_components_mut(|v| {
            v.weight *= DECAY_FACTOR;
            v.weight > DECAY_THRESHOLD
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
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
        assert_eq!(1., first.weight);
    }

    #[test]
    fn test_new() {
        let (dataset, model) = build_model(3);
        let mut components = model.iter_components();
        let first = components.next().unwrap();
        assert_eq!(dataset[1], first.mu);
        assert_eq!(20., first.sigma);
        assert_eq!(DECAY_FACTOR, first.weight);
        let second = components.next().unwrap();
        assert_eq!(vec![13.5, -11.5], second.mu);
        assert_eq!(12.5, second.sigma);
        assert_eq!(1., second.weight);
    }

    #[test]
    fn test_neighborhood_init() {
        let (_dataset, model) = build_model(3);
        let mut components = model.iter_components();
        let first = components.next().unwrap();
        let second = components.next().unwrap();
        let mut n1 = model.graph[0].iter_neighbors();
        assert_eq!(second.mu, n1.next().unwrap().as_data().mu);
        assert!(n1.next().is_none());
        let mut n2 = model.graph[1].iter_neighbors();
        assert_eq!(first.mu, n2.next().unwrap().as_data().mu);
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
        assert_eq!(second.mu, n1.next().unwrap().as_data().mu);
        assert_eq!(third.mu, n1.next().unwrap().as_data().mu); // appended during refinement
        let mut n2 = model.graph[1].iter_neighbors();
        assert_eq!(first.mu, n2.next().unwrap().as_data().mu);
        assert!(n2.next().is_none()); // not up to date for now
        let mut n3 = model.graph[2].iter_neighbors();
        assert_eq!(first.mu, n3.next().unwrap().as_data().mu);
        assert_eq!(second.mu, n3.next().unwrap().as_data().mu);
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
        assert_eq!(second.mu, n1.next().unwrap().as_data().mu);
        assert_eq!(third.mu, n1.next().unwrap().as_data().mu);
        let mut n2 = model.graph[1].iter_neighbors();
        assert_eq!(fourth.mu, n2.next().unwrap().as_data().mu); // prepended during refinement
        assert_eq!(first.mu, n2.next().unwrap().as_data().mu);
        let mut n3 = model.graph[2].iter_neighbors();
        assert_eq!(first.mu, n3.next().unwrap().as_data().mu);
        assert_eq!(second.mu, n3.next().unwrap().as_data().mu);
    }

    #[test]
    fn test_merge() {
        let (_dataset, model) = build_model(37);
        let mut components = model.iter_components();
        let first = components.next().unwrap();
        let second = components.next().unwrap();
        let third = components.next().unwrap();
        assert!(components.next().is_none());
        assert!(first.weight > 30.);
        assert!(second.weight < 1.);
        assert!(third.weight < 1.);
        assert!(first.mu[0] < 6.);
        assert!(first.mu[1] < -2.);
        assert!(second.mu[0] > 6.);
        assert!(second.mu[1] > -2.);
        assert!(third.mu[0] > 6.);
        assert!(third.mu[1] > -2.);
        let mut n1 = model.graph[0].iter_neighbors();
        assert_eq!(third.mu, n1.next().unwrap().as_data().mu);
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

    pub(crate) fn build_sample() -> Vec<Vec<f64>> {
        vec![
            vec![5., -1.],
            vec![1., 1.],
            vec![11., -9.],
            vec![8., 17.],
            vec![20., -3.],
            vec![8., -8.],
            vec![5., -4.],
            vec![4., -6.],
            vec![7., -6.],
            vec![3., -5.],
            vec![5., -6.],
            vec![5., -6.],
            vec![5., -5.],
            vec![3., -4.],
            vec![3., -3.],
            vec![5., -5.],
            vec![5., -4.],
            vec![7., -6.],
            vec![6., -5.],
            vec![6., -4.],
            vec![5., -3.],
            vec![3., -4.],
            vec![4., -5.],
            vec![4., -4.],
            vec![6., -6.],
            vec![5., -4.],
            vec![4., -6.],
            vec![7., -6.],
            vec![2., -2.],
            vec![3., -3.],
            vec![6., -5.],
            vec![6., -4.],
            vec![4., -5.],
            vec![4., -4.],
            vec![4., -3.],
            vec![4., -3.],
            vec![6., -6.],
        ]
    }
}
