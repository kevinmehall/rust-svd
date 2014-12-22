#![feature(quote, plugin_registrar)]
#![feature(phase)]
#![feature(link_args)]

#![allow(dead_code)]

#[phase(plugin)] extern crate foo;
extern crate bar;

foo!(

peripheral WDT {
    0x0 => reg8 DMA {
        0 => r[1] ENABLE,
        1 => r[1] DISABLE,
        2 => r[2] WAIT {
            Disable = 0,
            Alarm = 1,
            Watchdog = 2,
            TheyDontLoveYouLikeILoveYou = 3,
        },
    }

    0x1 => reg32 DMA2 {
        0 => r[1] ENABLE2,
        1 => r[1] DISABLE2
    }
}

/// Watchdog Timer
peripheral WDT2 {
  /// Control
  0x0 => reg8 CTRL {
    /// Enable
    1 => rw[1] ENABLE,
    /// Watchdog Timer Window Mode Enable
    2 => rw[1] WEN,
    /// Always-On
    7 => rw[1] ALWAYSON,
  }

  /// Configuration
  0x1 => reg8 CONFIG {
    /// Time-Out Period
    0 => rw[4] PER {
      /// 8 clock cycles
      USE_8_CLOCK_CYCLES = 0x0,
      /// 16 clock cycles
      USE_16_CLOCK_CYCLES = 0x1,
      /// 32 clock cycles
      USE_32_CLOCK_CYCLES = 0x2,
      /// 64 clock cycles
      USE_64_CLOCK_CYCLES = 0x3,
      /// 128 clock cycles
      USE_128_CLOCK_CYCLES = 0x4,
      /// 256 clock cycles
      USE_256_CLOCK_CYCLES = 0x5,
      /// 512 clock cycles
      USE_512_CLOCK_CYCLES = 0x6,
      /// 1024 clock cycles
      USE_1024_CLOCK_CYCLES = 0x7,
      /// 2048 clock cycles
      USE_2048_CLOCK_CYCLES = 0x8,
      /// 4096 clock cycles
      USE_4096_CLOCK_CYCLES = 0x9,
      /// 8192 clock cycles
      USE_8192_CLOCK_CYCLES = 0xa,
      /// 16384 clock cycles
      USE_16384_CLOCK_CYCLES = 0xb,
    },
    /// Window Mode Time-Out Period
    4 => rw[4] WINDOW {
      /// 8 clock cycles
      USE_8_CLOCK_CYCLES = 0x0,
      /// 16 clock cycles
      USE_16_CLOCK_CYCLES = 0x1,
      /// 32 clock cycles
      USE_32_CLOCK_CYCLES = 0x2,
      /// 64 clock cycles
      USE_64_CLOCK_CYCLES = 0x3,
      /// 128 clock cycles
      USE_128_CLOCK_CYCLES = 0x4,
      /// 256 clock cycles
      USE_256_CLOCK_CYCLES = 0x5,
      /// 512 clock cycles
      USE_512_CLOCK_CYCLES = 0x6,
      /// 1024 clock cycles
      USE_1024_CLOCK_CYCLES = 0x7,
      /// 2048 clock cycles
      USE_2048_CLOCK_CYCLES = 0x8,
      /// 4096 clock cycles
      USE_4096_CLOCK_CYCLES = 0x9,
      /// 8192 clock cycles
      USE_8192_CLOCK_CYCLES = 0xa,
      /// 16384 clock cycles
      USE_16384_CLOCK_CYCLES = 0xb,
    },
  }

  /// Early Warning Interrupt Control
  0x2 => reg8 EWCTRL {
    /// Early Warning Interrupt Time Offset
    0 => rw[4] EWOFFSET {
      /// 8 clock cycles
      USE_8_CLOCK_CYCLES = 0x0,
      /// 16 clock cycles
      USE_16_CLOCK_CYCLES = 0x1,
      /// 32 clock cycles
      USE_32_CLOCK_CYCLES = 0x2,
      /// 64 clock cycles
      USE_64_CLOCK_CYCLES = 0x3,
      /// 128 clock cycles
      USE_128_CLOCK_CYCLES = 0x4,
      /// 256 clock cycles
      USE_256_CLOCK_CYCLES = 0x5,
      /// 512 clock cycles
      USE_512_CLOCK_CYCLES = 0x6,
      /// 1024 clock cycles
      USE_1024_CLOCK_CYCLES = 0x7,
      /// 2048 clock cycles
      USE_2048_CLOCK_CYCLES = 0x8,
      /// 4096 clock cycles
      USE_4096_CLOCK_CYCLES = 0x9,
      /// 8192 clock cycles
      USE_8192_CLOCK_CYCLES = 0xa,
      /// 16384 clock cycles
      USE_16384_CLOCK_CYCLES = 0xb,
    },
  }

  /// Interrupt Enable Clear
  0x4 => reg8 INTENCLR {
    /// Early Warning Interrupt Enable
    0 => rw[1] EW,
  }

  /// Interrupt Enable Set
  0x5 => reg8 INTENSET {
    /// Early Warning Interrupt Enable
    0 => rw[1] EW,
  }

  /// Interrupt Flag Status and Clear
  0x6 => reg8 INTFLAG {
    /// Early Warning
    0 => rw[1] EW,
  }

  /// Status
  0x7 => reg8 STATUS {
    /// Synchronization Busy
    7 => r[1] SYNCBUSY,
  }

  /// Clear
  0x8 => reg8 CLEAR {
    /// Watchdog Clear
    0 => w[8] CLEAR {
      /// Clear Key
      KEY = 0xa5,
    },
  }

}

)

static mut wdt: WDT::Peripheral = WDT::INIT;
static mut wdt2: WDT2::Peripheral = WDT2::INIT;


#[test]
fn wdt_changes() {
    unsafe {
        wdt2.CONFIG.write()
            .WINDOW(WDT2::CONFIG::WINDOW::USE_16384_CLOCK_CYCLES)
            .PER(WDT2::CONFIG::PER::USE_16384_CLOCK_CYCLES);
        wdt2.CONFIG.update()
            .PER(WDT2::CONFIG::PER::USE_16384_CLOCK_CYCLES);
        assert_eq!(wdt2.CONFIG.field, 187);
    }
}

#[no_mangle]
#[inline(never)]
extern "C" fn jeepers() {
    unsafe {
        // wdt.DMA.write().ENABLE();
        // wdt.DMA.write().ENABLE().DISABLE();
        // wdt.DMA.update().WAIT(WDT::DMA::WAIT::Alarm).ENABLE(0);
        // volatile_store(&mut dma.field as *mut u32, 3);

        wdt2.CONFIG.write()
            .WINDOW(WDT2::CONFIG::WINDOW::USE_16384_CLOCK_CYCLES)
            .PER(WDT2::CONFIG::PER::USE_16384_CLOCK_CYCLES);
    }
}

fn main() {
    unsafe {
        jeepers();
        println!("wdt {}", wdt);
        println!("wdt2 {}", wdt2);
    }
}