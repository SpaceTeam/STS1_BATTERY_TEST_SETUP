#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use bbqueue::BBBuffer;
    use heapless::String;
    use std::cell::Cell;

    use crate::macros::*;
    use crate::receive::receive;
    use crate::send::{send, setup};
    use crate::test_messages::*;

    const BUF_SIZE: usize = 128;

    #[allow(unused_macros)]
    macro_rules! receive {
        ($data:expr, $result:expr, $called:expr) => {
            let called = Cell::new(CallbackCalled::None);
            receive::<MsgTypes, BUF_SIZE>(&mut $data, |res| match res {
                Ok(msg) => {
                    assert_eq!(msg, $result);
                    called.set(CallbackCalled::Ok);
                }
                Err(_) => {
                    called.set(CallbackCalled::Err);
                }
            });
            assert_eq!(called.get(), $called);
        };
    }

    #[allow(unused_macros)]
    macro_rules! receive_ok {
        ($data:expr, $result:expr) => {
            receive!($data, $result, CallbackCalled::Ok);
        };
    }

    #[allow(unused_macros)]
    macro_rules! receive_nothing {
        ($data:expr) => {
            receive!($data, MsgTypes::Test1(0), CallbackCalled::None);
        };
    }

    #[allow(unused_macros)]
    macro_rules! receive_error {
        ($data:expr) => {
            receive!($data, MsgTypes::Test1(0), CallbackCalled::Err);
        };
    }

    #[test]
    fn test_transmission_one_packet() {
        let buf: BBBuffer<BUF_SIZE> = BBBuffer::new();
        let (mut prod, mut cons) = buf.try_split().unwrap();

        setup(&mut prod);
        send(&mut prod, MsgTypes::Test1(18)).unwrap();

        receive_ok!(cons, MsgTypes::Test1(18));
        assert_bufs_eq!(cons, [0]);
    }

    #[test]
    fn test_transmission_multiple_packets() {
        let buf: BBBuffer<BUF_SIZE> = BBBuffer::new();
        let (mut prod, mut cons) = buf.try_split().unwrap();

        setup(&mut prod);
        send(&mut prod, MsgTypes::Test1(10)).unwrap();
        send(&mut prod, MsgTypes::Test1(20)).unwrap();
        receive_ok!(cons, MsgTypes::Test1(10));
        send(&mut prod, MsgTypes::Test1(30)).unwrap();
        receive_ok!(cons, MsgTypes::Test1(20));
        receive_ok!(cons, MsgTypes::Test1(30));

        assert_bufs_eq!(cons, [0]);
    }

    #[test]
    fn test_transmission_buffer_already_has_bad_data() {
        let buf: BBBuffer<BUF_SIZE> = BBBuffer::new();
        let (mut prod, mut cons) = buf.try_split().unwrap();

        write_data!(prod, [31, 26, 23]);
        receive_nothing!(cons);
        assert_bufs_eq!(cons, [23]);

        write_data!(prod, [0, 0, 0]);
        receive_nothing!(cons);
        assert_bufs_eq!(cons, [0]);

        setup(&mut prod);
        receive_nothing!(cons);
        assert_bufs_eq!(cons, [0]);

        send(&mut prod, MsgTypes::Test1(10)).unwrap();
        receive_ok!(cons, MsgTypes::Test1(10));

        assert_bufs_eq!(cons, [0]);
    }

    #[test]
    fn test_transmission_drop_bad_packet() {
        let buf: BBBuffer<BUF_SIZE> = BBBuffer::new();
        let (mut prod, mut cons) = buf.try_split().unwrap();

        write_data!(prod, [0, 31, 26, 23, 0, 0, 0]);

        receive_error!(cons);
        assert_bufs_eq!(cons, [0, 0, 0]);

        setup(&mut prod);
        receive_nothing!(cons);
        assert_bufs_eq!(cons, [0]);

        send(&mut prod, MsgTypes::Test1(10)).unwrap();
        receive_ok!(cons, MsgTypes::Test1(10));

        assert_bufs_eq!(cons, [0]);
    }
}
