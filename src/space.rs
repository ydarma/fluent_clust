//! This module defines the necessary functions to run the algorithm for data points that belong to R^n.
//!  - the Euclidian distance function
//!  - the vectorial barycentre function

/// A point in R^n.
pub type RealPoint = Vec<f64>;

/// Conputes Euclidian distance in R^n.
pub fn euclid_dist(p1: &RealPoint, p2: &RealPoint) -> f64 {
    p1.iter()
        .zip(p2)
        .map(|(x1, x2)| {
            let d = x1 - x2;
            d * d
        })
        .sum()
}

/// Computes weighted center in a R^n vector space.
pub fn real_combine(p1: &RealPoint, w1: f64, p2: &RealPoint, w2: f64) -> RealPoint {
    let w = w1 + w2;
    p1.iter()
        .zip(p2)
        .map(|(x1, x2)| (x1 * w1 + x2 * w2) / w)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::space::*;

    #[test]
    fn test_euclid_dist() {
        let d = euclid_dist(&vec![1., 1.], &vec![0., 0.]);
        assert_eq!(2., d);
        let d = euclid_dist(&vec![1., 3.], &vec![-1., 4.]);
        assert_eq!(5., d);
    }

    #[test]
    fn test_real_combine() {
        let c = real_combine(&vec![1., -1.2], 1., &vec![2.5, -0.9], 2.);
        assert_eq!(vec![2., -1.], c);
    }
}
