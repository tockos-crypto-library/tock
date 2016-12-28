//! Provide userspace applications with a driver interface to asynchronous
//! GPIO pins. These are pins that exist on something like a GPIO extender or
//! a radio that has controllable GPIOs.

use core::cell::Cell;

use kernel::hil;
use kernel::{AppId, Callback, Driver};

pub struct GPIOAsync<'a, Port: hil::gpio_async::GPIOAsyncPort + 'a> {
    ports: &'a [&'a Port],
    callback: Cell<Option<Callback>>,
}

impl<'a, Port: hil::gpio_async::GPIOAsyncPort> GPIOAsync<'a, Port> {
    pub fn new(ports: &'a [&'a Port]) -> GPIOAsync<'a, Port> {
        GPIOAsync {
            ports: ports,
            callback: Cell::new(None),
        }
    }

    fn configure_input_pin(&self, port: usize, pin: usize, config: usize) -> isize {
        let ports = self.ports.as_ref();
        if config > 2 {
            return -1;
        }
        let mode = match config {
            0 => hil::gpio::InputMode::PullUp,
            1 => hil::gpio::InputMode::PullDown,
            _ => hil::gpio::InputMode::PullNone,
        };
        ports[port].enable_input(pin, mode)
    }

    fn configure_interrupt(&self, port: usize, pin: usize, config: usize) -> isize {
        let ports = self.ports.as_ref();
        if config > 2 {
            return -1;
        }
        let mode = match config {
            0 => hil::gpio::InterruptMode::RisingEdge,
            1 => hil::gpio::InterruptMode::FallingEdge,
            _ => hil::gpio::InterruptMode::EitherEdge,
        };
        ports[port].enable_interrupt(pin, mode, port)
    }
}

impl<'a, Port: hil::gpio_async::GPIOAsyncPort> hil::gpio_async::Client for GPIOAsync<'a, Port> {
    fn fired(&self, port_pin_num: usize) {
        self.callback.get().map(|mut cb|
            cb.schedule(1, port_pin_num, 0)
        );
    }

    fn done(&self, value: usize) {
        self.callback.get().map(|mut cb|
            cb.schedule(0, value, 0)
        );
    }
}

impl<'a, Port: hil::gpio_async::GPIOAsyncPort> Driver for GPIOAsync<'a, Port> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            0 => {
                self.callback.set(Some(callback));
                0
            }

            // default
            _ => -1,
        }
    }

    fn command(&self, command_num: usize, data: usize, _: AppId) -> isize {
        let port = data & 0xFF;
        let pin = (data >> 8) & 0xFF;
        let other = (data >> 16) & 0xFFFF;
        let ports = self.ports.as_ref();

        // On any command other than 0, we check for ports length.
        if command_num != 0 && port >= ports.len() {
            return -1;
        }

        match command_num {
            // How many ports
            0 => ports.len() as isize,

            // enable output
            1 => ports[port].enable_output(pin),

            // set pin
            2 => ports[port].set(pin),

            // clear pin
            3 => ports[port].clear(pin),

            // toggle pin
            4 => ports[port].toggle(pin),

            // enable and configure input
            5 => self.configure_input_pin(port, pin, other & 0xFF),

            // read input
            6 => ports[port].read(pin),

            // enable interrupt on pin
            7 => self.configure_interrupt(port, pin, other & 0xFF),

            // disable interrupt on pin
            8 => ports[port].disable_interrupt(pin),

            // disable pin
            9 => ports[port].disable(pin),

            // default
            _ => -1,
        }
    }
}
