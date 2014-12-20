#![feature(quote, plugin_registrar)]
#![feature(phase)]

#![allow(dead_code)]

#[phase(plugin)] extern crate foo;
extern crate foo;

foo!(

reg DMA {
    0x0 => r[1] ENABLE,
    0x1 => r[1] DISABLE,
}

reg DMA2 {
    0x0 => r[1] ENABLE2,
    0x1 => r[1] DISABLE2
}

)

static mut dma:DMA::Reg = DMA::INIT;

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