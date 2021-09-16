pub mod buffer;
pub mod general;
pub mod geometry;

#[cfg(test)]
mod buffer_tests {
    use crate::buffer::RingBuffer;
    use approx::assert_ulps_eq;

    #[test]
    fn test_ringbuffer_1() {
        let x: RingBuffer<i32> = RingBuffer::new(5);
        assert!(x.get_avg().is_none());
    }
    #[test]
    fn test_ringbuffer_2() {
        let mut x: RingBuffer<i32> = RingBuffer::new(5);
        x.push(3);
        x.push(4);
        assert_ulps_eq!(x.get_avg().unwrap(), 3.5);
    }
    #[test]
    fn test_ringbuffer_3() {
        let mut x: RingBuffer<i32> = RingBuffer::new(5);
        x.push(3);
        x.push(4);
        x.push(2);
        x.push(1);
        x.push(5);
        x.push(10);
        assert_ulps_eq!(x.get_avg().unwrap(), 4.4);
    }
}

#[cfg(test)]
mod general_tests {
    use crate::general::{argmax, argsort, lin_interp, max, SortOrder};
    use approx::assert_ulps_eq;

    #[test]
    fn test_argmax_1() {
        let x: Vec<i32> = vec![3, -1, 5, 8, -2];
        assert_eq!(argmax(&x), 3);
    }
    #[test]
    fn test_argmax_2() {
        let x: Vec<f64> = vec![3.0, -1.0, 5.0, 8.0, -2.0];
        assert_eq!(argmax(&x), 3);
    }

    #[test]
    fn test_max_1() {
        let x: Vec<i32> = vec![3, -1, 5, 8, -2];
        assert_eq!(max(&x), 8);
    }
    #[test]
    fn test_max_2() {
        let x: Vec<f64> = vec![3.0, -1.0, 5.0, 8.0, -2.0];
        assert_ulps_eq!(max(&x), 8.0);
    }

    #[test]
    fn test_argsort_1() {
        let x: Vec<i32> = vec![3, -1, 5, 8, -2];
        assert_eq!(argsort(&x, SortOrder::Ascending), vec![4, 1, 0, 2, 3]);
    }
    #[test]
    fn test_argsort_2() {
        let x: Vec<i32> = vec![3, -1, 5, 8, -2];
        assert_eq!(argsort(&x, SortOrder::Descending), vec![3, 2, 0, 1, 4]);
    }
    #[test]
    fn test_argsort_3() {
        let x: Vec<f64> = vec![3.0, -1.0, 5.0, 8.0, -2.0];
        assert_eq!(argsort(&x, SortOrder::Ascending), vec![4, 1, 0, 2, 3]);
    }
    #[test]
    fn test_argsort_4() {
        let x: Vec<f64> = vec![3.0, -1.0, 5.0, 8.0, -2.0];
        assert_eq!(argsort(&x, SortOrder::Descending), vec![3, 2, 0, 1, 4]);
    }

    #[test]
    fn test_lin_interp_1() {
        let xp: Vec<f64> = vec![-5.0, 0.0, 5.0, 10.0];
        let fp: Vec<f64> = vec![1.0, 2.0, 1.0, 0.0];
        assert_ulps_eq!(lin_interp(-2.5, &xp, &fp), 1.5);
    }
    #[test]
    fn test_lin_interp_2() {
        let xp: Vec<f64> = vec![-5.0, 0.0, 5.0, 10.0];
        let fp: Vec<f64> = vec![1.0, 2.0, 1.0, 0.0];
        assert_ulps_eq!(lin_interp(7.5, &xp, &fp), 0.5);
    }
    #[test]
    fn test_lin_interp_3() {
        let xp: Vec<f64> = vec![-5.0, 0.0, 5.0, 10.0];
        let fp: Vec<f64> = vec![-1.0, -2.0, -1.0, 0.0];
        assert_ulps_eq!(lin_interp(7.5, &xp, &fp), -0.5);
    }
}

#[cfg(test)]
mod geometry_tests {
    use crate::geometry::Vector2d;
    use approx::assert_ulps_eq;

    #[test]
    fn test_vector2d_sub() {
        let v1: Vector2d = Vector2d { dx: 5.0, dy: 5.0 };
        let v2: Vector2d = Vector2d { dx: 2.0, dy: -1.0 };
        assert_eq!(v1.sub(&v2), Vector2d { dx: 3.0, dy: 6.0 });
    }
    #[test]
    fn test_vector2d_add() {
        let v1: Vector2d = Vector2d { dx: 5.0, dy: 5.0 };
        let v2: Vector2d = Vector2d { dx: 2.0, dy: -1.0 };
        assert_eq!(v1.add(&v2), Vector2d { dx: 7.0, dy: 4.0 });
    }
    #[test]
    fn test_vector2d_mult() {
        let v1: Vector2d = Vector2d { dx: 5.0, dy: 5.0 };
        assert_eq!(v1.mult(3.0), Vector2d { dx: 15.0, dy: 15.0 });
    }
    #[test]
    fn test_vector2d_cross() {
        let v1: Vector2d = Vector2d { dx: 5.0, dy: 5.0 };
        let v2: Vector2d = Vector2d { dx: 2.0, dy: -1.0 };
        assert_ulps_eq!(v1.cross(&v2), -15.0);
    }
    #[test]
    fn test_vector2d_abs() {
        let v1: Vector2d = Vector2d { dx: 5.0, dy: 5.0 };
        assert_ulps_eq!(v1.abs(), 50.0_f64.sqrt());
    }
    #[test]
    fn test_vector2d_normal_vector() {
        let v1: Vector2d = Vector2d { dx: 5.0, dy: 5.0 };
        assert_eq!(v1.normal_vector(), Vector2d { dx: -5.0, dy: 5.0 });
    }
    #[test]
    fn test_vector2d_normalized() {
        let v1: Vector2d = Vector2d { dx: 5.0, dy: 5.0 };
        assert_eq!(
            v1.normalized(),
            Vector2d {
                dx: 5.0 / 50.0_f64.sqrt(),
                dy: 5.0 / 50.0_f64.sqrt()
            }
        );
    }
}
