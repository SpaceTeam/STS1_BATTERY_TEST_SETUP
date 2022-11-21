use serialport::{self, SerialPort};
use std::{thread, time};

fn main() {
    println!("Hello, world!");

    let mut port = serialport::new("COM5", 115200)
        .open()
        .expect("Couldn't open the serial port");

    println!("{:?}", port.name());
    let count = port.bytes_to_read().unwrap();

    print_input(&mut port);

    write!(port, "Received {} bytes", count).unwrap();
    thread::sleep(time::Duration::from_secs(1));
    print_input(&mut port);

    println!("Done");
}

fn print_input(port: &mut Box<dyn SerialPort>) {
    let count = port.bytes_to_read().unwrap();
    let mut buf = vec![0u8; count as usize];
    port.read(&mut buf);

    let text = String::from_utf8_lossy(&buf);
    println!("{:?}", text);
}
