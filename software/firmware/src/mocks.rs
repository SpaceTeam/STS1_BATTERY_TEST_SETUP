use crate::traits::GpioOutput;

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
