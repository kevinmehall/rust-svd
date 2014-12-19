#![feature(quote, plugin_registrar)]
#![feature(phase)]

#[phase(plugin)] extern crate foo;

use std::num::UnsignedInt;
use std::intrinsics::{volatile_load, volatile_store};

struct RegField {
	pub width: uint,
}

impl RegField {
	fn set(&self) -> (uint, uint) {
		(0, (1 << self.width) - 1)
	}

	fn value(&self, value:uint) -> (uint, uint) {
		(0, (1 << self.width) & value)
	}

	fn clear(&self) -> (uint, uint) {
		((1 << self.width) - 1, 0)
	}

	fn update(&self, value:uint) -> (uint, uint) {
		((1 << self.width) - 1, (1 << self.width) & value)
	}
}

foo!(

0x0 => r[1] ENABLE,
0x0 => r[1] DISABLE,

)

// trait Updatable {
// 	fn update(self);
// }

// struct UpdateReg {
//     pub reg: [Updatable]
// }

// impl Drop for UpdateReg {
// 	fn drop(&mut self) {
// 		println!("cool");
// 	}
// }

// impl<T:Updatable> Deref<T> for UpdateReg<T> {
//     fn deref<'a>(&'a self) -> &'a T {
//         &self.reg
//     }
// }

#[deriving(Show)]
#[repr(C)]
struct DMA {
	// pub stage:(uint, uint),
	pub field:u32,
}

impl DMA {
    pub fn update(&'static mut self) -> DMAUpdate {
        DMAUpdate { origin: self, diff: (0, 0) }
    }

    pub fn modify(&mut self, diff: (uint, uint)) {
        let (c, s) = diff;
        if c != 0 {
        	unsafe {
        		let val = volatile_load(&self.field as *const u32);
        		volatile_store(&mut self.field as *mut u32, val & !(c as u32) | (s as u32));
        	}
        } else {
        	unsafe {
        		volatile_store(&mut self.field as *mut u32, s as u32);
        	}
        }
        // println!("self.field {}", self.field);
    }
}

fn or_tuples(l:(uint, uint), r:(uint, uint)) -> (uint, uint) {
	let (la, lb) = l;
	let (ra, rb) = r;
	(la | ra, lb | rb)
}

fn shift_tuple(pos:uint, l:(uint, uint)) -> (uint, uint) {
	let (la, lb) = l;
	(la << pos, lb << pos)
}

#[deriving(Show)]
struct DMAUpdate {
	pub origin:&'static mut DMA,
	pub diff:(uint, uint),
}

impl Drop for DMAUpdate {
	fn drop(&mut self) {
		self.origin.modify(self.diff);
	}
}

impl DMAUpdate {
	fn apply(&mut self, pos:uint, diff:(uint, uint)) -> &mut DMAUpdate {
		self.diff = or_tuples(self.diff, shift_tuple(pos, diff));
		self
	}

	fn set_enable(&mut self) -> &mut DMAUpdate {
		self.apply(0x0, ENABLE.set())
	}

	fn set_disable(&mut self) -> &mut DMAUpdate {
		self.apply(0x1, DISABLE.set())
	}
}


static mut dma:DMA = DMA { field: 0 };

#[no_mangle]
#[inline(never)]
extern "C" fn jeepers() {
	unsafe {
		dma.update().set_enable().set_disable();
		// volatile_store(&mut dma.field as *mut u32, 3);
	}
}

fn main() {
	unsafe {
		jeepers();
		println!("dma {}", dma);
	}
}