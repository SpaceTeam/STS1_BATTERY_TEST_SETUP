#![cfg_attr(not(test), no_std)]

use libm;
use msg_types::MsgTypes;
use traits::{AdcInput, PwmOutput};

use crate::traits::*;

#[cfg(test)]
mod mocks;
pub mod msg_types;
mod test;
pub mod traits;

macro_rules! generate_firmware {
    ( $( ($field_name:ident ; $type_name:ident : $trait:path) ),+ ;
      $( ( $obj_field_name:ident ; ( $($obj_type_name:ident : $obj_trait:path),+ ) ; $obj_type:ident ) ),+ ) => {

        pub struct Firmware<$( $type_name:  $trait, )+ $( $( $obj_type_name: $obj_trait, )+ )+> {
            $( pub $field_name: $type_name, )+
            $( pub $obj_field_name: $obj_type<$( $obj_type_name, )+>, )+
        }

        impl <$( $type_name:  $trait, )+ $( $( $obj_type_name: $obj_trait, )+ )+> Firmware<$( $type_name, )+ $($($obj_type_name,)+)+> {
            fn setup(&mut self) {
                self.on_board_led.set_output(false);
            }

            pub fn toggle_on_board_led(&mut self) {
                self.on_board_led.toggle();
            }

            pub fn update_serial(&mut self) {
                self.serial_receiver.receive(|val| match val {
                    MsgTypes::Ping(value) => {
                        self.serial_transmitter.transmit(MsgTypes::Ping(value + 1));
                    }
                    _ => {
                        unimplemented!();
                    }
                });
            }

            pub fn update_battery_units(&mut self, time: f32, delta_time: f32) {
                self.btu1.update(time, delta_time);
            }
        }
    };
}

generate_firmware!(
   (serial_receiver; TSerialRx: SerialReceiver),
   (serial_transmitter; TSerialTx: SerialTransmitter),
   (on_board_led; TLed: GpioOutput);
   (btu1; (TAdcInput1: AdcInput, TPwmOutput1: PwmOutput); BatteryTestUnit)
);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BatteryTestUnitMode {
    Idle,
    Charging,
    Discharging(f32),
}

macro_rules! generate_battery_test_unit {
    ( $( ($field_name:ident ; $type_name:ident : $trait:path) ),+ ) => {

        pub struct BatteryTestUnit<$( $type_name: $trait, )+> {
            current_mode: BatteryTestUnitMode,
            $( pub $field_name: $type_name, )+
        }

        impl<$( $type_name: $trait, )+> BatteryTestUnit<$( $type_name, )+> {
            pub fn new( $( $field_name: $type_name, )+ ) -> Self {
                let mut res = Self {
                    current_mode: BatteryTestUnitMode::Idle,
                    $( $field_name, )+
                };
                res.set_mode(BatteryTestUnitMode::Idle);
                res
            }

            pub fn update(&mut self, time: f32, delta_time: f32) {
                match self.current_mode {
                    BatteryTestUnitMode::Idle => {}
                    BatteryTestUnitMode::Charging => {}
                    BatteryTestUnitMode::Discharging(target_voltage) => {
                        let output = libm::sinf(time * 3.1415) * 0.5 + 0.5;
                        let output = output * self.load_pwm.get_max_duty_cycle() as f32;
                        self.load_pwm.set_duty_cycle(output as u16);
                    }
                }
            }

            pub fn set_mode(&mut self, new_mode: BatteryTestUnitMode) {
                match (self.current_mode, new_mode) {
                    (_, BatteryTestUnitMode::Idle) => {
                        let min = self.load_pwm.get_min_duty_cycle();
                        self.load_pwm.set_duty_cycle(min);
                    }
                    (BatteryTestUnitMode::Idle, BatteryTestUnitMode::Discharging(_)) => {
                        let min = self.load_pwm.get_min_duty_cycle();
                        self.load_pwm.set_duty_cycle(min);
                    }
                    _ => {
                        unimplemented!();
                    }
                }
                self.current_mode = new_mode;
            }

            pub fn get_mode(&self) -> BatteryTestUnitMode {
                self.current_mode
            }

            pub fn get_voltage(&mut self) -> f32 {
                self.voltage_adc.get_voltage()
            }
        }
    };
}

generate_battery_test_unit!(
    (voltage_adc; TAdcVoltage: AdcInput),
    (load_pwm; TLoad: PwmOutput)
);
