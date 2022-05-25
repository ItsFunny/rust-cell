use rocket::time::macros::time;
use crate::constants::EnumsProtocolStatus::Status;

pub enum EnumsProtocolStatus {
    Status(i64),
}

impl EnumsProtocolStatus {
    pub fn get_i64(self) -> i64 {
        match self {
            Status(v) => {
                v
            }
            _ => {
                0
            }
        }
    }
}

pub struct ProtocolStatus {}

impl ProtocolStatus {
    pub const SUCCESS: &'static EnumsProtocolStatus = &Status(1 << 0);
    pub const FAIL: &'static EnumsProtocolStatus = &Status(1 << 1);
    pub const TIMEOUT: &'static EnumsProtocolStatus = &Status(1 << 1 | 1 << 2);
}