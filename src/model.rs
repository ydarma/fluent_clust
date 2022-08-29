#[derive(Clone, Debug, PartialEq)]
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

fn model_dist<Point: 'static, Dist>(space_dist: Dist) -> impl Fn(&Point, &NormData<Point>) -> f64
where
    Dist: Fn(&Point, &Point) -> f64,
{
    Box::new(move |p1: &Point, p2: &NormData<Point>| space_dist(p1, &p2.mu) / p2.sigma)
}

#[cfg(test)]
mod tests {
    use crate::{
        graph::Vertex,
        model::*,
        neighbors::{GetNeighbors},
        space,
    };

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
        let dist = model_dist(space::euclid_dist);
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
        let point = vec![4.];
        let neighbors = graph
            .iter()
            .map(|v| v.get_data())
            .get_neighbors(&point, move |p, m| dist(p, &m));
        let data1 = &*graph[0].get_data();
        let data2 = &*graph[1].get_data();
        let neighbor1 = neighbors.0.unwrap();
        let neighbor2 = neighbors.1.unwrap();
        assert_eq!(data2, neighbor1.coord());
        assert_eq!(2., neighbor1.dist());
        assert_eq!(data1, neighbor2.coord());
        assert_eq!(2.25, neighbor2.dist());
    }
}
