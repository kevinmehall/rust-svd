#![feature(quote, plugin_registrar)]
#![feature(phase)]
#![feature(link_args)]

#![allow(dead_code)]

#[phase(plugin)] extern crate foo;
extern crate bar;

#[phase(plugin)] extern crate svd;

include_svd!("samd21", "svd/file.xml");


// static wdt: WDT::Peripheral = WDT::INIT;
static WDT: WDT::Peripheral = WDT::INIT;


#[test]
fn wdt_changes() {
      WDT.CONFIG.write()
          .WINDOW(WDT::CONFIG::WINDOW::USE_16384_CLOCK_CYCLES)
          .PER(WDT::CONFIG::PER::USE_16384_CLOCK_CYCLES);
      WDT.CONFIG.update()
          .PER(WDT::CONFIG::PER::USE_16384_CLOCK_CYCLES);

      assert!(WDT.CONFIG.read().PER().unwrap() == WDT::CONFIG::PER::USE_16384_CLOCK_CYCLES);
      unsafe {
        assert_eq!(*WDT.CONFIG.field.get(), 187);
      }
}

#[no_mangle]
#[inline(never)]
extern "C" fn jeepers() {
    // wdt.DMA.write().ENABLE();
    // wdt.DMA.write().ENABLE().DISABLE();
    // wdt.DMA.update().WAIT(WDT::DMA::WAIT::Alarm).ENABLE(0);
    // volatile_store(&mut dma.field as *mut u32, 3);

    WDT.CONFIG.write()
        .WINDOW(WDT::CONFIG::WINDOW::USE_16384_CLOCK_CYCLES)
        .PER(WDT::CONFIG::PER::USE_16384_CLOCK_CYCLES);
}

fn main() {
    jeepers();
    // println!("wdt {}", wdt);
    // println!("WDT {}", WDT);
}