pub trait GpioOutput {
    fn set_output(&mut self, value: bool);
    fn get_output(&mut self) -> bool;
    fn toggle(&mut self) {
        let val = !self.get_output();
        self.set_output(val);
    }
}
