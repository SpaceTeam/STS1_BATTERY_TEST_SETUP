use crate::MsgTypes;
use bbqueue::{Consumer, Producer};
use serialport::SerialPort;
use transmission::{
    receive::receive,
    send::{send, setup},
};

pub struct SerialManager<'a, const N: usize> {
    port: Box<dyn SerialPort>,

    prod_rx: Producer<'a, N>,
    cons_rx: Consumer<'a, N>,

    prod_tx: Producer<'a, N>,
    cons_tx: Consumer<'a, N>,
}

impl<'a, const N: usize> SerialManager<'a, N> {
    pub fn new(
        port: Box<dyn SerialPort>,
        prod_rx: Producer<'a, N>,
        cons_rx: Consumer<'a, N>,
        mut prod_tx: Producer<'a, N>,
        cons_tx: Consumer<'a, N>,
    ) -> Self {
        Self {
            port,
            prod_rx,
            cons_rx,
            prod_tx,
            cons_tx,
        }
    }

    pub fn send(&mut self, msg: MsgTypes) {
        send(&mut self.prod_tx, msg).unwrap();
    }

    pub fn receive(&mut self, mut cb: impl FnMut(MsgTypes)) {
        receive::<MsgTypes, N>(&mut self.cons_rx, |val| match val {
            Ok(msg) => cb(msg),
            Err(_) => {}
        });
    }

    pub fn setup(&mut self) {
        setup(&mut self.prod_tx);
    }

    pub fn update(&mut self) {
        let bytes_to_read = self.port.bytes_to_read().unwrap() as usize;

        for _ in 0..bytes_to_read {
            let mut wgr = self.prod_rx.grant_exact(1).unwrap();
            let mut buf = [0u8; 1];
            self.port.read(&mut buf).unwrap();

            wgr.buf().copy_from_slice(buf.as_slice());
            wgr.commit(1);
        }

        let grant = match self.cons_tx.read() {
            Ok(grant) => grant,
            Err(_) => return,
        };
        let bytes_to_write = grant.buf().len();
        // println!("bytes_to_write: {} {:?}", bytes_to_write, grant.buf());
        grant.buf().iter().for_each(|val| {
            self.port.write(&[*val]).unwrap();
        });
        grant.release(bytes_to_write);

        // println!("bytes_to_write: {:?} ", self.port.bytes_to_write());
    }
}
