use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

pub(crate) struct Vertex<Data> {
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
        Self::new(mean, [None, None])
    }

    pub(crate) fn new(data: Data, neighbors: [Neighbor<Data>; 2]) -> Vertex<Data> {
        Vertex {
            node: Rc::new(RefCell::new(Node { data, neighbors })),
        }
    }

    pub(crate) fn edges(&self) -> impl Iterator<Item = Vertex<Data>> + '_ {
        Edges::new(Ref::map(self.node.borrow(), |n| &n.neighbors))
    }

    pub(crate) fn get_data(&self) -> Ref<'_, Data> {
        Ref::map(self.node.borrow(), |n| &n.data)
    }

    pub(crate) fn set_data(&self, data: Data) {
        self.node.borrow_mut().data = data;
    }
}

struct Edges<'a, Data> {
    curr: usize,
    neighbors: Ref<'a, [Neighbor<Data>; 2]>,
}

impl<'a, Data> Iterator for Edges<'a, Data> {
    type Item = Vertex<Data>;

    fn next(&mut self) -> Option<Self::Item> {
        let vertex = Vertex::as_vertex(&self.neighbors[self.curr]);
        self.curr += 1;
        vertex
    }
}

impl<'a, Data> Edges<'a, Data> {
    fn new(neighbors: Ref<'a, [Neighbor<Data>; 2]>) -> Self {
        Edges { curr: 0, neighbors }
    }
}

pub struct Node<Data> {
    pub data: Data,
    pub neighbors: [Neighbor<Data>; 2],
}

impl<Point> Node<Point> {}

#[cfg(test)]
mod tests {
    use crate::graph::*;

    #[test]
    fn test_build_node() {
        let n1 = Vertex::first(1);
        let n2 = Vertex::new(2, [n1.as_neighbor(), None]);
        let n3 = Vertex::new(3, [n1.as_neighbor(), n2.as_neighbor()]);
        let mut e3 = n3.edges();
        assert_eq!(n1.node.as_ptr(), e3.next().unwrap().node.as_ptr());
        assert_eq!(
            n2.node.as_ptr(),
            e3.next().unwrap().node.as_ptr()
        );
    }

    #[test]
    fn test_graph_mutation() {
        let n1 = Vertex::first(1);
        let n2 = Vertex::new(1, [n1.as_neighbor(), None]);
        let n1_from_n2 = n2.edges().next().unwrap();
        n1_from_n2.set_data(3);
        assert_eq!(3, *n1.get_data());
    }

    #[test]
    fn test_vertex_suppression() {
        let n1 = Vertex::first(1);
        let n2 = Vertex::new(2, [n1.as_neighbor(), None]);
        let mut graph = vec![n1, n2];
        graph.remove(0);
        assert!(graph[0].edges().next().is_none());
    }
}
