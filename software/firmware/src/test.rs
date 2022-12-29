#[cfg(test)]
mod tests {
    use crate::mocks::MockGpioOutput;
    use crate::Firmware;

    macro_rules! new_mock_firmware {
        () => {
            Firmware {
                on_board_led: MockGpioOutput { value: false },
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
}
