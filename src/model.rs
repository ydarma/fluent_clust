use crate::space::DistFn;

#[derive(Clone)]
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

fn model_dist<Point: 'static>(space_dist: DistFn<Point, Point>) -> DistFn<Point, NormData<Point>> {
    Box::new(move |p1: &Point, p2: &NormData<Point>| space_dist(p1, &p2.mu) / p2.sigma)
}

#[cfg(test)]
mod tests {
    use crate::{graph::Vertex, model::*, space, neighbors::{GetNeighbors, self}};

    #[test]
    fn test_build_norm_data() {
        let norm = NormData::new(0., 1., 11.1);
        assert_eq!(norm.mu, 0.);
        assert_eq!(norm.sigma, 1.);
        assert_eq!(norm.weight, 11.1);
    }

    #[test]
    fn test_model_dist() {
        let dist = model_dist(Box::new(space::euclid_dist));
        let norm = NormData::new(vec![0.], 4., 11.1);
        let point = vec![4.];
        let d = dist(&point, &norm);
        assert_eq!(4., d);
    }

    #[test]
    fn test_model_find_neighbors() {
        let dist = model_dist(Box::new(space::euclid_dist));
        let norm1 = NormData::new(vec![1.], 4., 11.);
        let norm2 = NormData::new(vec![2.], 2., 1.);
        let norm3 = NormData::new(vec![6.], 1., 7.);
        let graph = vec![
            Vertex::new(norm1, vec![]),
            Vertex::new(norm2, vec![]),
            Vertex::new(norm3, vec![]),
        ];
        graph[0].set_edges(vec![&graph[1], &graph[2]]);
        graph[1].set_edges(vec![&graph[0], &graph[2]]);
        graph[2].set_edges(vec![&graph[0], &graph[1]]);
        let graph_dist = move |p1: &Vec<f64>, p2: &Vertex<NormData<Vec<f64>>>| {
          let data = p2.get_data();
          dist(p1, &data)
        };
        let point = vec![4.];
        let neighbors = graph.iter().get_neighbors(&point, Box::new(graph_dist));
    }
}
