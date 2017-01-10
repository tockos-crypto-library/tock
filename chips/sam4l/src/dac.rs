/// Implementation of DAC driver for SAM4L chip
///
/// Author: Yifan Hao <haoyifan@umich.edu>
///
/// Under development...
///
/// TODO:
/// 1. The manual says there are 4 FIFO buffer, but actually it can only hold 2 10 bits conversion
/// data. The wave form also shows it can hold 4 data (but each is one single byte). Isn't that
/// contradicting with itself? (Its resolution is 10 bits but manual is using 4 8 bits data...)
/// 2. How to interact with the data from APB? Simply store in the data register? through syscal?
/// 3. Need to reason about what happens when interrupt fires.

use core::cell::Cell;
use core::mem;
use kernel::common::volatile_cell::VolatileCell;
use kernel::hil;

use nvic;
use pm::{self, Clock, PBAClock};
use scif;

#[repr(C, packed)]
pub struct DacRegisters {
    // From page 905 of SAM4L manual
    cr: VolatileCell<u32>, // Control                   (0x00)
    mr: VolatileCell<u32>, // Mode                      (0x04)
    cdr: VolatileCell<u32>, //Conversion Data           (0x08)
    ier: VolatileCell<u32>, // Interrupt Enable         (0x0C)
    idr: VolatileCell<u32>, // Interrupt Disable        (0x10)
    imr: VolatileCell<u32>, // Interrupt Mask           (0x14)
    isr: VolatileCell<u32>, // Interrupt Status         (0x18)
    _unused: [u32; 50], // 0x1C - 0xE0
    wpmr: VolatileCell<u32>, // Write Protect Mode      (0xE4)
    wpsr: VolatileCell<u32>, // Write Protect Status    (0xE8)
    version: VolatileCell<u32>, // Version              (0xFC)
}


// page 59 of SAM4L data sheet
const BASE_ADDRESS: *mut DacRegisters = 0x4003C000 as *mut DacRegisters;

pub struct Dac {
    registers: *mut DacRegisters,
    enabled: Cell<bool>,
    // FIXME: needs to figure out if Dac needs other variables
}

pub static mut DAC: Dac = Dac::new(BASE_ADDRESS);

impl Dac {
    // Creates a new DAC object
    const fn new(base_address: *mut DacRegisters) -> Dac {
        Dac {
            registers: base_address,
            enabled: Cell::new(false),
        }
    }


    fn enable(&self) {
        let regs: &mut DacRegisters = unsafe { mem::transmute(self.registers) };
        if !self.enabled.get() {
            self.enabled.set(true);

            // Start the clock
            unsafe {
                pm::enable_clock(pm::Clock::PBA(pm::PBAClock::DACC));
                nvic::enable(nvic::NvicIdx::DACC);
            }

            // FIXME: do we need to write 1 to control register (CR) to do
            // software reset? when?

            let mut mr: u32 = regs.mr.get();
            let mut wpmr: u32 = regs.wpmr.get();

            // write to mode register to enable the DAC
            // This code changes DACEN in MR to 1
            mr = mr | (1 << 4);

            // configure startup time
            // This code changes STARTUP in MR to 0
            mr = mr | (0 << 8);

            // choose the trigger source
            // This code changes TRGSEL in MR to 0b001
            // choose peripheral event. NOTE: should depend on argument
            mr = mr | (1 << 1);

            // configure the transfer size
            // This code changes WORD in MR to 0
            // choose half word transfer. NOTE: should depend on argument
            mr = mr | (0 << 5);

            // configure the trigger enable
            // This code changes TRGEN in MR to 0 because we use peripheral event
            mr = mr | (0 << 0);

            // write to mode register
            regs.mr.set(mr);

            // write to the Write Protect Mode Register
            wpmr = wpmr | (0x444143 << 8) | 1;
            regs.wpmr.set(wpmr);

            // reset TXRDY bit in interrupt register

        }
    }

    pub fn handle_interrupt(&mut self){}
}

interrupt_handler!(dacc_handler, DACC);
