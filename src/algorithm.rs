//! The [Algo] struct implements the algorithm that fits a set of balls model from data point streams.

use std::{marker::PhantomData, ops::DerefMut};

use crate::model::{BallData, BallNode, GetNeighbors, Model};

const EXTRA_THRESHOLD: f64 = 25.;
const INTRA_THRESHOLD: f64 = 16.;
const MERGE_THRESHOLD: f64 = 1.;
const DECAY_FACTOR: f64 = 0.95;
const DECAY_THRESHOLD: f64 = 1E-2;
const MAX_NEIGHBORS: usize = 2;

/// Fits incoming points to a set of balls model.
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
/// let mut balls = model.iter_balls();
/// let first = balls.next().unwrap();
/// assert_eq!(&vec![6., -4.], first.center());
/// assert_eq!(110., first.radius());
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
            None => {
                self.init(model, point);
            }
            Some(candidate) => {
                let (vertex, maybe_neighbor) = self.update(model, candidate, point, &neighborhood);
                if let Some(maybe_neighbor) = maybe_neighbor {
                    self.update_local_graph(candidate, maybe_neighbor);
                };
                self.decay(model, vertex);
            }
        }
    }

    /// Initializes the model for the first incoming point.
    /// It creates a first balls with an infinite radius and a zero weight.
    /// The second point will be merged into this ball and the radius updated to the distance between the two points.
    fn init(&self, model: &mut Model<Point>, point: Point) -> BallNode<Point> {
        let ball = BallData::new(point, f64::INFINITY, 0.);
        model.add_ball(ball, vec![])
    }

    /// Updates the model for all points after the first.
    /// If the new point is "far" from its neighbors, a new ball is created
    /// otherwise it is merged into the closest one.
    /// In both case radius is calculated or updated using the distance between the point and its closest ball.
    fn update(
        &self,
        model: &mut Model<Point>,
        vertex: &BallNode<Point>,
        point: Point,
        neighborhood: &Vec<BallNode<Point>>,
    ) -> (BallNode<Point>, Option<BallNode<Point>>) {
        let mut closest = vertex.deref_data_mut();
        let d = (self.dist)(&closest.center, &point);
        if d < INTRA_THRESHOLD * closest.radius {
            self.update_ball(&mut closest, point, d);
            (vertex.clone(), neighborhood.get(1).map(|v| v.clone()))
        } else {
            let ball = self.split_ball(point, d, &closest);
            let vertex = model.add_ball(ball, neighborhood.get_neighbors());
            (vertex.clone(), Some(vertex))
        }
    }

    /// Updates the ball when the given point is merged.
    /// The center is updated to the weighted center of point ansd the ball.
    /// The radius is updated using the distance between the point and the ball center.
    fn update_ball(
        &self,
        ball: &mut impl DerefMut<Target = BallData<Point>>,
        point: Point,
        dist: f64,
    ) {
        ball.center = self.update_mu(ball, point);
        ball.radius = self.update_sigma(ball, dist);
        ball.weight += 1.;
    }

    /// Updates the ball center to the weighted center of point ansd the ball.
    fn update_mu(&self, ball: &impl DerefMut<Target = BallData<Point>>, point: Point) -> Point {
        (self.combine)(&ball.center, ball.weight, &point, 1.)
    }

    /// Updates the ball radius using the distance between the point and the ball center.
    fn update_sigma(&self, ball: &impl DerefMut<Target = BallData<Point>>, dist: f64) -> f64 {
        if ball.weight == 0. {
            dist
        } else {
            (ball.radius * ball.weight + dist) / (ball.weight + 1.)
        }
    }

    /// Creates a new ball for the point.
    /// The center and the radius are calculated using the distance to its closest neighbor.
    fn split_ball(
        &self,
        point: Point,
        d: f64,
        neighbor: &impl DerefMut<Target = BallData<Point>>,
    ) -> BallData<Point> {
        let radius = d / EXTRA_THRESHOLD;
        let center = (self.combine)(&neighbor.center, -1., &point, 5.);
        BallData::new(center, radius, 1.)
    }

    /// Updates the neighborhood of a ball with the candidate ball if it is closer than its current neighbors.
    /// Then merges the ball with its closest neighbor if close enough.
    fn update_local_graph(&self, vertex: &BallNode<Point>, candidate: BallNode<Point>) {
        let neighborhood: Vec<BallNode<Point>> = vertex.iter_neighbors().collect();
        let neighborhood = self.rebuild_neighborhood(vertex, neighborhood, candidate);
        let mut neighborhood = self.rebuild_merge(vertex, neighborhood);
        if neighborhood.len() > MAX_NEIGHBORS {
            neighborhood.pop();
        }
        vertex.set_neighbors(neighborhood.get_neighbors());
    }

    /// Updates the neighborhood of a ball with the candidate ball if it is closer than its current neighbors.
    fn rebuild_neighborhood(
        &self,
        vertex: &BallNode<Point>,
        mut neighborhood: Vec<BallNode<Point>>,
        candidate: BallNode<Point>,
    ) -> Vec<BallNode<Point>> {
        let current_point = &vertex.deref_data().center;
        let dist_to_current =
            |p: &BallNode<Point>| (self.dist)(&p.deref_data().center, &current_point);

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

    /// Merges a ball to its closest neighbor if it is close enough.
    fn rebuild_merge(
        &self,
        vertex: &BallNode<Point>,
        mut neighborhood: Vec<BallNode<Point>>,
    ) -> Vec<BallNode<Point>> {
        let (should_merge, d) = self.should_merge(vertex, &neighborhood[0]);
        if should_merge {
            self.merge_balls(vertex, &neighborhood[0], d);
            neighborhood.remove(0);
        }
        neighborhood
    }

    /// Decides if two balls are close enough to merge.
    fn should_merge(&self, first: &BallNode<Point>, second: &BallNode<Point>) -> (bool, f64) {
        let current_data = first.deref_data();
        let neighbor_data = second.deref_data();
        let d = (self.dist)(&current_data.center, &neighbor_data.center);
        let should_merge = d < (current_data.radius + neighbor_data.radius) * MERGE_THRESHOLD;
        (should_merge, d)
    }

    /// Merge two balls.
    /// The new center is the weighted center of the ball centers
    /// and the new radius is the weighted average of the balls variances.
    fn merge_balls(&self, vertex: &BallNode<Point>, neighbor: &BallNode<Point>, d: f64) {
        let mut current_data = vertex.deref_data_mut();
        let mut neighbor_data = neighbor.deref_data_mut();
        current_data.center = (self.combine)(
            &current_data.center,
            current_data.weight,
            &neighbor_data.center,
            neighbor_data.weight,
        );
        current_data.radius = d
            + (current_data.radius * current_data.weight
                + neighbor_data.radius * neighbor_data.weight)
                / (current_data.weight + neighbor_data.weight);
        current_data.weight = current_data.weight + neighbor_data.weight;
        neighbor_data.weight = 0.;
    }

    /// Decrease the weight of all balls by applying decay factor.
    /// Remove balls which weight is too low.
    fn decay(&self, model: &mut Model<Point>, vertex: BallNode<Point>) {
        model.graph.retain(|v| {
            if v.deref_data().ne(&vertex.deref_data()) {
                v.deref_data_mut().weight *= DECAY_FACTOR;
            }
            v.deref_data().weight > DECAY_THRESHOLD
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
        let mut balls = model.iter_balls();
        let first = balls.next().unwrap();
        assert_eq!(dataset[0], first.center);
        assert_eq!(f64::INFINITY, first.radius);
        assert_eq!(0., first.weight);
    }

    #[test]
    fn test_update() {
        let (dataset, model) = build_model(2);
        let mut balls = model.iter_balls();
        let first = balls.next().unwrap();
        assert_eq!(dataset[1], first.center);
        assert_eq!(20., first.radius);
        assert_eq!(1., first.weight);
    }

    #[test]
    fn test_new() {
        let (dataset, model) = build_model(3);
        let mut balls = model.iter_balls();
        let first = balls.next().unwrap();
        assert_eq!(dataset[1], first.center);
        assert_eq!(20., first.radius);
        assert_approx_eq!(DECAY_FACTOR, first.weight);
        let second = balls.next().unwrap();
        assert_eq!(vec![18.5, -16.5], second.center);
        assert_eq!(15.68, second.radius);
        assert_eq!(1., second.weight);
    }

    #[test]
    fn test_neighborhood_init() {
        let (_dataset, model) = build_model(3);
        let mut balls = model.iter_balls();
        let first = balls.next().unwrap();
        let second = balls.next().unwrap();
        let mut n1 = model.graph[0].iter_neighbors();
        assert_eq!(second.center, n1.next().unwrap().deref_data().center);
        assert!(n1.next().is_none());
        let mut n2 = model.graph[1].iter_neighbors();
        assert_eq!(first.center, n2.next().unwrap().deref_data().center);
        assert!(n2.next().is_none());
    }

    #[test]
    fn test_neighborhood_refine_append() {
        let (_dataset, model) = build_model(4);
        let mut balls = model.iter_balls();
        let first = balls.next().unwrap();
        let second = balls.next().unwrap();
        let third = balls.next().unwrap();
        let mut n1 = model.graph[0].iter_neighbors();
        assert_eq!(second.center, n1.next().unwrap().deref_data().center);
        assert_eq!(third.center, n1.next().unwrap().deref_data().center); // appended during refinement
        let mut n2 = model.graph[1].iter_neighbors();
        assert_eq!(first.center, n2.next().unwrap().deref_data().center);
        assert!(n2.next().is_none()); // not up to date for now
        let mut n3 = model.graph[2].iter_neighbors();
        assert_eq!(first.center, n3.next().unwrap().deref_data().center);
        assert_eq!(second.center, n3.next().unwrap().deref_data().center);
    }

    #[test]
    fn test_neighborhood_refine_prepend() {
        let (_dataset, model) = build_model(5);
        let mut balls = model.iter_balls();
        let first = balls.next().unwrap();
        let second = balls.next().unwrap();
        let third = balls.next().unwrap();
        let fourth = balls.next().unwrap();
        let mut n1 = model.graph[0].iter_neighbors();
        assert_eq!(second.center, n1.next().unwrap().deref_data().center);
        assert_eq!(third.center, n1.next().unwrap().deref_data().center);
        let mut n2 = model.graph[1].iter_neighbors();
        assert_eq!(fourth.center, n2.next().unwrap().deref_data().center); // prepended during refinement
        assert_eq!(first.center, n2.next().unwrap().deref_data().center);
        let mut n3 = model.graph[2].iter_neighbors();
        assert_eq!(first.center, n3.next().unwrap().deref_data().center);
        assert_eq!(second.center, n3.next().unwrap().deref_data().center);
    }

    #[test]
    fn test_merge() {
        let (_dataset, model) = build_model(8);
        let mut balls = model.iter_balls();
        let first = balls.next().unwrap();
        let second = balls.next().unwrap();
        let third = balls.next().unwrap();
        assert!(balls.next().is_none());
        assert!(first.weight > 3.);
        assert!(second.weight < 1.);
        assert!(third.weight < 1.);
        assert!(first.center[0] < 10.);
        assert!(first.center[1] < 0.);
        assert!(second.center[0] > 10.);
        assert!(second.center[1] > 0.);
        assert!(third.center[0] > 10.);
        assert!(third.center[1] > 0.);
        let mut n1 = model.graph[0].iter_neighbors();
        assert_eq!(third.center, n1.next().unwrap().deref_data().center);
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
            vec![-2., -5.],
        ]
    }
}
