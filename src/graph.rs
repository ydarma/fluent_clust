use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

/// A vertex of a graph.
pub(crate) struct Vertex<Data> {
    node: Rc<RefCell<Node<Data>>>,
}

/// Vertex internal structure, shared by vertices and edges thanks to a smart pointer.
struct Node<Data> {
    pub data: Data,
    pub neighbors: [Neighbor<Data>; 2],
}

/// A vertex neighbor. Neighbors are represented as weak pointers to avoid memory leaks.
type Neighbor<Data> = Option<Weak<RefCell<Node<Data>>>>;

type Edges<'a, Data> = [Option<&'a Vertex<Data>>; 2];

trait AsNeighbors<Data> {
    fn as_neighbors(&self) -> [Neighbor<Data>; 2];
}

impl<'a, Data> AsNeighbors<Data> for Edges<'a, Data> {
    fn as_neighbors(&self) -> [Option<Weak<RefCell<Node<Data>>>>; 2] {
        self.map(|v| v.map(|v| Rc::downgrade(&v.node)))
    }
}

impl<Data> Vertex<Data> {
    /// Build a new vertex.
    pub(crate) fn new(data: Data, edges: Edges<Data>) -> Vertex<Data> {
        Vertex {
            node: Rc::new(RefCell::new(Node {
                data,
                neighbors: edges.as_neighbors(),
            })),
        }
    }

    /// Get an iterator over the vertices that are neighbor of this vertex.
    pub(crate) fn edges(&self) -> impl Iterator<Item = Vertex<Data>> + '_ {
        EdgeIterator::new(Ref::map(self.node.borrow(), |n| &n.neighbors))
    }

    /// Update this vertex neighbors.
    pub(crate) fn set_edges(&self, edges: Edges<Data>) {
        self.node.borrow_mut().neighbors = edges.as_neighbors();
    }

    /// Get this vertex data.
    pub(crate) fn get_data(&self) -> Ref<'_, Data> {
        Ref::map(self.node.borrow(), |n| &n.data)
    }

    /// Update this vertex data.
    pub(crate) fn set_data(&self, data: Data) {
        self.node.borrow_mut().data = data;
    }
}

struct EdgeIterator<'a, Data> {
    curr: usize,
    neighbors: Ref<'a, [Neighbor<Data>; 2]>,
}

impl<'a, Data> Iterator for EdgeIterator<'a, Data> {
    type Item = Vertex<Data>;

    fn next(&mut self) -> Option<Self::Item> {
        let neighbor = self.neighbors[self.curr].as_ref()?;
        let vertex = neighbor.upgrade().map(|n| Vertex { node: n });
        self.curr += 1;
        vertex
    }
}

impl<'a, Data> EdgeIterator<'a, Data> {
    fn new(neighbors: Ref<'a, [Neighbor<Data>; 2]>) -> Self {
        EdgeIterator { curr: 0, neighbors }
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::*;

    #[test]
    fn test_build_node() {
        let n1 = Vertex::new(1, [None, None]);
        let n2 = Vertex::new(2, [Some(&n1), None]);
        let n3 = Vertex::new(3, [Some(&n1), Some(&n2)]);
        let mut e3 = n3.edges();
        assert_eq!(n1.node.as_ptr(), e3.next().unwrap().node.as_ptr());
        assert_eq!(n2.node.as_ptr(), e3.next().unwrap().node.as_ptr());
    }

    #[test]
    fn test_update_node_neighbors() {
        let n1 = Vertex::new(1, [None, None]);
        let n2 = Vertex::new(2, [Some(&n1), None]);
        let n3 = Vertex::new(3, [Some(&n1), Some(&n2)]);
        let nn2 = n2.edges().next().unwrap();
        n2.set_edges([Some(&nn2), Some(&n3)]);
        n1.set_edges([Some(&n2), Some(&n3)]);
        let mut e1 = n1.edges();
        assert_eq!(n2.node.as_ptr(), e1.next().unwrap().node.as_ptr());
        assert_eq!(n3.node.as_ptr(), e1.next().unwrap().node.as_ptr());
        let mut e2 = n2.edges();
        assert_eq!(n1.node.as_ptr(), e2.next().unwrap().node.as_ptr());
        assert_eq!(n3.node.as_ptr(), e2.next().unwrap().node.as_ptr());
    }

    #[test]
    fn test_graph_mutation() {
        let n1 = Vertex::new(1, [None, None]);
        let n2 = Vertex::new(2, [Some(&n1), None]);
        let n1_from_n2 = n2.edges().next().unwrap();
        n1_from_n2.set_data(3);
        assert_eq!(3, *n1.get_data());
    }

    #[test]
    fn test_vertex_suppression() {
        let n1 = Vertex::new(1, [None, None]);
        let n2 = Vertex::new(2, [Some(&n1), None]);
        let mut graph = vec![n1, n2];
        graph.remove(0);
        assert!(graph[0].edges().next().is_none());
    }
}
