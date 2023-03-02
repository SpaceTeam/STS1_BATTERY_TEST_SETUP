use crate::msg_types::MsgTypes;
use crate::traits::*;
use std::collections::VecDeque;

// +--------------------------------------------------------------------------+
// |                               GPIO Output                                |
// +--------------------------------------------------------------------------+

#[derive(PartialEq)]
pub struct MockGpioOutput {
    pub value: bool,
}

impl GpioOutput for MockGpioOutput {
    fn set_output(&mut self, value: bool) {
        self.value = value;
    }

    fn get_output(&mut self) -> bool {
        self.value
    }
}

// +--------------------------------------------------------------------------+
// |                                ADC Input                                 |
// +--------------------------------------------------------------------------+

pub struct MockAdcInput {
    pub voltage: f32,
}

impl MockAdcInput {
    pub fn new() -> Self {
        MockAdcInput { voltage: 0.0 }
    }

    pub fn from(voltage: f32) -> Self {
        MockAdcInput { voltage }
    }

    pub fn set_voltage(&mut self, voltage: f32) {
        self.voltage = voltage;
    }
}

impl AdcInput for MockAdcInput {
    fn get_voltage(&mut self) -> f32 {
        self.voltage
    }
}

// +--------------------------------------------------------------------------+
// |                                PWM Output                                |
// +--------------------------------------------------------------------------+

pub struct MockPwmOutput {
    pub duty_cycle: u16,
}

impl MockPwmOutput {
    pub fn new() -> Self {
        MockPwmOutput { duty_cycle: 0 }
    }
}

impl PwmOutput for MockPwmOutput {
    fn set_duty_cycle(&mut self, duty_cycle: u16) {
        self.duty_cycle = duty_cycle;
    }

    fn get_duty_cycle(&mut self) -> u16 {
        self.duty_cycle
    }

    fn get_max_duty_cycle(&mut self) -> u16 {
        100
    }
}

// +--------------------------------------------------------------------------+
// |                             Serial Receiver                              |
// +--------------------------------------------------------------------------+

pub struct MockSerialReceiver {
    pub msg_queue: VecDeque<MsgTypes>,
}

impl MockSerialReceiver {
    pub fn new(msg_queue: Vec<MsgTypes>) -> Self {
        MockSerialReceiver {
            msg_queue: VecDeque::from(msg_queue),
        }
    }
}

impl SerialReceiver for MockSerialReceiver {
    fn receive(&mut self, mut cb: impl FnMut(MsgTypes)) {
        cb(self.msg_queue.pop_front().unwrap());
    }
}

// +--------------------------------------------------------------------------+
// |                            Serial Transmitter                            |
// +--------------------------------------------------------------------------+

pub struct MockSerialTransmitter {
    pub msg_queue: VecDeque<MsgTypes>,
}

impl MockSerialTransmitter {
    pub fn new() -> Self {
        MockSerialTransmitter {
            msg_queue: VecDeque::new(),
        }
    }
}

impl SerialTransmitter for MockSerialTransmitter {
    fn transmit(&mut self, msg: MsgTypes) {
        self.msg_queue.push_back(msg);
    }
}
