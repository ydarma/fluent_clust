/// A function that compute a distance between two `Point`
pub(crate) type DistFn<Point> = fn(p1: &Point, p2: &Point) -> f64;

/// Euclidian distance
pub(crate) fn euclid_dist(p1: &Vec<f64>, p2: &Vec<f64>) -> f64 {
    p1.iter()
        .zip(p2)
        .map(|(x1, x2)| {
            let d = x1 - x2;
            d * d
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use crate::dist::*;

    #[test]
    fn test_euclid_dist() {
        let d = euclid_dist(&vec![1., 1.], &vec![0., 0.]);
        assert_eq!(2., d);
        let d = euclid_dist(&vec![1., 3.], &vec![-1., 4.]);
        assert_eq!(5., d);
    }

}
