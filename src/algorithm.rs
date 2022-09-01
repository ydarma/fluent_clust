use std::{marker::PhantomData, ops::DerefMut, fmt::Debug};

use crate::model::{GetNeighbors, Model, NormalData, NormalNode};

const EXTRA_FACTOR: f64 = 16.;
const INTRA_FACTOR: f64 = 9.;
const MERGE_FACTOR: f64 = 1.;

pub struct Algo<Point: Debug + PartialEq + 'static, Dist, Combine>
where
    Dist: Fn(&Point, &Point) -> f64,
    Combine: Fn(&Point, f64, &Point, f64) -> Point,
{
    dist: Dist,
    combine: Combine,
    phantom: PhantomData<Point>,
}

impl<Point: Debug + PartialEq + 'static, Dist, Combine> Algo<Point, Dist, Combine>
where
    Dist: Fn(&Point, &Point) -> f64,
    Combine: Fn(&Point, f64, &Point, f64) -> Point,
{
    pub fn new(dist: Dist, combine: Combine) -> Self {
        Self {
            dist,
            combine,
            phantom: PhantomData,
        }
    }

    pub fn fit<'a>(&'a self, model: &'a mut Model<Point>, point: Point) {
        let neighborhood = model.get_neighborhood(&point);
        match neighborhood.first() {
            None => {
                self.init(model, point);
            }
            Some(vertex) => {
                let candidate = self.update(model, vertex, point, &neighborhood);
                self.update_neighborhood(vertex, candidate);
            }
        }
    }

    fn init(&self, model: &mut Model<Point>, point: Point) {
        let component = NormalData::new(point, f64::INFINITY, 0.);
        model.add_component(component, vec![]);
    }

    fn update(
        &self,
        model: &mut Model<Point>,
        vertex: &NormalNode<Point>,
        point: Point,
        neighborhood: &Vec<NormalNode<Point>>,
    ) -> Option<crate::graph::Vertex<NormalData<Point>>> {
        let mut closest = vertex.as_data_mut();
        let d = (self.dist)(&closest.mu, &point);
        if d < INTRA_FACTOR * closest.sigma {
            self.update_component(&mut closest, point, d);
            neighborhood.get(1).map(|v| v.clone())
        } else {
            let component = self.split_component(&closest, point, d);
            let vertex = model.add_component(component, neighborhood.get_neighbors());
            Some(vertex)
        }
    }

    fn split_component(
        &self,
        component: &impl DerefMut<Target = NormalData<Point>>,
        point: Point,
        d: f64,
    ) -> NormalData<Point> {
        let sigma = d / EXTRA_FACTOR;
        let mu = (self.combine)(&component.mu, -1., &point, 5.);
        NormalData::new(mu, sigma, 1.)
    }

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

    fn update_mu(
        &self,
        component: &impl DerefMut<Target = NormalData<Point>>,
        point: Point,
    ) -> Point {
        (self.combine)(&component.mu, component.weight, &point, 1.)
    }

    fn update_sigma(
        &self,
        component: &impl DerefMut<Target = NormalData<Point>>,
        dist: f64,
    ) -> f64 {
        if component.weight == 0. {
            dist
        } else {
            (component.sigma * component.weight) + dist / (component.weight + 1.)
        }
    }

    fn update_neighborhood(
        &self,
        vertex: &NormalNode<Point>,
        maybe_candidate: Option<NormalNode<Point>>,
    ) {
        match maybe_candidate {
            Some(candidate) => {
                let vertex_neighborhood = self.refine_neighborhood(vertex, candidate);
                vertex.set_neighbors(vertex_neighborhood.get_neighbors());
            }
            None => {}
        };
    }

    fn refine_neighborhood(
        &self,
        vertex: &NormalNode<Point>,
        candidate: NormalNode<Point>,
    ) -> Vec<NormalNode<Point>> {
        let mut neighborhood: Vec<NormalNode<Point>> = vertex.iter_neighbors().collect();
        let current_point = &vertex.as_data().mu;
        let candidate_dist = (self.dist)(&candidate.as_data().mu, &current_point);
        let max_neighbors = 2;
        for i in 0..max_neighbors {
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
        // TODO: vertex may be merged with its closest neighbor
        // pop furthest known neighbor if more thant max neighbors are known
        if neighborhood.len() > max_neighbors {
            neighborhood.pop();
        }
        neighborhood
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(1., first.weight);
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

    fn build_model(count: usize) -> (Vec<Vec<f64>>, Model<Vec<f64>>) {
        let dataset = build_sample();
        let algo = Algo::new(space::euclid_dist, space::real_combine);
        let mut model = Model::new(algo.dist);
        for i in 0..count {
            algo.fit(&mut model, dataset[i].clone());
        }
        (dataset, model)
    }

    fn build_sample() -> Vec<Vec<f64>> {
        vec![
            vec![5., -1.],
            vec![1., 1.],
            vec![11., -9.],
            vec![8., 17.],
            vec![20., -3.],
            vec![8., -8.],
        ]
    }
}
