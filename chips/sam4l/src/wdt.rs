use core::cell::Cell;
use core::mem;
use kernel::common::volatile_cell::VolatileCell;
use pm::{self, Clock, PBDClock};
use kernel::hil;


#[repr(C, packed)]
pub struct WdtRegisters {
    cr: VolatileCell<u32>,
    clr: VolatileCell<u32>,
    sr: VolatileCell<u32>,
    ier: VolatileCell<u32>,
    idr: VolatileCell<u32>,
    imr: VolatileCell<u32>,
    isr: VolatileCell<u32>,
    icr: VolatileCell<u32>,
}

// Page 59 of SAM4L data sheet
const BASE_ADDRESS: *mut WdtRegisters = 0x400F0C00 as *mut WdtRegisters;

pub struct Wdt {
    registers: *mut WdtRegisters,
    enabled: Cell<bool>,
}

pub static mut WDT: Wdt = Wdt::new(BASE_ADDRESS);

impl Wdt {
    const fn new(base_address: *mut WdtRegisters) -> Wdt {
        Wdt {
            registers: base_address,
            enabled: Cell::new(false),
        }
    }

    fn start(&self) {
        let regs: &mut WdtRegisters = unsafe { mem::transmute(self.registers) };

        self.enabled.set(true);

        unsafe {
            pm::enable_clock(Clock::PBD(PBDClock::WDT));
        }

        let control = (1 << 16) | // Clock enable
                      (15 << 8) | // Set PSEL to 13, 570 ms watchdog period
                      (1 << 7)  | // Flash calibration done (set to default)
                      (1 << 1)  | // Disable after reset
                      (1 << 0);   // Enable

        // Need to write twice for it to work
        regs.cr.set((0x55 << 24) | control);
        regs.cr.set((0xAA << 24) | control);
    }

    fn stop(&self) {
        let regs: &mut WdtRegisters = unsafe { mem::transmute(self.registers) };

        // Set enable bit (bit 0) to 0 to disable
        let control = regs.cr.get() & !0x01;

        // Need to write twice for it to work
        regs.cr.set((0x55 << 24) | control);
        regs.cr.set((0xAA << 24) | control);

        unsafe {
            pm::disable_clock(Clock::PBD(PBDClock::WDT));
        }

        self.enabled.set(false);
    }

    fn tickle(&self) {
        let regs: &mut WdtRegisters = unsafe { mem::transmute(self.registers) };

        // Need to write the WDTCLR bit twice for it to work
        regs.clr.set((0x55 << 24) | (1 << 0));
        regs.clr.set((0xAA << 24) | (1 << 0));
    }
}

impl hil::watchdog::Watchdog for Wdt {
    fn start(&self) {
        self.start();
    }

    fn stop(&self) {
        self.stop();
    }

    fn tickle(&self) {
        self.tickle();
    }
}
