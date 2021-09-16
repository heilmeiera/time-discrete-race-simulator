use std::error::Error;
use std::fmt;

/// InputValueError is used if some simulation option or parameter does not fulfill the posed
/// requirements, e.g., by exceeding the track length.
#[derive(Debug, Clone)]
pub struct InputValueError;

impl fmt::Display for InputValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid input value")
    }
}

impl Error for InputValueError {}

/// argmax returns the index of the maximum value in the array x.
pub fn argmax<T: std::cmp::PartialOrd>(x: &[T]) -> usize {
    let (max_idx, _max_val) =
        x.iter()
            .enumerate()
            .fold((0, &x[0]), |(idx_max, val_max), (idx, val)| {
                if val_max > val {
                    (idx_max, val_max)
                } else {
                    (idx, val)
                }
            });
    max_idx
}

/// max returns the maximum value in the array x.
pub fn max<T: std::cmp::PartialOrd + std::marker::Copy>(x: &[T]) -> T {
    let &max_val = x.iter().fold(
        &x[0],
        |val_max, val| {
            if val_max > val {
                val_max
            } else {
                val
            }
        },
    );
    max_val
}

pub enum SortOrder {
    Ascending,
    Descending,
}

/// argsort returns the indices that sort the array x (unstable sort).
pub fn argsort<T: std::cmp::PartialOrd>(x: &[T], order: SortOrder) -> Vec<usize> {
    let mut idxs: Vec<usize> = (0..x.len()).collect();

    match order {
        SortOrder::Ascending => idxs.sort_unstable_by(|&a, &b| x[a].partial_cmp(&x[b]).unwrap()),
        SortOrder::Descending => idxs.sort_unstable_by(|&a, &b| x[b].partial_cmp(&x[a]).unwrap()),
    };

    idxs
}

/// lin_interp returns the linearly interpolated value at x for given discrete data points xp, fp.
/// xp must be increasing. Inspired by numpy.interp.
pub fn lin_interp(x: f64, xp: &[f64], fp: &[f64]) -> f64 {
    if xp.len() != fp.len() {
        panic!("Number of items in xp and fp must be equal!")
    }

    if x <= xp[0] {
        return fp[0];
    }

    for i in 1..xp.len() {
        if x <= xp[i] {
            return fp[i - 1] + (x - xp[i - 1]) * (fp[i] - fp[i - 1]) / (xp[i] - xp[i - 1]);
        }
    }

    *fp.last().unwrap()
}
