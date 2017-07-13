//! Shared code for EZO sensor chips. These chips are used for sensing aquatic
//! media.

#![recursion_limit = "1024"]

#![feature(inclusive_range_syntax)]

#[macro_use]
extern crate error_chain;
extern crate i2cdev;

pub mod errors;

use errors::*;
use i2cdev::core::I2CDevice;
use i2cdev::linux::LinuxI2CDevice;
use std::ffi::CStr;
use std::thread;
use std::time::Duration;

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

/// Turns off the high bit in each of the bytes of `v`.  Raspberry Pi
/// for some reason outputs i2c buffers with some of the high bits
/// turned on.
pub fn turn_off_high_bits(v: &mut [u8]) {
    for b in v.iter_mut () {
        *b = *b & 0x7f;
    }
}

/// Converts a slice of bytes, as they come raw from the i2c buffer,
/// into an owned String.  Due to a hardware glitch in the Broadcom
/// I2C module, we need to strip off the high bit of each byte in the
/// response strings.
///
/// This function ensures that the response is a nul-terminated string
/// and that it is valid UTF-8 (a superset of ASCII).
pub fn string_from_response_data(response: &[u8]) -> Result<String> {
    let mut buf = response.to_owned ();
    turn_off_high_bits (&mut buf);

    let terminated = CStr::from_bytes_with_nul(&buf)
        .chain_err(|| ErrorKind::MalformedResponse)?;

    let s = terminated.to_str ()
        .chain_err(|| ErrorKind::MalformedResponse)?
        .to_owned ();

    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns_off_high_bits() {
        let data: [u8; 11] = [63, 73, 44, 112, 72, 44, 49, 46, 57, 56, 0];
        let mut flipped_data: [u8; 11] = [63, 73, 172, 112, 200, 172, 49, 46, 57, 56, 0];
        turn_off_high_bits(&mut flipped_data);
        assert_eq!(data, flipped_data);
    }

    #[test]
    fn converts_valid_response_to_string() {
        // empty nul-terminated string
        assert_eq!(string_from_response_data(&b"\0"[..]).unwrap(), "");

        // non-empty nul-terminated string
        assert_eq!(string_from_response_data(&b"hello\0"[..]).unwrap(), "hello");

        // high bit is on in the last character
        assert_eq!(string_from_response_data(&b"hell\xef\0"[..]).unwrap(), "hello");
    }

    fn assert_converts_to_malformed_response(data: &[u8]) {
        let result = string_from_response_data(&data);

        match result {
            Err(Error(ErrorKind::MalformedResponse, _)) => (),
            _ => unreachable!()
        }
    }

    #[test]
    fn converts_invalid_response_to_error() {
        // No nul terminator in either of these
        assert_converts_to_malformed_response(&b""[..]);
        assert_converts_to_malformed_response(&b"\xff"[..]);
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
    }
}
