use std::{
    cell::RefCell,
    f64::INFINITY,
    rc::{Rc, Weak},
};

type Vertex<Point> = Rc<RefCell<Node<Point>>>;

type Neighbor<Point> = Option<Weak<RefCell<Node<Point>>>>;

trait AsNeighbor<Point> {
    fn as_neighbor(&self) -> Neighbor<Point>;
}

impl<Point> AsNeighbor<Point> for Vertex<Point> {
    fn as_neighbor(&self) -> Neighbor<Point> {
        Some(Rc::downgrade(self))
    }
}

pub struct Node<Point> {
    pub mean: Point,
    pub var: f64,
    pub weight: f64,
    pub neighbors: (Neighbor<Point>, Neighbor<Point>),
}

impl<Point> Node<Point> {
    pub(crate) fn first(mean: Point) -> Vertex<Point> {
        Self::new(mean, INFINITY, (None, None))
    }

    pub(crate) fn new(
        mean: Point,
        var: f64,
        neighbors: (Neighbor<Point>, Neighbor<Point>),
    ) -> Vertex<Point> {
        Rc::new(RefCell::new(Node {
            mean,
            var,
            weight: 0.,
            neighbors,
        }))
    }

    fn first_neighbor(&self) -> Option<Vertex<Point>> {
        self.neighbors.0.as_ref()?.upgrade()
    }

    fn second_neighbor(&self) -> Option<Vertex<Point>> {
        self.neighbors.1.as_ref()?.upgrade()
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::*;

    #[test]
    fn test_build_node() {
        let n1 = Node::first(vec![1., 1.]);
        let n2 = Node::new(vec![2., 2.], 1., (n1.as_neighbor(), None));
        let n3 = Node::new(vec![2., 2.], 1., (n1.as_neighbor(), n2.as_neighbor()));
        assert_eq!(n1.as_ptr(), n3.borrow().first_neighbor().unwrap().as_ptr());
        assert_eq!(n2.as_ptr(), n3.borrow().second_neighbor().unwrap().as_ptr());
    }

    #[test]
    fn test_graph_mutation() {
        let n1 = Node::first(vec![1., 1.]);
        let n2 = Node::new(vec![2., 2.], 1., (n1.as_neighbor(), None));
        let n1_from_n2 = n2.borrow().first_neighbor().unwrap();
        n1_from_n2.borrow_mut().var = 1.;
        assert_eq!(1., n1.borrow().var);
    }

    #[test]
    fn test_vertex_suppression() {
        let n1 = Node::first(vec![1., 1.]);
        let n2 = Node::new(vec![2., 2.], 1., (n1.as_neighbor(), None));
        let mut graph = vec![n1, n2];
        graph.remove(0);
        assert!(graph[0].borrow().first_neighbor().is_none());
    }
}
