use std::{marker::PhantomData, ops::DerefMut};

use crate::model::{Model, NormalData};

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
            Some(component) => {
                self.update_component(component, &point);
            }
            None => {
                self.init(model, point);
            }
        }
    }

    fn update_component(
        &self,
        mut component: impl DerefMut<Target = NormalData<Point>>,
        point: &Point,
    ) {
        component.mu = self.update_mu(&component, point);
        component.sigma = self.update_sigma(&component, point);
        component.weight += 1.;
    }

    fn update_mu(
        &self,
        component: &impl DerefMut<Target = NormalData<Point>>,
        point: &Point,
    ) -> Point {
        (self.combine)(&component.mu, component.weight, point, 1.)
    }

    fn update_sigma(
        &self,
        component: &impl DerefMut<Target = NormalData<Point>>,
        point: &Point,
    ) -> f64 {
        let d = (self.dist)(&component.mu, point);
        if component.weight == 0. {
            d
        } else {
            (component.sigma * component.weight) + d / (component.weight + 1.)
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

    fn build_sample() -> Vec<Vec<f64>> {
        vec![vec![5., -1.], vec![1., 1.]]
    }
}
