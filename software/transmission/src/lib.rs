#![cfg_attr(not(test), no_std)]

use core::ops::Deref;
use heapless::Vec;
use postcard::{from_bytes, ser_flavors::*, serialize_with_flavor};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Test<'a> {
    time: u64,
    msg: &'a str,
}

/* Having just one PackageSize for every package should be fine, because we will not store the serialized objects. The will be kept just for sending and than should be free again. */
const PACKAGE_SIZE: usize = 128;
pub type Package = Vec<u8, PACKAGE_SIZE>;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum MsgTypes<'a> {
    Msg(&'a str),
    Error(&'a str),
    Test1(f32, u8),
    Test2(u32),
    Test3(Test<'a>),
    // DataFrame(Df<'a>),
}

pub fn encode(t: MsgTypes) -> Package {
    // let bytes: Result<Package, postcard::Error> = to_vec(&t);
    let bytes = serialize_with_flavor::<
        MsgTypes,
        HVec<PACKAGE_SIZE>,
        heapless::Vec<u8, PACKAGE_SIZE>,
    >(&t, HVec::default());

    return bytes.unwrap();
}

pub fn decode<'a>(p: &'a Package) -> MsgTypes {
    let res: Result<MsgTypes, postcard::Error> = from_bytes(p.deref());
    res.unwrap()
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    // use heapless::pool::Uninit;

    use super::*;

    #[test]
    fn test_encode() {
        let tmp = MsgTypes::Test2(3);

        let p: &Package = &encode(tmp);

        assert_eq!(p.len(), 2);
        assert_eq!(p, &[3, 3]);
    }

    #[test]
    fn test_encode_2() {
        let tmp = MsgTypes::Test1(16.5, 8);

        let p: &Package = &encode(tmp);

        assert_eq!(p.len(), 6);
        assert_eq!(p, &[2, 0, 0, 132, 65, 8]);
    }

    #[test]
    fn test_encode_3() {
        let tmp = MsgTypes::Msg("testtesttesttest_testtesttesttest_testtesttesttest_testtesttesttest_testtesttesttest_testtesttesttest_testtesttesttest");

        let p: &Package = &encode(tmp);

        assert_eq!(p.len(), 120);
        assert_eq!(
            p,
            &[
                0, 118, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115,
                116, 95, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115,
                116, 95, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115,
                116, 95, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115,
                116, 95, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115,
                116, 95, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115,
                116, 95, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115,
                116,
            ],
        );
    }

    #[test]
    fn test_decode() {
        let mut d: Package = Package::new();
        d.extend([3, 5]);

        let msg = decode(&d);

        assert_eq!(msg, MsgTypes::Test2(5));
    }

    #[test]
    fn test_decode_2() {
        let mut d: Package = Package::new();
        d.extend([0, 4, 116, 101, 115, 116]);

        let msg = decode(&d);

        assert_eq!(msg, MsgTypes::Msg("test"));
    }

    #[test]
    fn test_decode_3() {
        let mut d: Package = Package::new();
        d.extend([
            0, 118, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116,
            95, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 95,
            116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 95,
            116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 95,
            116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 95,
            116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 95,
            116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116, 116, 101, 115, 116,
        ]);

        let msg = decode(&d);

        assert_eq!(msg, MsgTypes::Msg("testtesttesttest_testtesttesttest_testtesttesttest_testtesttesttest_testtesttesttest_testtesttesttest_testtesttesttest"));
    }

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_vec() {
        let mut v = vec![1, 2];

        v.push(3);

        assert_eq!(v.len(), 3);
    }
}
