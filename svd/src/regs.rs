#[derive(Copy)]
pub struct RegField {
    pub width: uint,
}

#[inline]
fn spread(width:uint) -> uint {
    (1 << width) - 1
}

impl RegField {
    pub fn set(&self) -> (uint, uint) {
        (0, spread(self.width))
    }

    pub fn update(&self) -> (uint, uint) {
        (spread(self.width), spread(self.width))
    }

    pub fn set_value(&self, value:uint) -> (uint, uint) {
        (0, spread(self.width) & value)
    }

    pub fn update_value(&self, value:uint) -> (uint, uint) {
        (spread(self.width), spread(self.width) & value)
    }

    pub fn read(&self, value:uint) -> uint {
        spread(self.width) & value
    }
}
