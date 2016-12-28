//! Driver for the Microchip MCP23008 I2C GPIO Extender
//!
//! http://www.microchip.com/wwwproducts/en/MCP23008

use core::cell::Cell;

use kernel::common::take_cell::TakeCell;
use kernel::hil;

// Buffer to use for I2C messages
pub static mut BUFFER : [u8; 4] = [0; 4];

#[allow(dead_code)]
enum Registers {
    IoDir = 0x00,
    IPol = 0x01,
    GpIntEn = 0x02,
    DefVal = 0x03,
    IntCon = 0x04,
    IoCon = 0x05,
    GpPu = 0x06,
    IntF = 0x07,
    IntCap = 0x08,
    Gpio = 0x09,
    OLat = 0x0a,
}

/// States of the I2C protocol with the MCP23008.
#[derive(Clone,Copy,PartialEq)]
enum State {
    Idle,

    SelectIoDir,
    ReadIoDir,
    SelectGpPu,
    ReadGpPu,
    SelectGpio,
    ReadGpio,
    SelectGpioToggle,
    ReadGpioToggle,
    SelectGpioRead,
    ReadGpioRead,
    EnableInterruptSettings,
    ReadInterruptSetup,
    ReadInterruptValues,

    /// Disable I2C and release buffer
    Done,
}

enum Direction {
    Input = 0x01,
    Output = 0x00,
}

enum PinState {
    High = 0x01,
    Low = 0x00,
}

pub struct MCP23008<'a> {
    i2c: &'a hil::i2c::I2CDevice,
    state: Cell<State>,
    buffer: TakeCell<&'static mut [u8]>,
    interrupt_pin: Option<&'static hil::gpio::Pin>,
    interrupt_settings: Cell<u32>, // Whether the pin interrupt is enabled, and what mode it's in.
    identifier: Cell<usize>,
    client: TakeCell<&'static hil::gpio_async::Client>,
}

impl<'a> MCP23008<'a> {
    pub fn new(i2c: &'a hil::i2c::I2CDevice, interrupt_pin: Option<&'static hil::gpio::Pin>, buffer: &'static mut [u8]) -> MCP23008<'a> {
        MCP23008{
            i2c: i2c,
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
            interrupt_pin: interrupt_pin,
            interrupt_settings: Cell::new(0),
            identifier: Cell::new(0),
            client: TakeCell::empty(),
        }
    }

    /// Set the client of this MCP23008 when commands finish or interrupts
    /// occur. The `identifier` is simply passed back with the callback
    /// so that the upper layer can keep track of which device triggered.
    pub fn set_client<C: hil::gpio_async::Client>(&self, client: &'static C, identifier: usize) {
        self.client.replace(client);
        self.identifier.set(identifier);
    }

    fn enable_host_interrupt(&self) -> isize {
        // We configure the MCP23008 to use an active high interrupt.
        // If we don't have an interrupt pin mapped to this driver then we
        // obviously can't do interrupts.
        self.interrupt_pin.map_or(-1, |interrupt_pin| {
            interrupt_pin.make_input();
            interrupt_pin.enable_interrupt(0, hil::gpio::InterruptMode::RisingEdge);
            0
        })
    }

    // fn disable_interrupts(&self) {
    //     self.interrupt_pin.map(|interrupt_pin| {
    //         interrupt_pin.disable_interrupt();
    //         interrupt_pin.disable();
    //     });
    // }

    fn set_direction(&self, pin_number: u8, direction: Direction) {
        self.buffer.take().map(|buffer| {
            // turn on i2c to send commands
            self.i2c.enable();

            buffer[0] = Registers::IoDir as u8;
            // Save settings in buffer so they automatically get passed to
            // state machine.
            buffer[1] = pin_number;
            buffer[2] = direction as u8;
            self.i2c.write(buffer, 1);
            self.state.set(State::SelectIoDir);
        });
    }

    fn configure_pullup(&self, pin_number: u8, enabled: bool) {
        self.buffer.take().map(|buffer| {
            // turn on i2c to send commands
            self.i2c.enable();

            buffer[0] = Registers::GpPu as u8;
            // Save settings in buffer so they automatically get passed to
            // state machine.
            buffer[1] = pin_number;
            buffer[2] = enabled as u8;
            self.i2c.write(buffer, 1);
            self.state.set(State::SelectGpPu);
        });
    }

    fn set_pin(&self, pin_number: u8, value: PinState) {
        self.buffer.take().map(|buffer| {
            // turn on i2c to send commands
            self.i2c.enable();

            buffer[0] = Registers::Gpio as u8;
            // Save settings in buffer so they automatically get passed to
            // state machine.
            buffer[1] = pin_number;
            buffer[2] = value as u8;
            self.i2c.write(buffer, 1);
            self.state.set(State::SelectGpio);
        });
    }

    fn toggle_pin(&self, pin_number: u8) {
        self.buffer.take().map(|buffer| {
            // turn on i2c to send commands
            self.i2c.enable();

            buffer[0] = Registers::Gpio as u8;
            // Save settings in buffer so they automatically get passed to
            // state machine.
            buffer[1] = pin_number;
            self.i2c.write(buffer, 1);
            self.state.set(State::SelectGpioToggle);
        });
    }

    fn read_pin(&self, pin_number: u8) {
        self.buffer.take().map(|buffer| {
            // turn on i2c to send commands
            self.i2c.enable();

            buffer[0] = Registers::Gpio as u8;
            // Save settings in buffer so they automatically get passed to
            // state machine.
            buffer[1] = pin_number;
            self.i2c.write(buffer, 1);
            self.state.set(State::SelectGpioRead);
        });
    }

    fn enable_interrupt_pin(&self, pin_number: u8, direction: hil::gpio::InterruptMode) {
        self.buffer.take().map(|buffer| {
            // turn on i2c to send commands
            self.i2c.enable();

            // Mark the settings that we have for this interrupt.
            // Since the MCP23008 only seems to support level interrupts
            // and both edge interrupts, we choose to use both edge interrupts
            // and then filter here in the driver if the user only asked
            // for one direction interrupts. To do this, we need to know what
            // the user asked for.
            self.save_pin_interrupt_state(pin_number, true, direction);

            // Setup interrupt configs that are true of all interrupts
            buffer[0] = Registers::IntCon as u8;
            buffer[1] = 0; // Make all pins toggle on every change.
            buffer[2] = 0b00000010; // Make MCP23008 interrupt pin active high.
            self.i2c.write(buffer, 3);
            self.state.set(State::EnableInterruptSettings);
        });
    }

    fn disable_interrupt_pin(&self, pin_number: u8) {
        self.buffer.take().map(|buffer| {
            // turn on i2c to send commands
            self.i2c.enable();

            // Clear this interrupt from our setup.
            self.remove_pin_interrupt_state(pin_number);

            // Just have to write the new interrupt settings.
            buffer[0] = Registers::GpIntEn as u8;
            buffer[1] = self.get_pin_interrupt_enabled_state();
            self.i2c.write(buffer, 2);
            self.state.set(State::Done);
        });
    }

    fn save_pin_interrupt_state(&self, pin_number: u8, enabled: bool, direction: hil::gpio::InterruptMode) {
        let mut current_state = self.interrupt_settings.get();
        // Clear out existing settings
        current_state &= !(0x0F << (4*pin_number));
        // Generate new settings
        let new_settings = (((enabled as u8) | ((direction as u8) << 1)) & 0x0F) as u32;
        // Update settings
        current_state |= new_settings << (4*pin_number);
        self.interrupt_settings.set(current_state);
    }

    fn remove_pin_interrupt_state(&self, pin_number: u8) {
        let new_settings = self.interrupt_settings.get() & !(0x0F << (4*pin_number));
        self.interrupt_settings.set(new_settings);
    }

    /// Create an 8 bit bitmask of which interrupts are enabled.
    fn get_pin_interrupt_enabled_state(&self) -> u8 {
        let current_state = self.interrupt_settings.get();
        let mut interrupts_enabled: u8 = 0;
        for i in 0..8 {
            if ((current_state >> (4*i)) & 0x01) == 0x01 {
                interrupts_enabled &= 1<<i;
            }
        }
        interrupts_enabled
    }

    fn check_pin_interrupt_enabled(&self, pin_number: u8) -> bool {
        (self.interrupt_settings.get() >> (pin_number*4)) & 0x01 == 0x01
    }

    fn get_pin_interrupt_direction(&self, pin_number: u8) -> hil::gpio::InterruptMode {
        let direction = self.interrupt_settings.get() >> ((pin_number*4)+1) & 0x03;
        match direction {
            0 => hil::gpio::InterruptMode::RisingEdge,
            1 => hil::gpio::InterruptMode::FallingEdge,
            _ => hil::gpio::InterruptMode::EitherEdge,
        }
    }

}

impl<'a> hil::i2c::I2CClient for MCP23008<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: hil::i2c::Error) {
        match self.state.get() {
            State::SelectIoDir => {
                self.i2c.read(buffer, 1);
                self.state.set(State::ReadIoDir);
            },
            State::ReadIoDir => {
                let pin_number = buffer[1];
                let direction = buffer[2];
                if direction == Direction::Input as u8 {
                    buffer[1] = buffer[0] | (1 << pin_number);
                } else {
                    buffer[1] = buffer[0] & !(1 << pin_number);
                }
                buffer[0] = Registers::IoDir as u8;
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);
            },
            State::SelectGpPu => {
                self.i2c.read(buffer, 1);
                self.state.set(State::ReadGpPu);
            },
            State::ReadGpPu => {
                let pin_number = buffer[1];
                let enabled = buffer[2] == 1;
                if enabled  {
                    buffer[1] = buffer[0] | (1 << pin_number);
                } else {
                    buffer[1] = buffer[0] & !(1 << pin_number);
                }
                buffer[0] = Registers::GpPu as u8;
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);
            },
            State::SelectGpio => {
                self.i2c.read(buffer, 1);
                self.state.set(State::ReadGpio);
            },
            State::ReadGpio => {
                let pin_number = buffer[1];
                let value = buffer[2];
                if value == PinState::High as u8 {
                    buffer[1] = buffer[0] | (1 << pin_number);
                } else {
                    buffer[1] = buffer[0] & !(1 << pin_number);
                }
                buffer[0] = Registers::Gpio as u8;
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);
            },
            State::SelectGpioToggle => {
                self.i2c.read(buffer, 1);
                self.state.set(State::ReadGpioToggle);
            },
            State::ReadGpioToggle => {
                let pin_number = buffer[1];
                buffer[1] = buffer[0] ^ (1 << pin_number);
                buffer[0] = Registers::Gpio as u8;
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);
            },
            State::SelectGpioRead => {
                self.i2c.read(buffer, 1);
                self.state.set(State::ReadGpioRead);
            },
            State::ReadGpioRead => {
                let pin_number = buffer[1];
                let pin_value = (buffer[0] >> pin_number) & 0x01;

                self.client.map(|client| {
                    client.done(pin_value as usize);
                });

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            },
            State::EnableInterruptSettings => {
                // Rather than read the current interrupts and write those
                // back, just write the entire register with our saved state.
                buffer[0] = Registers::GpIntEn as u8;
                buffer[1] = self.get_pin_interrupt_enabled_state();
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);
            },
            State::ReadInterruptSetup => {
                // Now read the interrupt flags and the state of the lines
                self.i2c.read(buffer, 3);
                self.state.set(State::ReadInterruptValues);
            },
            State::ReadInterruptValues => {
                let interrupt_flags = buffer[0];
                let pins_status = buffer[2];
                // Check each bit to see if that pin triggered an interrupt.
                for i in 0..8 {
                    // Check that this pin is actually enabled.
                    if !self.check_pin_interrupt_enabled(i) {
                        continue;
                    }
                    if (interrupt_flags >> i) & 0x01 == 0x01 {
                        // Use the GPIO register to determine which way the
                        // interrupt went.
                        let pin_status = (pins_status >> i) & 0x01;
                        let interrupt_direction = self.get_pin_interrupt_direction(i);
                        // Check to see if this was an interrupt we want
                        // to report.
                        let fire_interrupt = match interrupt_direction {
                            hil::gpio::InterruptMode::EitherEdge => true,
                            hil::gpio::InterruptMode::RisingEdge => pin_status == 0x01,
                            hil::gpio::InterruptMode::FallingEdge => pin_status == 0x00,
                        };
                        // if interrupt_direction == hil::gpio::InterruptMode::EitherEdge ||
                        //    (interrupt_direction == hil::gpio::InterruptMode::RisingEdge &&
                        //     pin_status == 0x01) ||
                        //    (interrupt_direction == hil::gpio::InterruptMode::FallingEdge &&
                        //     pin_status == 0x00) {
                        if fire_interrupt {
                            // Signal this interrupt to the application.
                            self.client.map(|client| {
                                // Put the port number in the lower half of the forwarded identifier.
                                let ret = (self.identifier.get() & 0x00FF) | ((i as usize) << 8);
                                client.fired(ret);
                            });
                            break;
                        }
                    }
                }
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            },
            State::Done => {
                self.client.map(|client| {
                    client.done(0);
                });

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            },
            _ => {}
        }
    }
}

impl<'a> hil::gpio::Client for MCP23008<'a> {
    fn fired(&self, _: usize) {
        self.buffer.take().map(|buffer| {
            // turn on i2c to send commands
            self.i2c.enable();

            // Need to read the IntF register which marks which pins
            // interrupted.
            buffer[0] = Registers::IntF as u8;
            self.i2c.write(buffer, 1);
            self.state.set(State::ReadInterruptSetup);
        });

        // // TODO: This should ask the chip which pins interrupted.
        // self.client.map(|client| {
        //     // Put the port number in the lower half of the forwarded identifier.
        //     client.fired(self.identifier.get() & 0x00FF);
        // });
    }
}

impl<'a> hil::gpio_async::GPIOAsyncPort for MCP23008<'a> {
    fn disable(&self, pin: usize) -> isize {
        // Best we can do is make this an input.
        self.set_direction(pin as u8, Direction::Input);
        0
    }

    fn enable_output(&self, pin: usize) -> isize {
        if pin > 7 {
            -1
        } else {
            self.set_direction(pin as u8, Direction::Output);
            0
        }
    }

    fn enable_input(&self, pin: usize, mode: hil::gpio::InputMode) -> isize {
        if pin > 7 {
            -1
        } else {
            self.set_direction(pin as u8, Direction::Input);
            match mode {
                hil::gpio::InputMode::PullUp => {
                    self.configure_pullup(pin as u8, true);
                },
                hil::gpio::InputMode::PullDown => {
                    // No support for this
                },
                hil::gpio::InputMode::PullNone => {
                    self.configure_pullup(pin as u8, false);
                },
            }
            0
        }
    }

    fn read(&self, pin: usize) -> isize {
        if pin > 7 {
            -1
        } else {
            self.read_pin(pin as u8);
            0
        }
    }

    fn toggle(&self, pin: usize) -> isize {
        if pin > 7 {
            -1
        } else {
            self.toggle_pin(pin as u8);
            0
        }
    }

    fn set(&self, pin: usize) -> isize {
        if pin > 7 {
            -1
        } else {
            self.set_pin(pin as u8, PinState::High);
            0
        }
    }

    fn clear(&self, pin: usize) -> isize {
        if pin > 7 {
            -1
        } else {
            self.set_pin(pin as u8, PinState::Low);
            0
        }
    }

    fn enable_interrupt(&self, pin: usize, mode: hil::gpio::InterruptMode) -> isize {
        let main_interrupt_enabled = self.enable_host_interrupt();
        if main_interrupt_enabled < 0 {
            return -1;
        }
        self.enable_interrupt_pin(pin as u8, mode);
        0
    }

    fn disable_interrupt(&self, pin: usize) -> isize {
        self.disable_interrupt_pin(pin as u8);
        0
    }
}
