use std::ops::Deref;

use crate::{
    graph::Vertex,
    neighbors::{GetNeighbors, Neighborhood},
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

    fn get_neighbors(
        &self,
        point: &Point,
    ) -> Neighborhood<NormData<Point>, impl Deref<Target = NormData<Point>> + '_> {
        self.graph
            .iter()
            .map(|v| v.as_data())
            .get_neighbors(point, |p, m| (self.dist)(p, m))
    }

    fn get_data(&self) -> impl Iterator<Item = impl Deref<Target = NormData<Point>> + '_> {
        self.graph.iter().map(|v| v.as_data())
    }
}

#[cfg(test)]
mod tests {
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
        let mut model = Model::new(space::euclid_dist);
        {
            let graph = model.graph.as_mut();
            *graph = vec![
                Vertex::new(NormData::new(vec![1.], 4., 11.)),
                Vertex::new(NormData::new(vec![2.], 2., 1.)),
                Vertex::new(NormData::new(vec![6.], 1., 7.)),
            ];
            graph[0].set_neighbors(vec![graph[1].as_neighbor(), graph[2].as_neighbor()]);
            graph[1].set_neighbors(vec![graph[0].as_neighbor(), graph[2].as_neighbor()]);
            graph[2].set_neighbors(vec![graph[0].as_neighbor(), graph[1].as_neighbor()]);
        }

        let point = vec![4.];
        let neighbors = model.get_neighbors(&point);
        let mut data = model.get_data();
        let data1 = &*data.next().unwrap();
        let data2 = &*data.next().unwrap();
        let neighbor1 = neighbors.0.unwrap();
        let neighbor2 = neighbors.1.unwrap();
        assert_eq!(data2, neighbor1.coord());
        assert_eq!(2., neighbor1.dist());
        assert_eq!(data1, neighbor2.coord());
        assert_eq!(2.25, neighbor2.dist());
    }
}
