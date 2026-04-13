pub struct Bitmap<const SIZE: usize> {
    inner: [u64; SIZE],
}

impl<const SIZE: usize> Bitmap<SIZE> {
    pub const fn new() -> Self {
        Self { inner: [0; SIZE] }
    }

    pub const fn with_inner(inner: [u64; SIZE]) -> Self {
        Self { inner }
    }

    pub const fn all() -> Self {
        Self {
            inner: [u64::MAX; SIZE],
        }
    }

    pub fn set_all(&mut self) {
        self.inner = [u64::MAX; SIZE];
    }

    pub fn get(&self, bit: usize) -> bool {
        let index = bit / 64;
        let offset = bit % 64;
        (self.inner[index] & (1_u64 << offset)) != 0
    }

    pub fn set(&mut self, bit: usize, val: bool) {
        let index = bit / 64;
        let offset = bit % 64;
        if val {
            self.inner[index] |= 1_u64 << offset;
        } else {
            self.inner[index] &= !(1_u64 << offset);
        }
    }

    pub fn first_zero(&self) -> usize {
        for (i, &block) in self.inner.iter().enumerate() {
            if block != u64::MAX {
                return i * 64 + (!block).trailing_zeros() as usize;
            }
        }
        SIZE * 64
    }

    pub fn first_one(&self) -> usize {
        for (i, &block) in self.inner.iter().enumerate() {
            if block != 0 {
                return i * 64 + block.trailing_zeros() as usize;
            }
        }
        SIZE * 64
    }

    pub fn set_bits(&mut self, start: usize, count: usize, val: bool) {
        for i in 0..count {
            self.set(start + i, val);
        }
    }

    pub fn all_set(&self) -> bool {
        self.inner.iter().all(|&block| block == u64::MAX)
    }

    pub fn all_clear(&self) -> bool {
        self.inner.iter().all(|&block| block == 0)
    }
}

pub struct BitmapRef<'a> {
    inner: &'a mut [u64],
}

impl<'a> BitmapRef<'a> {
    pub const fn new(inner: &'a mut [u64]) -> Self {
        Self { inner }
    }

    pub fn get(&self, bit: usize) -> bool {
        let index = bit / 64;
        let offset = bit % 64;
        (self.inner[index] & (1_u64 << offset)) != 0
    }

    pub fn set(&mut self, bit: usize, val: bool) {
        let index = bit / 64;
        let offset = bit % 64;
        if val {
            self.inner[index] |= 1_u64 << offset;
        } else {
            self.inner[index] &= !(1_u64 << offset);
        }
    }

    pub fn first_zero(&self) -> usize {
        for (i, &block) in self.inner.iter().enumerate() {
            if block != u64::MAX {
                return i * 64 + (!block).trailing_zeros() as usize;
            }
        }
        self.inner.len() * 64
    }

    pub fn first_one(&self) -> usize {
        for (i, &block) in self.inner.iter().enumerate() {
            if block != 0 {
                return i * 64 + block.trailing_zeros() as usize;
            }
        }
        self.inner.len() * 64
    }

    pub fn set_bits(&mut self, start: usize, count: usize, val: bool) {
        for i in 0..count {
            self.set(start + i, val);
        }
    }
}
