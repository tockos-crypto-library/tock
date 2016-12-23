//! Driver for the FXOS8700CQ accelerometer
//! http://www.nxp.com/assets/documents/data/en/data-sheets/FXOS8700CQ.pdf
//! The driver provides x, y, and z acceleration data to a callback function.
//! To use readings from the sensor in userland, see FXOS8700CQ.h in libtock.

use core::cell::Cell;
use kernel::{AppId, Callback, Container, Driver};
use kernel::common::take_cell::TakeCell;
use kernel::hil::i2c::{I2CDevice, I2CClient, Error};

pub static mut BUF: [u8; 6] = [0; 6];

#[allow(dead_code)]
enum Registers {
    SensorStatus = 0x00,
    OutXMSB = 0x01,
    OutXLSB = 0x02,
    OutYMSB = 0x03,
    OutYLSB = 0x04,
    OutZMSB = 0x05,
    OutZLSB = 0x06,
    XyzDataCfg = 0x0E,
    WhoAmI = 0x0D,
    CtrlReg1 = 0x2A,
}

#[derive(Clone,Copy,PartialEq)]
enum State {
    /// Sensor does not take acceleration readings
    Disabled,

    /// Verifying that sensor is present
    Enabling,

    /// Activate sensor to take readings
    Activating,

    /// Reading accelerometer data
    ReadingAcceleration,

    /// Deactivate sensor
    Deactivating(i16, i16, i16),
}

pub struct App {
    callback: Option<Callback>,
    pending_command: bool,
    command: usize,
}

impl Default for App {
    fn default() -> App {
        App {
            callback: None,
            pending_command: false,
            command: 0,
        }
    }
}

pub struct Fxos8700cq<'a> {
    i2c: &'a I2CDevice,
    state: Cell<State>,
    buffer: TakeCell<&'static mut [u8]>,
    apps: Container<App>,
    current_app: Cell<Option<AppId>>,
}

impl<'a> Fxos8700cq<'a> {
    pub fn new(i2c: &'a I2CDevice, buffer: &'static mut [u8], container: Container<App>) -> Fxos8700cq<'a> {
        Fxos8700cq {
            i2c: i2c,
            state: Cell::new(State::Enabling),
            buffer: TakeCell::new(buffer),
            apps: container,
            current_app: Cell::new(None),
        }
    }

    fn start_read_accel(&self) {
        self.buffer.take().map_or_else(|| {
            panic!("no buffs");
        }, |buf| {
            self.i2c.enable();
            buf[0] = Registers::WhoAmI as u8;
            self.i2c.write_read(buf, 1, 1);
            self.state.set(State::Enabling);
        });
    }
}

impl<'a> I2CClient for Fxos8700cq<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: Error) {
        match self.state.get() {
            State::Disabled => {
                // self.i2c.disable();
            }
            State::Enabling => {
                buffer[0] = Registers::CtrlReg1 as u8; // CTRL_REG1
                buffer[1] = 1; // active
                self.i2c.write(buffer, 2);
                self.state.set(State::Activating);
            }
            State::Activating => {
                buffer[0] = Registers::OutXMSB as u8;
                self.i2c.write_read(buffer, 1, 6); // read 6 accel registers for xyz
                self.state.set(State::ReadingAcceleration);
            }
            State::ReadingAcceleration => {
                let x = (((buffer[0] as i16) << 8) | buffer[1] as i16) >> 2;
                let y = (((buffer[2] as i16) << 8) | buffer[3] as i16) >> 2;
                let z = (((buffer[4] as i16) << 8) | buffer[5] as i16) >> 2;

                let x = ((x as isize) * 244) / 1000;
                let y = ((y as isize) * 244) / 1000;
                let z = ((z as isize) * 244) / 1000;

                buffer[0] = 0;
                self.i2c.write(buffer, 2);
                self.state.set(State::Deactivating(x as i16, y as i16, z as i16));
            }
            State::Deactivating(x, y, z) => {
                self.i2c.disable();
                self.state.set(State::Disabled);
                self.buffer.replace(buffer);

                // Notify the current app of the reading
                self.current_app.get().map_or_else(|| {
                    panic!("no current app!!");
                }, |appid| {
                    self.current_app.set(None);
                    self.apps.enter(appid, |app, _| {
                        // app.pending_command = false;
                        app.callback.map_or_else(|| {
                            panic!("no bc");
                        }, |mut cb| {
                            cb.schedule(x as usize, y as usize, z as usize);
                        });
                    });
                });

                // Check to see if there are any pending operations
                let mut running_command = false;
                if self.current_app.get().is_none() {
                    for cntr in self.apps.iter() {
                        let started_command = cntr.enter(|app, _| {
                            if app.pending_command {
                                app.pending_command = false;
                                self.current_app.set(Some(app.appid()));

                                // panic!("why is command 4645?? {} {}", app.appid().idx, app.command);

                                // self.start_read_accel();
                                // true

                                match app.command {
                                    0 => {

                                        self.start_read_accel();
                                        true
                                    }
                                    _ => false
                                }
                            } else {
                                false
                            }
                        });
                        if started_command {
                            running_command = true;
                            break;
                        }
                    }
                    // if !running_command {
                    //     panic!("no pending command");
                    // }
                } else {
                    panic!("how is there a current app already?");
                }
            }
        }
    }
}

impl<'a> Driver for Fxos8700cq<'a> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            0 => {
                self.apps.enter(callback.app_id(), |app, _| {
                    app.callback = Some(callback);
                    0
                })
                .unwrap_or(-1)
            }
            _ => -1,
        }
    }

    fn command(&self, command_num: usize, _arg1: usize, appid: AppId) -> isize {
        self.apps.enter(appid, |app, _| {
            // Check so see if we are doing something. If not,
            // go ahead and do this command. If so, this is queued
            // and will be run when the pending command completes.
            if self.current_app.get().is_none() {
                self.current_app.set(Some(appid));
                match command_num {
                    0 => {
                        self.start_read_accel();
                        0
                    }
                    _ => -1,
                }
            } else {
                // panic!("woah");
                app.pending_command = true;
                app.command = command_num;
                -10
            }
        }).unwrap_or(-1)
    }
}
