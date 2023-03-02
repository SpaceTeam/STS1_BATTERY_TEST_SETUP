use core::cell::RefCell;

use embedded_hal::PwmPin;
use firmware::traits;
use stm32f4xx_hal::adc::Adc;
use stm32f4xx_hal::gpio::GpioExt;
use stm32f4xx_hal::gpio::{Alternate, Analog, Input, Output, Pin, PinState, PushPull};
use stm32f4xx_hal::pac::{Peripherals, ADC1};
use stm32f4xx_hal::timer::pwm::PwmChannel;
use stm32f4xx_hal::timer::{Instance, PwmHz};

// +--------------------------------------------------------------------------+
// |                               GPIO Output                                |
// +--------------------------------------------------------------------------+

pub struct GpioOutput<const P: char, const N: u8, MODE = Input> {
    pin: Pin<P, N, MODE>,
}

impl<const P: char, const N: u8, MODE> GpioOutput<P, N, MODE> {
    pub fn new(pin: Pin<P, N, MODE>) -> Self {
        Self { pin }
    }
}

impl traits::GpioOutput for GpioOutput<'A', 5, Output<PushPull>> {
    fn set_output(&mut self, value: bool) {
        self.pin.set_state(PinState::from(value));
    }

    fn get_output(&mut self) -> bool {
        match self.pin.get_state() {
            PinState::High => true,
            PinState::Low => false,
        }
    }
}

// +--------------------------------------------------------------------------+
// |                                ADC Input                                 |
// +--------------------------------------------------------------------------+

pub struct AdcInput<const P: char, const N: u8, MODE = Analog> {
    adc: Adc<ADC1>,
    pin: Pin<P, N, MODE>,
}

impl<const P: char, const N: u8, MODE> AdcInput<P, N, MODE> {
    pub fn new(pin: Pin<P, N, MODE>, adc: Adc<ADC1>) -> Self {
        Self { pin, adc }
    }
}

impl traits::AdcInput for AdcInput<'A', 0, Analog> {
    fn get_voltage(&mut self) -> f32 {
        let sample = self.adc.convert(
            &mut self.pin,
            stm32f4xx_hal::adc::config::SampleTime::Cycles_112,
        );

        self.adc.sample_to_millivolts(sample) as f32 / 1000.0
    }
}

// +--------------------------------------------------------------------------+
// |                                PWM Output                                |
// +--------------------------------------------------------------------------+

pub struct PwmOutput<T: PwmPin> {
    pub pwm: T,
}

// impl PwmOutput {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

impl<T: PwmPin> traits::PwmOutput for PwmOutput<T>
where
    T::Duty: From<u16> + Into<u16>,
{
    fn set_duty_cycle(&mut self, duty_cycle: u16) {
        self.pwm.set_duty(duty_cycle.into());
    }

    fn get_duty_cycle(&mut self) -> u16 {
        self.pwm.get_duty().into()
    }

    fn get_max_duty_cycle(&mut self) -> u16 {
        self.pwm.get_max_duty().into()
    }
}

// +--------------------------------------------------------------------------+
// |                             Serial Receiver                              |
// +--------------------------------------------------------------------------+

pub struct SerialReceiver {}

impl traits::SerialReceiver for SerialReceiver {
    fn receive(&mut self, cb: impl FnMut(firmware::msg_types::MsgTypes)) {
        unimplemented!()
    }
}

// +--------------------------------------------------------------------------+
// |                            Serial Transmitter                            |
// +--------------------------------------------------------------------------+

pub struct SerialTransmitter {}

impl traits::SerialTransmitter for SerialTransmitter {
    fn transmit(&mut self, msg: firmware::msg_types::MsgTypes) {
        unimplemented!()
    }
}
