use std::ops::{Deref, DerefMut};

use crate::{
    graph::{Neighbor, Vertex},
    neighbors::{self, GetNeighborhood, Neighborhood},
};

#[derive(Clone, Copy, Debug, PartialEq)]
struct NormData<Point> {
    mu: Point,
    sigma: f64,
    weight: f64,
}

impl<Point> NormData<Point> {
    fn new(mu: Point, sigma: f64, weight: f64) -> Self {
        NormData { mu, sigma, weight }
    }
}

fn model_dist<Point, Dist>(space_dist: Dist) -> impl Fn(&Point, &NormData<Point>) -> f64
where
    Dist: Fn(&Point, &Point) -> f64,
{
    Box::new(move |p1: &Point, p2: &NormData<Point>| space_dist(p1, &p2.mu) / p2.sigma)
}

struct Model<Point> {
    dist: Box<dyn Fn(&Point, &NormData<Point>) -> f64>,
    graph: Vec<Vertex<NormData<Point>>>,
}

impl<Point: 'static> Model<Point> {
    fn new<Dist>(space_dist: Dist) -> Self
    where
        Dist: Fn(&Point, &Point) -> f64 + 'static,
    {
        Self {
            dist: Box::new(model_dist(space_dist)),
            graph: vec![],
        }
    }

    fn get_neighborhood(
        &self,
        point: &Point,
    ) -> Neighborhood<Vertex<NormData<Point>>, impl Deref<Target = Vertex<NormData<Point>>> + '_>
    {
        self.graph
            .iter()
            .get_neighborhood(point, |p, m| (self.dist)(p, &*m.as_data()))
    }

    fn get_data(&self) -> impl Iterator<Item = impl Deref<Target = NormData<Point>> + '_> {
        self.graph.iter().map(|v| v.as_data())
    }

    fn add_component(
        &mut self,
        component: NormData<Point>,
        neighbors: Vec<Neighbor<NormData<Point>>>,
    ) {
        let i = self.graph.len();
        self.graph.push(Vertex::new(component));
        self.graph[i].set_neighbors(neighbors);
    }

    fn get_neighbors<RefPoint>(
        neighborhood: Neighborhood<Vertex<NormData<Point>>, RefPoint>,
    ) -> Vec<Neighbor<NormData<Point>>>
    where
        RefPoint: Deref<Target = Vertex<NormData<Point>>>,
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

    fn get_components(&self) -> impl Iterator<Item = impl Deref<Target = NormData<Point>> + '_> {
        self.graph.iter().map(|v| v.as_data())
    }

    fn get_components_mut(
        &self,
    ) -> impl Iterator<Item = impl DerefMut<Target = NormData<Point>> + '_> {
        self.graph.iter().map(|v| v.as_data_mut())
    }
}

#[cfg(test)]
mod tests {
    use std::f64::INFINITY;

    use crate::{graph::Vertex, model::*, space};

    #[test]
    fn test_build_norm_data() {
        let norm = NormData::new(0., 1., 11.1);
        assert_eq!(norm.mu, 0.);
        assert_eq!(norm.sigma, 1.);
        assert_eq!(norm.weight, 11.1);
    }

    #[test]
    fn test_model_dist() {
        let dist = model_dist(space::euclid_dist);
        let norm = NormData::new(vec![0.], 4., 11.1);
        let point = vec![4.];
        let d = dist(&point, &norm);
        assert_eq!(4., d);
    }

    #[test]
    fn test_model_find_neighbors() {
        let components = vec![
            NormData::new(vec![1.], 4., 11.),
            NormData::new(vec![2.], 2., 1.),
            NormData::new(vec![6.], 1., 7.),
        ];
        let point = vec![4.];
        let dist = model_dist(space::euclid_dist);
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
        let mut components = model.get_components();
        let c1 = &*components.next().unwrap();
        assert_eq!(&n1, c1);
        let c2 = &*components.next().unwrap();
        assert_eq!(&n2, c2);
    }

    #[test]
    fn test_model_update_component() {
        let (model, n1, n2) = build_model();
        for mut component in model.get_components_mut() {
            component.weight *= 0.85;
        }
        let mut components = model.get_components();
        let c1 = &*components.next().unwrap();
        assert_eq!(n1.weight * 0.95, c1.weight);
        let c2 = &*components.next().unwrap();
        assert_eq!(n2.weight * 0.95, c2.weight);
    }

    fn build_model() -> (Model<Vec<f64>>, NormData<Vec<f64>>, NormData<Vec<f64>>) {
        let mut model = Model::new(space::euclid_dist);
        let n1 = NormData::new(vec![4.], INFINITY, 0.);
        model.add_component(n1.clone(), vec![]);
        let p2 = vec![3.];
        let neighborhood = model.get_neighborhood(&p2);
        let neighbors = Model::get_neighbors(neighborhood);
        let n2 = NormData::new(p2, 3., 0.);
        model.add_component(n2.clone(), neighbors);
        (model, n1, n2)
    }
}
