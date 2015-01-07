#![feature(quote, plugin_registrar)]
#![feature(phase)]
#![feature(link_args)]

#![allow(dead_code)]

extern crate svd;
#[phase(plugin)] extern crate svd_macros;

// #[phase(plugin)] extern crate foo;

include_svd!("SAMD21", "./tests/samd21.xml");

// static wdt: WDT::Peripheral = WDT::INIT;
static WDT: WDT::Peripheral = WDT::INIT;
static PORT: PORT::Peripheral = PORT::INIT;
static I2CM: SERCOM0::I2CM::Cluster = SERCOM0::I2CM::INIT;
static I2CM5: SERCOM5::I2CM::Cluster = SERCOM5::I2CM::INIT;

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

    I2CM.ADDR.write()
      .ADDR(0xde);

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