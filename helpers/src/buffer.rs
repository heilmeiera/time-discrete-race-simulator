/// RingBuffer provides a buffer with a user-defined capacity. As soon as the capacity is reached,
/// the buffer overwrites old values when new values are pushed to it.
#[derive(Debug)]
pub struct RingBuffer<T> {
    vals: Vec<T>,
    idx: usize,
}

impl<T: Into<f64> + std::marker::Copy> RingBuffer<T> {
    pub fn new(capacity: usize) -> RingBuffer<T> {
        RingBuffer {
            vals: Vec::with_capacity(capacity),
            idx: 0,
        }
    }
    pub fn push(&mut self, val: T) {
        if self.vals.len() < self.vals.capacity() {
            self.vals.push(val);
        } else {
            self.vals[self.idx] = val;
            self.idx = (self.idx + 1) % self.vals.capacity();
        }
    }
    pub fn get_avg(&self) -> Option<f64> {
        if self.vals.is_empty() {
            return None;
        }
        Some(self.get_sum() / self.vals.len() as f64)
    }
    fn get_sum(&self) -> f64 {
        let mut sum = 0.0;
        for val in self.vals.iter() {
            sum += (*val).into()
        }
        sum
    }
}
