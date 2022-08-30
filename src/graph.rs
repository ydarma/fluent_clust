use std::{
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

/// A vertex of a graph.
pub(crate) struct Vertex<Data> {
    node: Rc<RefCell<Node<Data>>>,
}

/// A vertex neighbor. Neighbors are represented as weak pointers to avoid memory leaks.
pub(crate) struct Neighbor<Data> {
    target: Weak<RefCell<Node<Data>>>,
}

/// Vertex internal structure, shared by vertices and neighbors thanks to a smart pointer.
struct Node<Data> {
    data: Data,
    neighbors: Vec<Neighbor<Data>>,
}

/// Target vertices for some vertex neighbors.
type Neighbors<'a, Data> = Vec<&'a Vertex<Data>>;

impl<Data> Vertex<Data> {
    /// Build a new vertex.
    pub(crate) fn new(data: Data) -> Vertex<Data> {
        Vertex {
            node: Rc::new(RefCell::new(Node {
                data,
                neighbors: vec![],
            })),
        }
    }

    pub fn as_neighbor(&self) -> Neighbor<Data> {
        Neighbor {
            target: Rc::downgrade(&self.node),
        }
    }

    /// Get an iterator over the vertices that are neighbor of this vertex.
    pub(crate) fn neighbors(&self) -> impl Iterator<Item = Vertex<Data>> + '_ {
        NeighborIterator::new(Ref::map(self.node.borrow(), |n| &n.neighbors))
    }

    /// Update this vertex neighbors.
    pub(crate) fn set_neighbors(&self, neighbors: Vec<Neighbor<Data>>) {
        self.node.borrow_mut().neighbors = neighbors;
    }

    /// Get a `Ref` to this vertex data.
    pub(crate) fn as_data(&self) -> impl Deref<Target = Data> + '_ {
        Ref::map(self.node.borrow(), |n| &n.data)
    }

    /// Get a `RefMut` to this vertex data.
    pub(crate) fn as_data_mut(&self) -> impl DerefMut<Target = Data> + '_ {
        RefMut::map(self.node.borrow_mut(), |n| &mut n.data)
    }
}

/// Iterator over a reference to a `Vec` of neighbors that returns target vertices
struct NeighborIterator<'a, Data> {
    curr: usize,
    neighbors: Ref<'a, Vec<Neighbor<Data>>>,
}

impl<'a, Data> Iterator for NeighborIterator<'a, Data> {
    type Item = Vertex<Data>;

    /// Returns the next vertex.
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr >= self.neighbors.len() {
            None
        } else {
            let neighbor = self.neighbors[self.curr].target.upgrade();
            self.curr += 1;
            if neighbor.is_none() {
                self.next()
            } else {
                neighbor.map(|n| Vertex { node: n })
            }
        }
    }
}

impl<'a, Data> NeighborIterator<'a, Data> {
    /// Builds a new iterator.
    fn new(neighbors: Ref<'a, Vec<Neighbor<Data>>>) -> Self {
        NeighborIterator { curr: 0, neighbors }
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::*;

    #[test]
    fn test_build_node() {
        let n1 = Vertex::new(1);
        let n2 = Vertex::new(2);
        let n3 = Vertex::new(3);
        n2.set_neighbors(vec![n1.as_neighbor()]);
        n3.set_neighbors(vec![n1.as_neighbor(), n2.as_neighbor()]);
        let mut e3 = n3.neighbors();
        assert_eq!(n1.node.as_ptr(), e3.next().unwrap().node.as_ptr());
        assert_eq!(n2.node.as_ptr(), e3.next().unwrap().node.as_ptr());
    }

    #[test]
    fn test_update_node_neighbors() {
        let n1 = Vertex::new(1);
        let n2 = Vertex::new(2);
        let n3 = Vertex::new(3);
        n2.set_neighbors(vec![n1.as_neighbor()]);
        n3.set_neighbors(vec![n1.as_neighbor(), n2.as_neighbor()]);
        let n1_from_n2 = n2.neighbors().next().unwrap();
        n2.set_neighbors(vec![n1_from_n2.as_neighbor(), n3.as_neighbor()]);
        n1.set_neighbors(vec![n2.as_neighbor(), n3.as_neighbor()]);
        let mut e1 = n1.neighbors();
        assert_eq!(n2.node.as_ptr(), e1.next().unwrap().node.as_ptr());
        assert_eq!(n3.node.as_ptr(), e1.next().unwrap().node.as_ptr());
        let mut e2 = n2.neighbors();
        assert_eq!(n1.node.as_ptr(), e2.next().unwrap().node.as_ptr());
        assert_eq!(n3.node.as_ptr(), e2.next().unwrap().node.as_ptr());
        let mut e3 = n3.neighbors();
        assert_eq!(n1.node.as_ptr(), e3.next().unwrap().node.as_ptr());
        assert_eq!(n2.node.as_ptr(), e3.next().unwrap().node.as_ptr());
    }

    #[test]
    fn test_graph_mutation() {
        let n1 = Vertex::new(1);
        let n2 = Vertex::new(2);
        n2.set_neighbors(vec![n1.as_neighbor()]);
        let n1_from_n2 = n2.neighbors().next().unwrap();
        *n1_from_n2.as_data_mut() = 3;
        assert_eq!(3, *n1.as_data());
    }

    #[test]
    fn test_vertex_suppression() {
        let n1 = Vertex::new(1);
        let n2 = Vertex::new(2);
        let n3 = Vertex::new(3);
        n2.set_neighbors(vec![n1.as_neighbor()]);
        n3.set_neighbors(vec![n1.as_neighbor(), n2.as_neighbor()]);
        let mut graph = vec![n1, n2, n3];
        graph.remove(0);
        let mut e2 = graph[0].neighbors();
        assert!(e2.next().is_none());
        let mut e3 = graph[1].neighbors();
        assert_eq!(graph[0].node.as_ptr(), e3.next().unwrap().node.as_ptr());
        assert!(e3.next().is_none());
    }
}
