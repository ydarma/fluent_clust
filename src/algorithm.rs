use std::{marker::PhantomData, ops::DerefMut};

use crate::model::{GetNeighbors, Model, NormalData, NormalNode};

pub struct Algo<Point: PartialEq + 'static, Dist, Combine>
where
    Dist: Fn(&Point, &Point) -> f64,
    Combine: Fn(&Point, f64, &Point, f64) -> Point,
{
    dist: Dist,
    combine: Combine,
    phantom: PhantomData<Point>,
}

impl<Point: PartialEq + 'static, Dist, Combine> Algo<Point, Dist, Combine>
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

    fn init(&self, model: &mut Model<Point>, point: Point) {
        let component = NormalData::new(point, f64::INFINITY, 0.);
        model.add_component(component, vec![]);
    }

    pub fn update<'a>(&'a self, model: &'a mut Model<Point>, point: Point) {
        let neighborhood = model.get_neighborhood(&point);
        let mut iter = neighborhood.iter();
        match iter.next() {
            Some(vertex) => {
                let maybe_candidate = {
                    let mut closest = vertex.as_data_mut();
                    let d = (self.dist)(&closest.mu, &point);
                    if d < 9. * closest.sigma {
                        self.update_component(&mut closest, point, d);
                        iter.next().map(|v| v.clone())
                    } else {
                        let component = self.split_component(&closest, &point, d);
                        let vertex = model.add_component(component, neighborhood.get_neighbors());
                        Some(vertex)
                    }
                };
                let vertex_neighborhood = match maybe_candidate {
                    Some(candidate) => {
                        self.refine_neighborhood(vertex, candidate)
                    }
                    None => {
                      vertex.iter_neighbors().collect()
                    }
                };
                vertex.set_neighbors(vertex_neighborhood.get_neighbors());
            }
            None => {
                self.init(model, point);
            }
        }
    }

    fn refine_neighborhood(
        &self,
        vertex: &NormalNode<Point>,
        candidate: NormalNode<Point>,
    ) -> Vec<NormalNode<Point>> {
        let mut neighborhood: Vec<NormalNode<Point>> = vertex.iter_neighbors().collect();
        if neighborhood.len() == 0 {
            append(&mut neighborhood, candidate);
        } else {
            let current_point = &vertex.as_data().mu;
            let candidate_dist = (self.dist)(&candidate.as_data().mu, &current_point);
            if (neighborhood[0]).ne(&candidate) {
                if self.is_closer(current_point, &neighborhood[0], &candidate, candidate_dist) {
                    shift(&mut neighborhood, candidate);
                } else if neighborhood.len() == 1 {
                    append(&mut neighborhood, candidate);
                } else if self.is_closer(
                    current_point,
                    &neighborhood[1],
                    &candidate,
                    candidate_dist,
                ) {
                    intersperse(&mut neighborhood, candidate);
                }
            }
        }
        neighborhood
    }

    fn is_closer(
        &self,
        current_point: &Point,
        neighbor: &NormalNode<Point>,
        candidate: &NormalNode<Point>,
        candidate_dist: f64,
    ) -> bool {
        (neighbor).ne(candidate)
            && (self.dist)(&neighbor.as_data().mu, &current_point) > candidate_dist
    }

    fn split_component(
        &self,
        component: &impl DerefMut<Target = NormalData<Point>>,
        point: &Point,
        d: f64,
    ) -> NormalData<Point> {
        let sigma = d / 16.;
        let mu = (self.combine)(&component.mu, -1., point, 5.);
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
}

fn shift<Point: PartialEq>(
    neighborhood: &mut Vec<NormalNode<Point>>,
    candidate: NormalNode<Point>,
) {
    neighborhood.insert(0, candidate);
    neighborhood.pop();
}

fn intersperse<Point: PartialEq>(
    neighborhood: &mut Vec<NormalNode<Point>>,
    candidate: NormalNode<Point>,
) {
    neighborhood.insert(1, candidate);
    neighborhood.pop();
}

fn append<Point: PartialEq>(
    neighborhood: &mut Vec<NormalNode<Point>>,
    candidate: NormalNode<Point>,
) {
    neighborhood.push(candidate);
}

#[cfg(test)]
mod tests {
    use crate::algorithm::*;
    use crate::space;

    #[test]
    fn test_init() {
        let dataset = build_sample();
        let algo = Algo::new(space::euclid_dist, space::real_combine);
        let mut model = Model::new(algo.dist);
        algo.update(&mut model, dataset[0].clone());
        let mut components = model.iter_components();
        let first = components.next().unwrap();
        assert_eq!(dataset[0], first.mu);
        assert_eq!(f64::INFINITY, first.sigma);
        assert_eq!(0., first.weight);
    }

    #[test]
    fn test_merge() {
        let dataset = build_sample();
        let algo = Algo::new(space::euclid_dist, space::real_combine);
        let mut model = Model::new(algo.dist);
        algo.update(&mut model, dataset[0].clone());
        algo.update(&mut model, dataset[1].clone());
        let mut components = model.iter_components();
        let first = components.next().unwrap();
        assert_eq!(dataset[1], first.mu);
        assert_eq!(20., first.sigma);
        assert_eq!(1., first.weight);
    }

    #[test]
    fn test_new() {
        let dataset = build_sample();
        let algo = Algo::new(space::euclid_dist, space::real_combine);
        let mut model = Model::new(algo.dist);
        algo.update(&mut model, dataset[0].clone());
        algo.update(&mut model, dataset[1].clone());
        algo.update(&mut model, dataset[2].clone());
        let mut components = model.iter_components();
        let first = components.next().unwrap();
        assert_eq!(dataset[1], first.mu);
        assert_eq!(20., first.sigma);
        assert_eq!(1., first.weight);
        let second = components.next().unwrap();
        assert_eq!(vec![13.5, -11.5], second.mu);
        assert_eq!(12.5, second.sigma);
        assert_eq!(1., second.weight);
        let mut n0 = model.graph[0].iter_neighbors();
        assert_eq!(second.mu, n0.next().unwrap().as_data().mu);
        let mut n1 = model.graph[1].iter_neighbors();
        assert_eq!(first.mu, n1.next().unwrap().as_data().mu);
    }

    fn build_sample() -> Vec<Vec<f64>> {
        vec![vec![5., -1.], vec![1., 1.], vec![11., -9.]]
    }
}
