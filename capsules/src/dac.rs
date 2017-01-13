use core::cell::Cell;
use kernel::{AppId, Driver};
use kernel::hil::dac::DacSingle;

pub struct DAC<'a, A: DacSingle + 'a> {
    dac: &'a A,
}

impl<'a, A: DacSingle + 'a> DAC<'a, A> {
    pub fn new(dac: &'a A) -> DAC<`a, A> {
        DAC{
            dac: dac,
        }
    }

    fn enable(&self) {
        self.dac.enable();
    }

    fn set(&self, data: u16){
        self.dac.set(data);
    }
}

impl<'a, A: DacSIngle + 'a> Driver for DAC<'a, A> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            // default
            // shouldn't call subscribe
            _ => -1,
        }
    }

    fn command(&self, command_num: usize, data: usize, _: AppId) -> isize {
        match command_num {
            0 /* check if present */ => 0

            // enable the dac
            1 => {
                self.enable();
                return 0;
            }

            // set the data on CDR
            2 => {
                if !self.set(data as u16) {
                    return -1;
                }
                return 0;
            }

            _ => -1,
        }
    }
}
