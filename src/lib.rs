//! Shared code for EZO sensor chips. These chips are used for sensing aquatic
//! media.

#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;

/// Use error-chain for error-handling.
pub mod errors {
    error_chain!{}
}

/// Allowable baudrates used when changing the chip to UART mode.
#[derive(Debug)]
pub enum BpsRate {
    Bps300 = 300,
    Bps1200 = 1200,
    Bps2400 = 2400,
    Bps9600 = 9600,
    Bps19200 = 19200,
    Bps38400 = 38400,
    Bps57600 = 57600,
    Bps115200 = 115200,
}

/// Known response codes from EZO chip interactions.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum ResponseCode {
    NoDataExpected = 0xFF,
    Pending = 0xFE,
    DeviceError = 0x02,
    Success = 0x01,
    UnknownError = 0x00, // This code is NOT implemented by the EZO chips
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn process_no_data_response_code() {
        assert_eq!(response_code(255), ResponseCode::NoDataExpected);
    }
    #[test]
    fn process_pending_response_code() {
        assert_eq!(response_code(254), ResponseCode::Pending);
    }
    #[test]
    fn process_error_response_code() {
        assert_eq!(response_code(2), ResponseCode::DeviceError);
    }
    #[test]
    fn process_success_response_code() {
        assert_eq!(response_code(1), ResponseCode::Success);
    }
    #[test]
    fn process_unknown_response_code() {
        assert_eq!(response_code(0), ResponseCode::UnknownError);
        assert_eq!(response_code(16), ResponseCode::UnknownError);
        assert_eq!(response_code(156), ResponseCode::UnknownError);
        assert_eq!(response_code(256), ResponseCode::UnknownError);
    }
    #[test]
    fn parsing_nonzeros_response() {
        let data: [u8; 0] = [];
        let parsed = parse_data_ascii_bytes(&data);
        assert_eq!(parsed.len(), 0);
        let data: [u8; 6] = [0, 98, 99, 65, 66, 67];
        let parsed = parse_data_ascii_bytes(&data);
        assert_eq!(parsed.len(), 0);
        let data: [u8; 6] = [97, 98, 0, 65, 66, 67];
        let parsed = parse_data_ascii_bytes(&data);
        assert_eq!(parsed.len(), 2);
        let data: [u8; 6] = [97, 98, 99, 65, 66, 67];
        let parsed = parse_data_ascii_bytes(&data);
        assert_eq!(parsed.len(), 6);
    }
    #[test]
    fn parsing_abc_response() {
        let data: [u8; 6] = [97, 98, 99, 65, 66, 67];
        let parsed = String::from_utf8(parse_data_ascii_bytes(&data)).unwrap();
        assert_eq!(&parsed, "abcABC");
    }
    #[test]
    fn parsing_empty_response() {
        let data: [u8; 3] = [0, 0, 0];
        let parsed = String::from_utf8(parse_data_ascii_bytes(&data)).unwrap();
        assert_eq!(&parsed, "");
    }
    #[test]
    fn parsing_non_flipped_data_response() {
        let data: [u8; 11] = [63, 73, 44, 112, 72, 44, 49, 46, 57, 56, 0];
        let parsed = String::from_utf8(parse_data_ascii_bytes(&data)).unwrap();
        assert_eq!(&parsed, "?I,pH,1.98");
    }
    #[test]
    fn parsing_flipped_data_response() {
        let data: [u8; 11] = [63, 73, 172, 112, 200, 172, 49, 46, 57, 56, 0];
        let parsed = String::from_utf8(parse_data_ascii_bytes(&data)).unwrap();
        assert_eq!(&parsed, "?I,pH,1.98");
    }
}
