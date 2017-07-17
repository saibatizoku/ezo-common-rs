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

#[cfg(test)]
/// I2C command for the EZO chip.
pub trait Command {
    type Response;

    fn get_command_string (&self) -> String;
    fn get_delay (&self) -> u64;
    fn run(&self, dev: &mut LinuxI2CDevice) -> Result<Self::Response>;
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

impl BpsRate {
    /// Returns the BpsRate as a `u32` value.
    pub fn parse(&self) -> u32 {
        match *self {
            BpsRate::Bps300 => BpsRate::Bps300 as u32,
            BpsRate::Bps1200 => BpsRate::Bps1200 as u32,
            BpsRate::Bps2400 => BpsRate::Bps2400 as u32,
            BpsRate::Bps9600 => BpsRate::Bps9600 as u32,
            BpsRate::Bps19200 => BpsRate::Bps19200 as u32,
            BpsRate::Bps38400 => BpsRate::Bps38400 as u32,
            BpsRate::Bps57600 => BpsRate::Bps57600 as u32,
            BpsRate::Bps115200 => BpsRate::Bps115200 as u32,
        }
    }
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
fn turn_off_high_bits(v: &mut [u8]) {
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
///
/// After reading your buffer from the i2c device, check the first
/// byte for the response code.  Then, pass a slice with the rest of
/// the buffer (without that first byte) to this function to get an
/// UTF-8 string.
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

#[macro_export]
macro_rules! define_command_impl {
    ($name:ident, $response:ty, $command_string:block, $delay:expr, $run_func:expr) => {
        impl Command for $name {
            type Response = $response;

            fn get_command_string(&self) -> String {
                $command_string
            }

            fn get_delay(&self) -> u64 {
                $delay
            }

            fn run (&self, dev: &mut LinuxI2CDevice) -> Result<$response> {
                let cmd = self.get_command_string();
                let _w = write_to_ezo(dev, cmd.as_bytes())
                    .chain_err(|| "Error writing to EZO device.")?;
                let delay = self.get_delay();
                if delay > 0 {
                    thread::sleep(Duration::from_millis(delay));
                };
                $run_func
            }
        }
    };
    ($cmd:ident : $name:ident, $response:ty, $command_string:block, $delay:expr, $run_func:expr) => {
        impl Command for $name {
            type Response = $response;

            fn get_command_string(&self) -> String {
                let $cmd = &self.0;
                $command_string
            }

            fn get_delay(&self) -> u64 {
                $delay
            }

            fn run (&self, dev: &mut LinuxI2CDevice) -> Result<$response> {
                let cmd = self.get_command_string();
                let _w = write_to_ezo(dev, cmd.as_bytes())
                    .chain_err(|| "Error writing to EZO device.")?;
                let delay = self.get_delay();
                if delay > 0 {
                    thread::sleep(Duration::from_millis(delay));
                };
                $run_func
            }
        }
    };
}

#[macro_export]
macro_rules! define_command {
    (doc: $doc:tt, $name:ident, $command_string:block, $delay:expr) => {
        #[doc=$doc]
        pub struct $name;

        define_command_impl!($name, (), $command_string, $delay, Ok (()) );
    };
    (doc: $doc:tt, $name:ident, $command_string:block) => {
        #[doc=$doc]
        pub struct $name;

        define_command_impl!($name, (), $command_string, 0, Ok (()) );
    };
    (doc: $doc:tt, $name:ident, $command_string:block, $delay:expr, $response:ty, $run_func:expr) => {
        #[doc=$doc]
        pub struct $name;

        define_command_impl!($name, $response, $command_string, $delay, $run_func);
    };
    ($name:ident, $command_string:block, $delay:expr) => {
        pub struct $name;

        define_command_impl!($name, (), $command_string, $delay, Ok (()) );
    };
    ($name:ident, $command_string:block) => {
        pub struct $name;

        define_command_impl!($name, (), $command_string, 0, Ok (()) );
    };
    ($name:ident, $command_string:block, $delay:expr, $response:ty, $run_func:expr) => {
        pub struct $name;

        define_command_impl!($name, $response, $command_string, $delay, $run_func);
    };
    (doc: $doc:tt, $cmd:ident : $name:ident($data:ty), $command_string:block, $delay:expr) => {
        #[doc=$doc]
        pub struct $name(pub $data);

        define_command_impl!($cmd: $name, (), $command_string, $delay, Ok (()) );
    };
    (doc: $doc:tt, $cmd:ident : $name:ident($data:ty), $command_string:block, $delay:expr, $response:ty, $run_func:expr) => {
        #[doc=$doc]
        pub struct $name(pub $data);

        define_command_impl!($cmd: $name, $response, $command_string, $delay, $run_func);
    };
    ($cmd:ident : $name:ident($data:ty), $command_string:block, $delay:expr) => {
        pub struct $name(pub $data);

        define_command_impl!($cmd: $name, (), $command_string, $delay, Ok (()) );
    };
    ($cmd:ident : $name:ident($data:ty), $command_string:block, $delay:expr, $response:ty, $run_func:expr) => {
        pub struct $name(pub $data);

        define_command_impl!($cmd: $name, $response, $command_string, $delay, $run_func);
    };
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_baud_rates_to_numbers() {
        assert_eq!(BpsRate::Bps300.parse(), 300);
        assert_eq!(BpsRate::Bps1200.parse(), 1200);
        assert_eq!(BpsRate::Bps2400.parse(), 2400);
        assert_eq!(BpsRate::Bps9600.parse(), 9600);
        assert_eq!(BpsRate::Bps19200.parse(), 19200);
        assert_eq!(BpsRate::Bps38400.parse(), 38400);
        assert_eq!(BpsRate::Bps57600.parse(), 57600);
        assert_eq!(BpsRate::Bps115200.parse(), 115200);
    }

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
    #[test]
    fn defines_command_with_no_delay_no_response() {
        define_command! { NewCommand, { "NewCommand".to_string() } }
        assert_eq!(NewCommand.get_command_string(), "NewCommand");
    }
    #[test]
    fn defines_command_with_delay_no_response() {
        define_command! { NewCommand, { "NewCommand".to_string() }, 100 }
        assert_eq!(NewCommand.get_command_string(), "NewCommand");
        assert_eq!(NewCommand.get_delay(), 100);
    }
    #[test]
    fn defines_command_with_delay_with_response() {
        define_command! { NewCommand, { "NewCommand".to_string() }, 100, u32, Ok (0u32) }
        assert_eq!(NewCommand.get_command_string(), "NewCommand");
        assert_eq!(NewCommand.get_delay(), 100);
    }
    #[test]
    fn defines_command_with_no_delay_no_response_with_docs() {
        define_command! { doc: "docs", NewCommand, { "NewCommand".to_string() } }
        assert_eq!(NewCommand.get_command_string(), "NewCommand");
    }
    #[test]
    fn defines_command_with_delay_no_response_with_docs() {
        define_command! { doc: "docs", NewCommand, { "NewCommand".to_string() }, 100 }
        assert_eq!(NewCommand.get_command_string(), "NewCommand");
        assert_eq!(NewCommand.get_delay(), 100);
    }
    #[test]
    fn defines_command_with_delay_with_response_with_docs() {
        define_command! { doc: "docs", NewCommand, { "NewCommand".to_string() }, 100, u32, Ok (0u32) }
        assert_eq!(NewCommand.get_command_string(), "NewCommand");
        assert_eq!(NewCommand.get_delay(), 100);
    }

    #[test]
    fn defines_data_command_with_delay_no_response() {
        define_command! { cmd: NewCommand(u8), { format!("NewCommand,{}", cmd) }, 100 }
        assert_eq!(NewCommand(100).get_command_string(), "NewCommand,100");
        assert_eq!(NewCommand(100).get_delay(), 100);
    }
    #[test]
    fn defines_data_command_with_delay_with_response() {
        define_command! { cmd: NewCommand(u8), { format!("NewCommand,{}", cmd) }, 100 , u32, Ok (0u32) }
        assert_eq!(NewCommand(0).get_command_string(), "NewCommand,0");
        assert_eq!(NewCommand(0).get_delay(), 100);
    }
    #[test]
    fn defines_data_command_with_delay_no_response_with_docs() {
        define_command! { doc: "docs", cmd: NewCommand(u8), { format!("NewCommand,{}", cmd) }, 100 }
        assert_eq!(NewCommand(100).get_command_string(), "NewCommand,100");
        assert_eq!(NewCommand(100).get_delay(), 100);
    }
    #[test]
    fn defines_data_command_with_delay_with_response_with_docs() {
        define_command! { doc: "docs", cmd: NewCommand(u8), { format!("NewCommand,{}", cmd) }, 100 , u32, Ok (0u32) }
        assert_eq!(NewCommand(0).get_command_string(), "NewCommand,0");
        assert_eq!(NewCommand(0).get_delay(), 100);
    }
}
