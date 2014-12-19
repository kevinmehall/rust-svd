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

0x0 => r[1] ENABLE;

)

foo!(

0x0 => r[1] DISABLE;

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

#[deriving(Show)]
struct DMAUpdate {
	pub origin:&'static mut DMA,
	pub diff:(uint, uint),
}

impl Drop for DMAUpdate {
	fn drop(&mut self) {
		// println!("UPDATING");
		self.origin.modify(self.diff);
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

trait HAS_ENABLE {
	fn OFFSET_ENABLE(&self) -> uint;
}

trait SET_ENABLE for Sized? {
	fn set_enable(&mut self) -> &mut Self;
}

impl<'a,T> SET_ENABLE for T where T:Applyable+HAS_ENABLE+'a {
	fn set_enable(&mut self) -> &mut T {
		let offset = self.OFFSET_ENABLE();
		self.apply(offset, ENABLE.set())
	}
}

trait HAS_DISABLE {
	fn OFFSET_DISABLE(&self) -> uint;
}

trait SET_DISABLE for Sized? {
	fn set_disable(&mut self) -> &mut Self;
}

impl<'a,T> SET_DISABLE for T where T:Applyable+HAS_DISABLE+'a {
	fn set_disable(&mut self) -> &mut T {
		let offset = self.OFFSET_DISABLE();
		self.apply(offset, DISABLE.set())
	}
}

trait Applyable {
	fn apply(&mut self, pos:uint, diff:(uint, uint)) -> &mut Self;
}

impl Applyable for DMAUpdate {
	fn apply(&mut self, pos:uint, diff:(uint, uint)) -> &mut DMAUpdate {
		self.diff = or_tuples(self.diff, shift_tuple(pos, diff));
		self
	}
}

impl HAS_ENABLE for DMAUpdate {
	fn OFFSET_ENABLE(&self) -> uint { 0 }
}

impl HAS_DISABLE for DMAUpdate {
	fn OFFSET_DISABLE(&self) -> uint { 1 }
}

// impl<'a> Updatable for DMA {
// 	fn update(self) {
// 		println!("DMA {}", self.stage);
// 	}
// }

// impl Drop for DMA {
// 	fn drop(&mut self) {
// 		println!("DONE");
// 	}
// }

// impl DMA {
// 	fn set_enable(&self) -> UpdateReg<DMA> {
// 		let (c, s) = self.stage;
// 		UpdateReg {
// 			reg: DMA {
// 				field: self.field,
// 				stage: (c | 0, s | 0x1),
// 			},
// 		}
// 	}

// 	fn set_disable(&self) -> UpdateReg<DMA> {
// 		let (c, s) = self.stage;
// 		UpdateReg {
// 			reg: DMA {
// 				field: self.field,
// 				stage: (c | 0, s | 0x2),
// 			},
// 		}
// 	}
// }

// fn main() {
//     let x = DMA { field: 0, stage: (0, 0) };

//     let res = x.set_enable().set_disable();

//     println!("{}", res);
// }

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