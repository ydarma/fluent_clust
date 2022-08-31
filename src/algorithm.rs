use std::{marker::PhantomData, ops::DerefMut};

use crate::model::{GetNeighbors, Model, NormalData};

pub struct Algo<Point: 'static, Dist, Combine>
where
    Dist: Fn(&Point, &Point) -> f64,
    Combine: Fn(&Point, f64, &Point, f64) -> Point,
{
    dist: Dist,
    combine: Combine,
    phantom: PhantomData<Point>,
}

impl<Point: 'static, Dist, Combine> Algo<Point, Dist, Combine>
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
        let closest = neighborhood.first().map(|v| v.as_data_mut());
        match closest {
            Some(closest) => {
                let d = (self.dist)(&closest.mu, &point);
                if d < 9. * closest.sigma {
                    self.update_component(closest, point, d);
                } else {
                    let component = self.split_component(closest, &point, d);
                    model.add_component(component, neighborhood.get_neighbors());
                }
            }
            None => {
                self.init(model, point);
            }
        }
    }

    fn split_component(
        &self,
        closest: impl DerefMut<Target = NormalData<Point>>,
        point: &Point,
        d: f64,
    ) -> NormalData<Point> {
        let sigma = d / 16.;
        let mu = (self.combine)(&closest.mu, -1., point, 5.);
        NormalData::new(mu, sigma, 1.)
    }

    fn update_component(
        &self,
        mut component: impl DerefMut<Target = NormalData<Point>>,
        point: Point,
        dist: f64,
    ) {
        component.mu = self.update_mu(&component, point);
        component.sigma = self.update_sigma(&component, dist);
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
    }

    fn build_sample() -> Vec<Vec<f64>> {
        vec![vec![5., -1.], vec![1., 1.], vec![11., -9.]]
    }
}
