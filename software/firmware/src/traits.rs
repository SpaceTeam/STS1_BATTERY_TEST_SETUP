use time::PrimitiveDateTime;

use crate::msg_types::MsgTypes;

pub trait GpioOutput {
    fn set_output(&mut self, value: bool);
    fn get_output(&mut self) -> bool;
    fn toggle(&mut self) {
        let val = !self.get_output();
        self.set_output(val);
    }
}

pub trait GpioInput {
    fn get_input(&mut self) -> bool;
}

pub trait AdcInput {
    fn get_voltage(&mut self) -> f32;
}

pub trait PwmOutput {
    fn set_duty_cycle(&mut self, duty_cycle: u16);
    fn get_duty_cycle(&mut self) -> u16;
    fn get_max_duty_cycle(&mut self) -> u16;

    fn get_min_duty_cycle(&mut self) -> u16 {
        0
    }
}

pub trait SerialReceiver {
    fn receive(&mut self, cb: impl FnMut(MsgTypes));
}

pub trait SerialTransmitter {
    fn transmit(&mut self, msg: MsgTypes);
}

pub trait RealTimeClock {
    fn set_datetime(&mut self, datetime: PrimitiveDateTime);
    fn get_datetime(&mut self) -> PrimitiveDateTime;
}

pub trait SystemTime {
    fn get_delta_time(&self) -> f64;
    fn get_delta_time_micros(&self) -> u32;
    fn get_delta_time_millis(&self) -> u32 {
        self.get_delta_time_micros() / 1000
    }
}
