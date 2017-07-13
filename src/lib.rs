//! Shared code for EZO sensor chips. These chips are used for sensing aquatic
//! media.

#![recursion_limit = "1024"]

#![feature(inclusive_range_syntax)]

#[macro_use]
extern crate error_chain;
extern crate i2cdev;

/// Use error-chain for error-handling.
pub mod errors {
    error_chain!{}
}

use errors::*;
use i2cdev::core::I2CDevice;
use i2cdev::linux::LinuxI2CDevice;
use std::ascii::AsciiExt;
use std::thread;
use std::time::Duration;

/// Crude parser for the data string sent by the EZO chip.
pub fn parse_data_ascii_bytes(data_buffer: &[u8]) -> Vec<u8> {
    match data_buffer.iter().position(|&x| x == 0) {
        Some(len) => read_hardware_buffer(&data_buffer[...len]),
        _ => read_hardware_buffer(&data_buffer[..]),
    }
}

/// Read byte buffer from the hardware.
pub fn read_hardware_buffer(response: &[u8]) -> Vec<u8> {
    if !response.is_ascii() {
        response.iter().map(|buf| (*buf & !0x80)).collect()
    } else {
        Vec::from(&response[..])
    }
}

/// Determines the response code sent by the EZO chip.
pub fn response_code(code_byte: u8) -> ResponseCode {
    use self::ResponseCode::*;
    match code_byte {
        x if x == NoDataExpected as u8 => NoDataExpected,
        x if x == Pending as u8 => Pending,
        x if x == DeviceError as u8 => DeviceError,
        x if x == Success as u8 => Success,
        _ => UnknownError,
    }
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

/// Writes the ASCII command to the EZO chip, with one retry.
pub fn write_to_ezo(dev: &mut LinuxI2CDevice, cmd: &[u8]) -> Result<()> {
    if let Err(_) = dev.write(cmd) {
        thread::sleep(Duration::from_millis(100));
        dev.write(cmd)
            .chain_err(|| "Command could not be sent")?;
    };
    Ok(())
}

/// Read the buffered response from the EZO chip.
pub fn read_raw_buffer(dev: &mut LinuxI2CDevice, max_data: usize) -> Result<Vec<u8>> {
    let mut data_buffer = vec![0u8; max_data];
    dev.read(&mut data_buffer)
        .chain_err(|| "Error reading from device")?;
    Ok(data_buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn determine_if_response_is_ascii() {
        let data: [u8; 11] = [63, 73, 44, 112, 72, 44, 49, 46, 57, 56, 0];
        let flipped_data: [u8; 11] = [63, 73, 172, 112, 200, 172, 49, 46, 57, 56, 0];
        assert_eq!(data.is_ascii(), true);
        assert_eq!(flipped_data.is_ascii(), false);
    }
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
        assert_eq!(parsed.len(), 1);
        let data: [u8; 6] = [97, 98, 0, 65, 66, 67];
        let parsed = parse_data_ascii_bytes(&data);
        assert_eq!(parsed.len(), 3);
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
        assert_eq!(&parsed, "\0");
    }
    #[test]
    fn parsing_non_flipped_data_response() {
        let data: [u8; 11] = [63, 73, 44, 112, 72, 44, 49, 46, 57, 56, 0];
        let parsed = String::from_utf8(parse_data_ascii_bytes(&data)).unwrap();
        assert_eq!(&parsed, "?I,pH,1.98\0");
    }
    #[test]
    fn parsing_flipped_data_response() {
        let data: [u8; 11] = [63, 73, 172, 112, 200, 172, 49, 46, 57, 56, 0];
        let parsed = String::from_utf8(parse_data_ascii_bytes(&data)).unwrap();
        assert_eq!(&parsed, "?I,pH,1.98\0");
    }
}
