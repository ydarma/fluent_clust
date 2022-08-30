use std::{mem::swap, ops::Deref};

/// A reference to a `Point` and its distance to some other `Point`
#[derive(PartialEq, Debug)]
pub(crate) struct PointDist<Point, RefPoint>(RefPoint, f64)
where
    RefPoint: Deref<Target = Point>;

impl<Point, RefPoint> PointDist<Point, RefPoint>
where
    RefPoint: Deref<Target = Point>,
{
    /// The point refenrence
    pub fn coord(&self) -> &Point {
        &self.0
    }

    /// The distance to some other `Point`
    pub fn dist(&self) -> f64 {
        self.1
    }
}

/// The two nearest neighbors when they exist
#[derive(PartialEq, Debug)]
pub(crate) struct Neighborhood<Point, RefPoint>(
    pub Option<PointDist<Point, RefPoint>>,
    pub Option<PointDist<Point, RefPoint>>,
)
where
    RefPoint: Deref<Target = Point>;

/// Defines a two nearest neighbors getter function.
///
/// This trait is implemented by stucts that represents a set of centroids in a space of `Point`.
pub(crate) trait GetNeighborhood<Point, Model, RefModel, Dist>
where
    Dist: Fn(&Point, &Model) -> f64,
    RefModel: Deref<Target = Model>,
{
    /// Get the two nearest neighbors, ordered by their distance from the given point.
    fn get_neighborhood(&mut self, point: &Point, dist: Dist) -> Neighborhood<Model, RefModel>;
}

/// Implementation of two nearest neighbors getter for a `Vec<Point>` that represents a set of centroids.
impl<Iter, Point, Model, RefModel, Dist> GetNeighborhood<Point, Model, RefModel, Dist> for Iter
where
    Iter: Iterator<Item = RefModel>,
    Dist: Fn(&Point, &Model) -> f64,
    RefModel: Deref<Target = Model>,
{
    fn get_neighborhood(&mut self, point: &Point, dist: Dist) -> Neighborhood<Model, RefModel> {
        let iter = self.map(|p| {
            let dist = dist(&point, &p);
            PointDist(p, dist)
        });
        fold_0(iter)
    }
}

/// find neighbors given a (centroid, distance) couples iterator
fn fold_0<Point, RefPoint>(
    mut iter: impl Iterator<Item = PointDist<Point, RefPoint>>,
) -> Neighborhood<Point, RefPoint>
where
    RefPoint: Deref<Target = Point>,
{
    let p1 = iter.next();
    if let Some(d1) = p1 {
        fold_1(d1, iter)
    } else {
        Neighborhood(None, None)
    }
}

/// find the two nearest neighbors when at least one centroid exist.
fn fold_1<Point, RefPoint>(
    first: PointDist<Point, RefPoint>,
    mut others: impl Iterator<Item = PointDist<Point, RefPoint>>,
) -> Neighborhood<Point, RefPoint>
where
    RefPoint: Deref<Target = Point>,
{
    let p2 = others.next();
    if let Some(d2) = p2 {
        fold_others_2(first, d2, others)
    } else {
        Neighborhood(Some(first), None)
    }
}

/// find the two nearest neighbors when at least two centroids exist.
fn fold_others_2<Point, RefPoint>(
    mut first: PointDist<Point, RefPoint>,
    mut second: PointDist<Point, RefPoint>,
    others: impl Iterator<Item = PointDist<Point, RefPoint>>,
) -> Neighborhood<Point, RefPoint>
where
    RefPoint: Deref<Target = Point>,
{
    if first.1 > second.1 {
        swap(&mut first, &mut second)
    }
    let (d1, d2) = others.fold((first, second), |(d1, d2), d| smallest(d1, d2, d));
    Neighborhood(Some(d1), Some(d2))
}

/// find the two nearest neighbors among three centroids.
fn smallest<Point, RefPoint>(
    mut d1: PointDist<Point, RefPoint>,
    mut d2: PointDist<Point, RefPoint>,
    mut d3: PointDist<Point, RefPoint>,
) -> (PointDist<Point, RefPoint>, PointDist<Point, RefPoint>)
where
    RefPoint: Deref<Target = Point>,
{
    if d1.1 > d2.1 {
        swap(&mut d1, &mut d2);
    }
    if d2.1 > d3.1 {
        swap(&mut d2, &mut d3);
    }
    if d1.1 > d2.1 {
        swap(&mut d1, &mut d2);
    }
    (d1, d2)
}

#[cfg(test)]
mod tests {
    use crate::neighbors::*;
    use crate::space;

    #[test]
    fn test_point_dist() {
        let point = vec![0., 0.];
        let p = PointDist(&point, 2.4);
        assert_eq!(&point, p.coord());
        assert_eq!(2.4, p.dist());
    }

    #[test]
    fn test_neighbors() {
        let centers = vec![vec![1., 1.], vec![3.5, -1.6], vec![2.4, 4.], vec![-0.5, 1.]];
        let point = &vec![0., 0.];
        let nn = centers
            .iter()
            .get_neighborhood(point, Box::new(space::euclid_dist));
        assert_eq!(
            Neighborhood(
                Some(PointDist(&centers[3], 1.25)),
                Some(PointDist(&centers[0], 2.))
            ),
            nn
        );
        let point = &vec![1.2, 5.];
        let nn = centers
            .iter()
            .get_neighborhood(point, Box::new(space::euclid_dist));
        assert_eq!(
            Neighborhood(
                Some(PointDist(&centers[2], 2.44)),
                Some(PointDist(&centers[0], 16.04))
            ),
            nn
        );
    }

    #[test]
    fn test_neighbors_0_centroid() {
        let centers = vec![];
        let point = &vec![0., 0.];
        let nn = centers
            .iter()
            .get_neighborhood(point, Box::new(space::euclid_dist));
        assert_eq!(Neighborhood(None, None), nn);
    }

    #[test]
    fn test_neighbors_1_centroid() {
        let centers = vec![vec![1., 1.]];
        let point = &vec![0., 0.];
        let nn = centers
            .iter()
            .get_neighborhood(point, Box::new(space::euclid_dist));
        assert_eq!(Neighborhood(Some(PointDist(&centers[0], 2.)), None), nn);
    }

    #[test]
    fn test_neighbors_2_centroids() {
        let centers = vec![vec![1., 1.], vec![-0.5, 1.]];
        let point = &vec![0., 0.];
        let nn = centers
            .iter()
            .get_neighborhood(point, Box::new(space::euclid_dist));
        assert_eq!(
            Neighborhood(
                Some(PointDist(&centers[1], 1.25)),
                Some(PointDist(&centers[0], 2.))
            ),
            nn
        );
    }

    #[test]
    fn test_smallest() {
        let p: Vec<f64> = vec![];
        let d1 = PointDist(&p, 7.);
        let d2 = PointDist(&p, 4.);
        let d3 = PointDist(&p, 1.);
        let s = smallest(d1, d2, d3);
        assert_eq!((PointDist(&p, 1.), PointDist(&p, 4.)), s);
        let d1 = PointDist(&p, 7.);
        let d2 = PointDist(&p, 4.);
        let d3 = PointDist(&p, 5.);
        let s = smallest(d1, d2, d3);
        assert_eq!((PointDist(&p, 4.), PointDist(&p, 5.)), s);
        let d1 = PointDist(&p, 7.);
        let d2 = PointDist(&p, 4.);
        let d3 = PointDist(&p, 8.);
        let s = smallest(d1, d2, d3);
        assert_eq!((PointDist(&p, 4.), PointDist(&p, 7.)), s);
    }
}
