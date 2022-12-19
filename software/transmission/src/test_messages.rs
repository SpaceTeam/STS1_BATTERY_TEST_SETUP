use heapless::String;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Data {
    pub timestamp: u32,
    pub data: [u8; 16],
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MsgTypes {
    Msg(String<32>),
    Test1(u32),
    Test2(f32, u8),
    Data(Data),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CallbackCalled {
    Ok,
    Err,
    None,
}
