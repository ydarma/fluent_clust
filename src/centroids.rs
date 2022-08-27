use std::mem::swap;

use crate::dist::DistFn;

/// A couple referencing a point and a distance to this point
#[derive(PartialEq, Debug)]
pub(crate) struct PointDist<'a, Point>(&'a Point, f64);

/// The two nearest neighbors when they exist
#[derive(PartialEq, Debug)]
pub(crate) struct Neighborhood<'a, Point>(
    Option<PointDist<'a, Point>>,
    Option<PointDist<'a, Point>>,
);

/// Get the two nearest neighbors, ordered by their distance from the given point.
pub(crate) fn get_neighbors<'a, Iter, Point>(
    iter: Iter,
    point: &Point,
    dist: fn(p1: &Point, p2: &Point) -> f64,
) -> Neighborhood<'a, Point>
where
    Iter: Iterator<Item = &'a Point>,
{
    let iter = iter.map(|p| PointDist(p, dist(&point, p)));
    fold_0(iter)
}

/// find neighbors given a (centroid, distance) couples iterator
fn fold_0<'a, Point>(
    mut iter: impl Iterator<Item = PointDist<'a, Point>>,
) -> Neighborhood<'a, Point> {
    let p1 = iter.next();
    if let Some(d1) = p1 {
        fold_1(d1, iter)
    } else {
        Neighborhood(None, None)
    }
}

/// find the two nearest neighbors when at least one centroid exist.
fn fold_1<'a, Point>(
    first: PointDist<'a, Point>,
    mut others: impl Iterator<Item = PointDist<'a, Point>>,
) -> Neighborhood<'a, Point> {
    let p2 = others.next();
    if let Some(d2) = p2 {
        fold_others_2(first, d2, others)
    } else {
        Neighborhood(Some(first), None)
    }
}

/// find the two nearest neighbors when at least two centroids exist.
fn fold_others_2<'a, Point>(
    mut first: PointDist<'a, Point>,
    mut second: PointDist<'a, Point>,
    others: impl Iterator<Item = PointDist<'a, Point>>,
) -> Neighborhood<'a, Point> {
    if first.1 > second.1 {
        swap(&mut first, &mut second)
    }
    let (d1, d2) = others.fold((first, second), |(d1, d2), d| smallest(d1, d2, d));
    Neighborhood(Some(d1), Some(d2))
}

/// find the two nearest neighbors among three centroids.
fn smallest<'a, Point>(
    mut d1: PointDist<'a, Point>,
    mut d2: PointDist<'a, Point>,
    mut d3: PointDist<'a, Point>,
) -> (PointDist<'a, Point>, PointDist<'a, Point>) {
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
    use crate::centroids::*;
    use crate::dist;

    #[test]
    fn test_neighbors() {
        let centers = vec![vec![1., 1.], vec![3.5, -1.6], vec![2.4, 4.], vec![-0.5, 1.]];
        let nn = get_neighbors(centers.iter(), &vec![0., 0.], dist::euclid_dist);
        assert_eq!(
            Neighborhood(
                Some(PointDist(&centers[3], 1.25)),
                Some(PointDist(&centers[0], 2.))
            ),
            nn
        );
        let nn = get_neighbors(centers.iter(), &vec![1.2, 5.], dist::euclid_dist);
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
        let nn = get_neighbors(centers.iter(), &vec![0., 0.], dist::euclid_dist);
        assert_eq!(Neighborhood(None, None), nn);
    }

    #[test]
    fn test_neighbors_1_centroid() {
        let centers = vec![vec![1., 1.]];
        let nn = get_neighbors(centers.iter(), &vec![0., 0.], dist::euclid_dist);
        assert_eq!(Neighborhood(Some(PointDist(&centers[0], 2.)), None), nn);
    }

    #[test]
    fn test_neighbors_2_centroids() {
        let centers = vec![vec![1., 1.], vec![-0.5, 1.]];
        let nn = get_neighbors(centers.iter(), &vec![0., 0.], dist::euclid_dist);
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
