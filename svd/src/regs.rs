//#[derive(Copy)]
pub struct RegField {
    pub width: usize,
}

#[inline]
fn spread(width:usize) -> usize {
    (1 << width) - 1
}

impl RegField {
    pub fn set(&self) -> (usize, usize) {
        (0, spread(self.width))
    }

    pub fn update(&self) -> (usize, usize) {
        (spread(self.width), spread(self.width))
    }

    pub fn set_value(&self, value:usize) -> (usize, usize) {
        (0, spread(self.width) & value)
    }

    pub fn update_value(&self, value:usize) -> (usize, usize) {
        (spread(self.width), spread(self.width) & value)
    }

    pub fn read(&self, value:usize) -> usize {
        spread(self.width) & value
    }
}

//impl Clone for RegField {
//}
