use bbqueue::Producer;
use heapless::Vec;
use postcard::to_vec_cobs;
use serde::Serialize;

/// Call this once before sending the first package.
/// This will write a zero to the buffer, so that the receiver knows that the next byte is the start of a package.
pub fn setup<const N: usize>(producer: &mut Producer<N>) {
    let mut grant = producer.grant_exact(1).unwrap();
    grant[0] = 0;
    grant.commit(1);
}

pub fn send<T: Serialize, const N: usize>(
    prod: &mut Producer<N>,
    msg: T,
) -> Result<(), &'static str> {
    let encoded = encode::<T, N>(&msg)?;

    let length_needed = encoded.len() + 1;

    match prod.grant_exact(length_needed) {
        Ok(mut grant) => {
            grant[0] = encoded.len() as u8;
            grant[1..].copy_from_slice(&encoded);

            grant.commit(length_needed);
        }
        Err(_) => return Err("Could not grant space for message"),
    }

    Ok(())
}

pub fn encode<T: Serialize, const N: usize>(msg: &T) -> Result<Vec<u8, N>, &'static str> {
    match to_vec_cobs(msg) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err("Could not encode data"),
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use crate::test_messages::{Data, MsgTypes};
    #[allow(unused_imports)]
    use bbqueue::BBBuffer;
    use heapless::String;

    #[test]
    fn test_encode_1() {
        macro_rules! t {
            ($input:expr, $result:expr) => {
                let res = encode::<MsgTypes, 32>(&$input);
                assert_eq!(res, Ok(hVec!(32, $result)));
            };
        }

        t!(MsgTypes::Test1(18), [3, 1, 18, 0]);
        t!(MsgTypes::Test2(0.75, 13), [2, 2, 1, 4, 64, 63, 13, 0]);
        t!(
            MsgTypes::Msg(String::from("Hello")),
            [1, 7, 5, 72, 101, 108, 108, 111, 0]
        );
        t!(
            MsgTypes::Msg(String::from("PANIC!!!")),
            [1, 10, 8, 80, 65, 78, 73, 67, 33, 33, 33, 0]
        );
        t!(
            MsgTypes::Data(Data {
                timestamp: 123,
                data: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
            }),
            [19, 3, 123, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 0]
        );
    }

    #[test]
    fn test_send_1() {
        let buf: BBBuffer<32> = BBBuffer::new();
        let (mut prod, mut cons) = buf.try_split().unwrap();

        send(&mut prod, MsgTypes::Test1(18)).unwrap();

        let grant = cons.split_read().unwrap();
        let (buf1, buf2) = grant.bufs();

        assert_eq!(
            buf1[0],
            buf1.len() as u8 - 1,
            "first byte should be the length of the message"
        );
        assert_eq!(
            *buf1.last().unwrap(),
            0u8,
            "last byte should be the seperator"
        );
        assert_eq!(buf1, &[4, 3, 1, 18, 0]);
        assert_eq!(buf2, &[]);
    }

    #[test]
    fn test_send_2() {
        let buf: BBBuffer<32> = BBBuffer::new();
        let (mut prod, mut cons) = buf.try_split().unwrap();

        send(&mut prod, MsgTypes::Msg(String::from("STS1"))).unwrap();

        let grant = cons.split_read().unwrap();
        let (buf1, buf2) = grant.bufs();

        assert_eq!(
            buf1[0],
            buf1.len() as u8 - 1,
            "first byte should be the length of the message"
        );
        assert_eq!(
            *buf1.last().unwrap(),
            0u8,
            "last byte should be the seperator"
        );
        assert_eq!(buf1, &[8, 1, 6, 4, 83, 84, 83, 49, 0]);
        assert_eq!(buf2, &[]);
    }

    #[test]
    fn test_send_3() {
        let buf: BBBuffer<32> = BBBuffer::new();
        let (mut prod, mut cons) = buf.try_split().unwrap();

        send(&mut prod, MsgTypes::Test1(128)).unwrap();
        send(&mut prod, MsgTypes::Test2(1.0, 123)).unwrap();

        let grant = cons.split_read().unwrap();
        let (buf1, buf2) = grant.bufs();

        assert_eq!(buf1, &[5, 4, 1, 128, 1, 0, 8, 2, 2, 1, 4, 128, 63, 123, 0]);
        assert_eq!(buf2, &[]);
    }
}
