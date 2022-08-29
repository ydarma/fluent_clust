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
    pub neighbors: Vec<Neighbor<Data>>,
}

/// A vertex neighbor. Neighbors are represented as weak pointers to avoid memory leaks.
type Neighbor<Data> = Weak<RefCell<Node<Data>>>;

/// Target vertices for some vertex edges.
type Edges<'a, Data> = Vec<&'a Vertex<Data>>;

trait AsNeighbors<Data> {
    /// Cast vertices as neighbors.
    fn as_neighbors(&self) -> Vec<Neighbor<Data>>;
}

impl<'a, Data> AsNeighbors<Data> for Edges<'a, Data> {
    /// Cast target vertices as node neighbors
    fn as_neighbors(&self) -> Vec<Neighbor<Data>> {
        self.iter().map(|v| Rc::downgrade(&v.node)).collect()
    }
}

impl<Data> Vertex<Data> {
    /// Build a new vertex.
    pub(crate) fn new(data: Data, edges: Vec<&Vertex<Data>>) -> Vertex<Data> {
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

/// Iterator over a reference to a `Vec` of neighbors that returns target vertices
struct EdgeIterator<'a, Data> {
    curr: usize,
    neighbors: Ref<'a, Vec<Neighbor<Data>>>,
}

impl<'a, Data> Iterator for EdgeIterator<'a, Data> {
    type Item = Vertex<Data>;

    /// Returns the next vertex.
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr >= self.neighbors.len() {
            None
        } else {
            let neighbor = self.neighbors[self.curr].upgrade();
            self.curr += 1;
            if neighbor.is_none() {
                self.next()
            } else {
                neighbor.map(|n| Vertex { node: n })
            }
        }
    }
}

impl<'a, Data> EdgeIterator<'a, Data> {
    /// Builds a new iterator.
    fn new(neighbors: Ref<'a, Vec<Neighbor<Data>>>) -> Self {
        EdgeIterator { curr: 0, neighbors }
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::*;

    #[test]
    fn test_build_node() {
        let n1 = Vertex::new(1, vec![]);
        let n2 = Vertex::new(2, vec![&n1]);
        let n3 = Vertex::new(3, vec![&n1, &n2]);
        let mut e3 = n3.edges();
        assert_eq!(n1.node.as_ptr(), e3.next().unwrap().node.as_ptr());
        assert_eq!(n2.node.as_ptr(), e3.next().unwrap().node.as_ptr());
    }

    #[test]
    fn test_update_node_neighbors() {
        let n1 = Vertex::new(1, vec![]);
        let n2 = Vertex::new(2, vec![&n1]);
        let n3 = Vertex::new(3, vec![&n1, &n2]);
        let n1_from_n2 = n2.edges().next().unwrap();
        n2.set_edges(vec![&n1_from_n2, &n3]);
        n1.set_edges(vec![&n2, &n3]);
        let mut e1 = n1.edges();
        assert_eq!(n2.node.as_ptr(), e1.next().unwrap().node.as_ptr());
        assert_eq!(n3.node.as_ptr(), e1.next().unwrap().node.as_ptr());
        let mut e2 = n2.edges();
        assert_eq!(n1.node.as_ptr(), e2.next().unwrap().node.as_ptr());
        assert_eq!(n3.node.as_ptr(), e2.next().unwrap().node.as_ptr());
        let mut e3 = n3.edges();
        assert_eq!(n1.node.as_ptr(), e3.next().unwrap().node.as_ptr());
        assert_eq!(n2.node.as_ptr(), e3.next().unwrap().node.as_ptr());
    }

    #[test]
    fn test_graph_mutation() {
        let n1 = Vertex::new(1, vec![]);
        let n2 = Vertex::new(2, vec![&n1]);
        let n1_from_n2 = n2.edges().next().unwrap();
        n1_from_n2.set_data(3);
        assert_eq!(3, *n1.get_data());
    }

    #[test]
    fn test_vertex_suppression() {
        let n1 = Vertex::new(1, vec![]);
        let n2 = Vertex::new(2, vec![&n1]);
        let n3 = Vertex::new(2, vec![&n1, &n2]);
        let mut graph = vec![n1, n2, n3];
        graph.remove(0);
        let mut e2 = graph[0].edges();
        assert!(e2.next().is_none());
        let mut e3 = graph[1].edges();
        assert_eq!(graph[0].node.as_ptr(), e3.next().unwrap().node.as_ptr());
        assert!(e3.next().is_none());
    }
}
