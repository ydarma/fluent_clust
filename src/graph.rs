use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

struct Vertex<Data> {
    node: Rc<RefCell<Node<Data>>>,
}

type Neighbor<Data> = Option<Weak<RefCell<Node<Data>>>>;

impl<Data> Vertex<Data> {
    fn as_neighbor(&self) -> Neighbor<Data> {
        Some(Rc::downgrade(&self.node))
    }

    fn as_vertex(neighbor: &Neighbor<Data>) -> Option<Vertex<Data>> {
        neighbor.as_ref()?.upgrade().map(|n| Vertex { node: n })
    }

    pub(crate) fn first(mean: Data) -> Vertex<Data> {
        Self::new(mean, (None, None))
    }

    pub(crate) fn new(data: Data, neighbors: (Neighbor<Data>, Neighbor<Data>)) -> Vertex<Data> {
        Vertex {
            node: Rc::new(RefCell::new(Node { data, neighbors })),
        }
    }

    fn first_neighbor(&self) -> Option<Vertex<Data>> {
        Vertex::as_vertex(&self.node.borrow().neighbors.0)
    }

    fn second_neighbor(&self) -> Option<Vertex<Data>> {
        Vertex::as_vertex(&self.node.borrow().neighbors.1)
    }

    fn get_data(&self) -> Ref<'_, Data> {
        Ref::map(self.node.borrow(), |n| &n.data)
    }

    fn set_data(&self, data: Data) {
        self.node.borrow_mut().data = data;
    }
}

pub struct Node<Data> {
    pub data: Data,
    pub neighbors: (Neighbor<Data>, Neighbor<Data>),
}

impl<Point> Node<Point> {}

#[cfg(test)]
mod tests {
    use crate::graph::*;

    #[test]
    fn test_build_node() {
        let n1 = Vertex::first(1);
        let n2 = Vertex::new(2, (n1.as_neighbor(), None));
        let n3 = Vertex::new(3, (n1.as_neighbor(), n2.as_neighbor()));
        assert_eq!(n1.node.as_ptr(), n3.first_neighbor().unwrap().node.as_ptr());
        assert_eq!(
            n2.node.as_ptr(),
            n3.second_neighbor().unwrap().node.as_ptr()
        );
    }

    #[test]
    fn test_graph_mutation() {
        let n1 = Vertex::first(1);
        let n2 = Vertex::new(1, (n1.as_neighbor(), None));
        let n1_from_n2 = n2.first_neighbor().unwrap();
        n1_from_n2.set_data(3);
        assert_eq!(3, *n1.get_data());
    }

    #[test]
    fn test_vertex_suppression() {
        let n1 = Vertex::first(1);
        let n2 = Vertex::new(2, (n1.as_neighbor(), None));
        let mut graph = vec![n1, n2];
        graph.remove(0);
        assert!(graph[0].first_neighbor().is_none());
    }
}
