use bbqueue::{Consumer, SplitGrantR};
use postcard::from_bytes_cobs;
use serde::Deserialize;

pub fn receive<T: for<'a> Deserialize<'a>, const N: usize>(cons: &mut Consumer<N>, cb: impl Fn(T)) {
    if is_at_package_start(cons) == false {
        skip_to_package_start(cons);
    }

    // TODO: use the length (first  byte send) instead of looking for the end
    // TODO: handle timeout
    let end_index = match find_package_end(cons) {
        Some(end_index) => end_index,
        None => return,
    };

    // TODO: Refactor this mess
    let grant: SplitGrantR<N> = match cons.split_read() {
        Ok(grant) => grant,
        Err(_) => return,
    };
    let (buf1, buf2) = grant.bufs();
    let mut tmp = [0u8; N];

    tmp[..buf1.len() - 2].clone_from_slice(&buf1[2..]);
    tmp[buf1.len() - 2..buf1.len() - 2 + buf2.len()].clone_from_slice(&buf2);

    decode::<T, N>(&mut tmp[..end_index], cb);

    grant.release(end_index);
}

fn find_package_end<const N: usize>(cons: &mut Consumer<N>) -> Option<usize> {
    let grant: SplitGrantR<N> = match cons.split_read() {
        Ok(grant) => grant,
        Err(_) => return None,
    };
    let (buf1, buf2) = grant.bufs();

    let iter = buf1.iter().chain(buf2.iter());

    match iter.skip(1).position(|byte| *byte == 0) {
        Some(pos) => Some(pos + 1), // the skip will effect the result of position()
        None => None,
    }
}

fn is_at_package_start<const N: usize>(cons: &mut Consumer<N>) -> bool {
    let grant: SplitGrantR<N> = match cons.split_read() {
        Ok(grant) => grant,
        Err(_) => return false,
    };
    let (buf1, buf2) = grant.bufs();
    let mut iter = buf1.iter().chain(buf2.iter());

    let mut valid = match iter.next() {
        Some(byte) => *byte == 0,
        None => false,
    };

    valid &= match iter.next() {
        Some(byte) => *byte != 0,
        None => false,
    };

    return valid;
}

fn skip_to_package_start<const N: usize>(cons: &mut Consumer<N>) {
    let grant: SplitGrantR<N> = match cons.split_read() {
        Ok(grant) => grant,
        Err(_) => return,
    };
    let (buf1, buf2) = grant.bufs();

    let iter = buf1.iter().chain(buf2.iter());
    let non_zeros_to_skip = iter.take_while(|byte| **byte != 0).count();

    let iter = buf1.iter().chain(buf2.iter());
    let zeros_to_skip = &iter
        .skip(non_zeros_to_skip)
        .take_while(|byte| **byte == 0)
        .count();

    // keep the last zero, so that we know where at the start of a package
    grant.release(zeros_to_skip + non_zeros_to_skip - 1);
}

pub fn decode<T: for<'a> Deserialize<'a>, const N: usize>(data: &mut [u8], cb: impl Fn(T)) {
    let res = from_bytes_cobs::<T>(data);

    match res {
        Ok(msg) => {
            cb(msg);
        }
        Err(_) => panic!("Could not decode data"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use bbqueue::BBBuffer;
    use heapless::String;
    use serde::{Deserialize, Serialize};
    use std::cell::Cell;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub enum MsgTypes {
        Msg(String<32>),
        Test1(u32),
        Test2(f32, u8),
    }

    #[test]
    fn test_decode_1() {
        {
            let called = Cell::new(false);
            decode::<MsgTypes, 32>(&mut [3, 1, 18, 0], |msg| {
                assert_eq!(msg, MsgTypes::Test1(18));
                called.set(true);
            });
            assert!(called.get());
        }
        {
            let called = Cell::new(false);
            decode::<MsgTypes, 32>(&mut [2, 2, 1, 4, 64, 63, 13, 0], |msg| {
                assert_eq!(msg, MsgTypes::Test2(0.75, 13));
                called.set(true);
            });
            assert!(called.get());
        }
        {
            let called = Cell::new(false);
            decode::<MsgTypes, 32>(&mut [1, 7, 5, 72, 101, 108, 108, 111, 0], |msg| {
                assert_eq!(msg, MsgTypes::Msg(String::from("Hello")));
                called.set(true);
            });
            assert!(called.get());
        }
    }

    #[test]
    fn test_receive_1() {
        macro_rules! t {
            ($data:expr, $valid:expr, $result:expr) => {
                let buf: BBBuffer<32> = BBBuffer::new();
                let (mut prod, mut cons) = buf.try_split().unwrap();

                write_data!(prod, $data);

                let called = Cell::new(false);
                receive::<MsgTypes, 32>(&mut cons, |msg| {
                    assert_eq!(msg, $result);
                    called.set(true);
                });
                assert_eq!(called.get(), $valid);
                assert_bufs_eq!(cons, [0]);
            };
        }

        t!([0, 4, 3, 1, 18, 0], true, MsgTypes::Test1(18));
        t!([1, 1, 0, 0, 0, 4, 3, 1, 18, 0], true, MsgTypes::Test1(18));

        t!([99, 4, 3, 1, 18, 0], false, MsgTypes::Test1(18));
    }

    #[test]
    fn test_skip_to_package_start_1() {
        let buf: BBBuffer<32> = BBBuffer::new();
        let (mut prod, mut cons) = buf.try_split().unwrap();

        skip_to_package_start(&mut cons);
        assert!(cons.read().is_err());

        write_data!(prod, [1, 2, 3, 4, 0, 1]);

        skip_to_package_start(&mut cons);
        assert_bufs_eq!(cons, [0, 1]);

        skip_to_package_start(&mut cons);
        assert_bufs_eq!(cons, [0, 1]);
    }

    #[test]
    fn test_skip_to_package_start_2() {
        let buf: BBBuffer<32> = BBBuffer::new();
        let (mut prod, mut cons) = buf.try_split().unwrap();

        write_data!(prod, [1, 2, 0, 0, 0]);

        skip_to_package_start(&mut cons);
        assert_bufs_eq!(cons, [0]);

        write_data!(prod, [0, 5, 6]);

        skip_to_package_start(&mut cons);
        assert_bufs_eq!(cons, [0, 5, 6]);
    }

    #[test]
    fn test_find_package_end() {
        let buf: BBBuffer<32> = BBBuffer::new();
        let (mut prod, mut cons) = buf.try_split().unwrap();

        write_data!(prod, [0, 1, 2, 0, 1]);

        assert_eq!(find_package_end(&mut cons), Some(3));
    }

    #[test]
    fn test_is_package_start() {
        macro_rules! t {
            ($data:expr, $result:expr) => {
                let buf: BBBuffer<32> = BBBuffer::new();
                let (mut prod, mut cons) = buf.try_split().unwrap();
                write_data!(prod, $data);
                assert_eq!(is_at_package_start(&mut cons), $result);
            };
        }

        t!([0], false);
        t!([1], false);
        t!([0, 1], true);
        t!([1, 1], false);
        t!([1, 0], false);
        t!([0, 0], false);
        t!([0, 0, 1], false);
        t!([0, 2, 3], true);
    }
}
