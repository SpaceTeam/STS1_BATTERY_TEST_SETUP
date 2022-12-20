use heapless::String;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MsgTypes {
    Msg(String<128>),
    Ping(u16),
    Test1(u32),
    Test2(f32, u8),
}
