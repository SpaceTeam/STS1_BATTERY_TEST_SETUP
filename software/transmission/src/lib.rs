// #![cfg_attr(not(test), no_std)]

#[macro_use]
mod macros;
pub mod receive;
pub mod send;

#[cfg(test)]
mod test {
    use super::*;
    #[allow(unused_imports)]
    use bbqueue::BBBuffer;
    use heapless::String;
    use serde::{Deserialize, Serialize};

    use receive::receive;
    use send::{send, setup};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub enum MsgTypes {
        Msg(String<32>),
        Test1(u32),
        Test2(f32, u8),
    }

    #[test]
    fn test_transmission() {
        {
            let buf: BBBuffer<128> = BBBuffer::new();
            let (mut prod, mut cons) = buf.try_split().unwrap();

            setup(&mut prod);
            send(&mut prod, MsgTypes::Test1(18)).unwrap();
            receive::<MsgTypes, 128>(&mut cons, |msg| {
                assert_eq!(msg, MsgTypes::Test1(18));
            });

            assert_bufs_eq!(cons, [0]);
        }
        {
            let buf: BBBuffer<128> = BBBuffer::new();
            let (mut prod, mut cons) = buf.try_split().unwrap();

            setup(&mut prod);
            send(&mut prod, MsgTypes::Test1(18)).unwrap();
            send(&mut prod, MsgTypes::Test2(1.25, 0)).unwrap();
            receive::<MsgTypes, 128>(&mut cons, |msg| {
                assert_eq!(msg, MsgTypes::Test1(18));
            });

            send(&mut prod, MsgTypes::Msg(String::from("Hallo!!!"))).unwrap();

            receive::<MsgTypes, 128>(&mut cons, |msg| {
                assert_eq!(msg, MsgTypes::Test2(1.25, 0));
            });
            receive::<MsgTypes, 128>(&mut cons, |msg| {
                assert_eq!(msg, MsgTypes::Msg(String::from("Hallo!!!")));
            });

            assert_bufs_eq!(cons, [0]);
        }
    }
}
