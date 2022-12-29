#![cfg_attr(not(test), no_std)]

use crate::traits::GpioOutput;

mod mocks;
pub mod msg_types;
mod test;
pub mod traits;

pub struct Firmware<TLed: GpioOutput> {
    pub on_board_led: TLed,
}

impl<TLed: GpioOutput> Firmware<TLed> {
    fn setup(&mut self) {
        self.on_board_led.set_output(false);
    }

    pub fn toggle_on_board_led(&mut self) {
        self.on_board_led.toggle();
    }
}
