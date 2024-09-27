//! Simple finite impulse resonse filter.
pub struct FirFilter<const N: usize> {
    ring_fifo: [i32; N],
    offset: usize,
    sum: i32,
}

impl<const N: usize> FirFilter<N> {
    pub const fn new() -> FirFilter<N> {
        FirFilter{
            ring_fifo: [0; N],
            offset: 0,
            sum: 0,
        }
    }
    pub fn update(&mut self, value: i32) {
        let old_value = self.ring_fifo[self.offset];
        self.ring_fifo[self.offset] = value;
        self.offset = (self.offset + 1) % N;
        self.sum = self.sum + value - old_value;
    }
    #[allow(dead_code)]
    pub fn filtered_value(&self) -> i32 {
        (self.sum + (((N as i32) / 2) - 1)) / (N as i32)
    }
}
