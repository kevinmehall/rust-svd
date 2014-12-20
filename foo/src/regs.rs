#[deriving(Copy)]
pub struct RegField {
    pub width: uint,
}

impl RegField {
    pub fn set(&self) -> (uint, uint) {
        (0, (1 << self.width) - 1)
    }

    pub fn value(&self, value:uint) -> (uint, uint) {
        (0, (1 << self.width) & value)
    }

    pub fn clear(&self) -> (uint, uint) {
        ((1 << self.width) - 1, 0)
    }

    pub fn update(&self, value:uint) -> (uint, uint) {
        ((1 << self.width) - 1, (1 << self.width) & value)
    }
}
