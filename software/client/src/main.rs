use bbqueue::{BBBuffer, Consumer, Producer};
use heapless::String;
use serde::{Deserialize, Serialize};
use serialport::{self, SerialPort};
use std::{thread, time};
use transmission::{
    receive::receive,
    send::{send, setup},
};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MsgTypes {
    Msg(String<32>),
    Test1(u32),
    Test2(f32, u8),
}

fn main() {
    println!("Hello, world!");

    let mut port = serialport::new("COM5", 115200)
        .open()
        .expect("Couldn't open the serial port");

    println!("{:?}", port.name());

    let buf_tx: BBBuffer<1024> = BBBuffer::new();
    let buf_rx: BBBuffer<1024> = BBBuffer::new();
    let (mut prod_tx, mut cons_tx) = buf_tx.try_split().unwrap();
    let (mut prod_rx, mut cons_rx) = buf_rx.try_split().unwrap();

    let count = port.bytes_to_read().unwrap() as usize;
    let mut buf = vec![0u8; count as usize];
    port.read(&mut buf);
    dbg!(&buf);

    let mut wgr = prod_rx.grant_exact(count).unwrap();
    wgr.buf().copy_from_slice(buf.as_slice());
    wgr.commit(count);

    // dbg!(cons_rx.read().unwrap());

    // receive(cons, cb)
    receive::<MsgTypes, 1024>(&mut cons_rx, |msg| {
        println!("{:?}", msg);
    });

    // print_input(&mut port);

    // write!(port, "Received {} bytes", count).unwrap();
    // thread::sleep(time::Duration::from_secs(1));
    // print_input(&mut port);

    println!("Done");
}

fn print_input(port: &mut Box<dyn SerialPort>) {
    let count = port.bytes_to_read().unwrap();
    let mut buf = vec![0u8; count as usize];
    port.read(&mut buf);

    // let text = String::from_utf8_lossy(&buf);
    println!("{:?}", buf);
}
