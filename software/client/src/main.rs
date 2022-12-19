use bbqueue::BBBuffer;
use heapless::String;
use serde::{Deserialize, Serialize};
use serialport;
use ui::AppEvent;

mod input_parser;
mod serial_manager;
mod ui;

const BUFFER_SIZE: usize = 1024;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MsgTypes {
    Msg(String<128>),
    Ping(u16),
    Test1(u32),
    Test2(f32, u8),
}

fn main() {
    let mut terminal = ui::setup().unwrap();
    let mut app = ui::App::default();

    let port = serialport::new("COM5", 115200)
        .open()
        .expect("Couldn't open the serial port");
    let buf_tx: BBBuffer<BUFFER_SIZE> = BBBuffer::new();
    let buf_rx: BBBuffer<BUFFER_SIZE> = BBBuffer::new();
    let (prod_tx, cons_tx) = buf_tx.try_split().unwrap();
    let (prod_rx, cons_rx) = buf_rx.try_split().unwrap();
    let mut port = serial_manager::SerialManager::new(port, prod_tx, cons_tx, prod_rx, cons_rx);

    port.setup();

    loop {
        match ui::update(&mut terminal, &mut app) {
            AppEvent::Quit => break,
            AppEvent::Input(input) => {
                app.messages.push(format!("invalid input: {}", input));
            }
            AppEvent::SendPing(val) => {
                app.messages.push(format!("sending ping {}", val));
                port.send(MsgTypes::Ping(val));
            }
            _ => {}
        }

        port.update();

        port.receive(|msg| match msg {
            MsgTypes::Msg(msg) => {
                app.messages.push(format!("received msg: {}", msg));
            }
            MsgTypes::Ping(val) => {
                app.messages.push(format!("received ping: {}", val));
            }
            _ => {
                app.messages.push(format!(
                    "received something, but this message isn't implemented for the variant"
                ));
            }
        });

        std::thread::sleep(std::time::Duration::from_millis(15));
    }

    ui::restore(&mut terminal).unwrap();
}
