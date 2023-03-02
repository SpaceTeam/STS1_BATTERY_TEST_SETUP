#[cfg(test)]
mod tests {
    use crate::mocks::*;
    use crate::msg_types::MsgTypes;
    use crate::traits::PwmOutput;
    use crate::{BatteryTestUnit, BatteryTestUnitMode, Firmware};

    macro_rules! new_mock_firmware {
        () => {
            new_mock_firmware!(Vec::new())
        };
        ($serial_rx_queue: expr) => {
            Firmware {
                on_board_led: MockGpioOutput { value: false },
                serial_receiver: MockSerialReceiver::new($serial_rx_queue),
                serial_transmitter: MockSerialTransmitter::new(),
                btu1: BatteryTestUnit::new(MockAdcInput::new(), MockPwmOutput::new()),
            }
        };
    }

    #[test]
    fn test_setup() {
        let mut firmware = new_mock_firmware!();

        firmware.setup();
        assert_eq!(firmware.on_board_led.value, false);
    }

    #[test]
    fn test_toggle_on_board_led() {
        let mut firmware = new_mock_firmware!();

        firmware.toggle_on_board_led();
        assert_eq!(firmware.on_board_led.value, true);

        firmware.toggle_on_board_led();
        assert_eq!(firmware.on_board_led.value, false);
    }

    #[test]
    fn test_serial_ping() {
        let mut firmware = new_mock_firmware!(vec![MsgTypes::Ping(1)]);

        firmware.update_serial();

        assert_eq!(firmware.serial_transmitter.msg_queue.len(), 1);
        assert_eq!(
            firmware.serial_transmitter.msg_queue.pop_front(),
            Some(MsgTypes::Ping(2))
        );
    }

    #[test]
    fn test_battery_unit_discharge_state_transition() {
        let mut btu = BatteryTestUnit::new(MockAdcInput::new(), MockPwmOutput::new());

        let MIN_DC = btu.load_pwm.get_min_duty_cycle();
        let MAX_DC = btu.load_pwm.get_max_duty_cycle();

        btu.voltage_adc.set_voltage(3.3);
        assert_eq!(btu.get_voltage(), 3.3);

        assert_eq!(btu.get_mode(), BatteryTestUnitMode::Idle);
        assert_eq!(btu.load_pwm.duty_cycle, MIN_DC);

        btu.set_mode(BatteryTestUnitMode::Discharging(2.5));
        assert_eq!(btu.get_mode(), BatteryTestUnitMode::Discharging(2.5));
        assert_eq!(btu.load_pwm.duty_cycle, MIN_DC);

        // assume the duty cycle was set
        btu.load_pwm.set_duty_cycle(123);

        btu.set_mode(BatteryTestUnitMode::Idle);
        assert_eq!(btu.get_mode(), BatteryTestUnitMode::Idle);
        assert_eq!(btu.load_pwm.duty_cycle, MIN_DC);
    }

    #[test]
    fn test_battery_unit_discharge() {
        let mut btu = BatteryTestUnit::new(MockAdcInput::new(), MockPwmOutput::new());

        let MIN_DC = btu.load_pwm.get_min_duty_cycle();

        btu.set_mode(BatteryTestUnitMode::Discharging(2.0));

        // btu.voltage_adc.set_voltage(3.3);
        assert_eq!(btu.load_pwm.duty_cycle, MIN_DC);

        btu.update(0.0, 0.5);
        assert_eq!(btu.load_pwm.duty_cycle, 50);

        btu.update(0.5, 0.5);
        assert_eq!(btu.load_pwm.duty_cycle, 100);

        btu.update(0.75, 0.5);
        assert_eq!(btu.load_pwm.duty_cycle, 85);

        btu.update(1.0, 0.5);
        assert_eq!(btu.load_pwm.duty_cycle, 50);

        btu.update(1.5, 0.5);
        assert_eq!(btu.load_pwm.duty_cycle, 0);

        btu.update(2.0, 0.5);
        assert_eq!(btu.load_pwm.duty_cycle, 49);
    }
}
