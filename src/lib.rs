//! Shared code for EZO sensor chips. These chips are used for sensing aquatic
//! media.
//!
//! > Currently, only __I2C Mode__ is supported.

#![recursion_limit = "1024"]

#![feature(inclusive_range_syntax)]

#[macro_use]
extern crate error_chain;
extern crate i2cdev;

pub mod errors;

use errors::*;
use i2cdev::core::I2CDevice;
use i2cdev::linux::LinuxI2CDevice;
use std::ffi::{CStr, CString};
use std::thread;
use std::time::Duration;

/// Default buffer size for ASCII data responses.
///
/// Implement your own version of MAX_DATA wherever you are implementing
/// the `define_command!` macro, to override.
pub const MAX_DATA: usize = 42;

/// I2C command for the EZO chip.
pub trait Command {
    type Response;

    fn get_command_string(&self) -> String;
    fn get_delay(&self) -> u64;
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
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// Returns the `BpsRate` from a `u32` value.
    pub fn from_num(bps_rate: u32) -> Result<BpsRate> {
        let bps = match bps_rate {
            x if x == BpsRate::Bps300 as u32 => BpsRate::Bps300,
            x if x == BpsRate::Bps1200 as u32 => BpsRate::Bps1200,
            x if x == BpsRate::Bps2400 as u32 => BpsRate::Bps2400,
            x if x == BpsRate::Bps9600 as u32 => BpsRate::Bps9600,
            x if x == BpsRate::Bps19200 as u32 => BpsRate::Bps19200,
            x if x == BpsRate::Bps38400 as u32 => BpsRate::Bps38400,
            x if x == BpsRate::Bps57600 as u32 => BpsRate::Bps57600,
            x if x == BpsRate::Bps115200 as u32 => BpsRate::Bps115200,
            _ => return Err(ErrorKind::BpsRateParse.into()),
        };
        Ok(bps)
    }
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
pub fn write_to_ezo(dev: &mut LinuxI2CDevice, cmd_str: &str) -> Result<()> {
    let cmd = CString::new(cmd_str)
        .chain_err(|| "Command cannot be used")?;
    if let Err(_) = dev.write(cmd.as_bytes_with_nul()) {
        thread::sleep(Duration::from_millis(100));
        dev.write(cmd.as_bytes_with_nul()).chain_err(|| "Command could not be sent")?;
    };
    Ok(())
}

/// Turns off the high bit in each of the bytes of `v`.  Raspberry Pi
/// for some reason outputs i2c buffers with some of the high bits
/// turned on.
fn turn_off_high_bits(v: &mut [u8]) {
    for b in v.iter_mut() {
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
    let mut buf = response.to_owned();
    turn_off_high_bits(&mut buf);

    let terminated = CStr::from_bytes_with_nul(&buf)
        .chain_err(|| ErrorKind::MalformedResponse)?;

    let s = terminated
        .to_str()
        .chain_err(|| ErrorKind::MalformedResponse)?
        .to_owned();

    Ok(s)
}

/// Implements `fn run(dev: &mut LinuxI2CDevice) -> Result<$response>` for
/// `define_command_impl!`.
#[macro_export]
macro_rules! command_run_fn {
    (Ack) => {
        fn run (&self, dev: &mut LinuxI2CDevice) -> Result<()> {
            let cmd = self.get_command_string();

            let _w = write_to_ezo(dev, &cmd)
                .chain_err(|| "Error writing to EZO device.")?;

            let delay = self.get_delay();

            if delay > 0 {
                thread::sleep(Duration::from_millis(delay));
            };

            let mut data_buffer = [0u8; MAX_DATA];

            let _r = dev.read(&mut data_buffer)
                .chain_err(|| ErrorKind::I2CRead)?;

            match response_code(data_buffer[0]) {
                ResponseCode::Success => Ok(()),

                ResponseCode::Pending => Err(ErrorKind::PendingResponse.into()),

                ResponseCode::DeviceError => Err(ErrorKind::DeviceErrorResponse.into()),

                ResponseCode::NoDataExpected => Err(ErrorKind::NoDataExpectedResponse.into()),

                ResponseCode::UnknownError => Err(ErrorKind::MalformedResponse.into()),
            }
        }
    };
    (NoAck) => {
        fn run (&self, dev: &mut LinuxI2CDevice) -> Result<()> {
            let cmd = self.get_command_string();

            let _w = write_to_ezo(dev, &cmd)
                .chain_err(|| "Error writing to EZO device.")?;

            let delay = self.get_delay();

            if delay > 0 {
                thread::sleep(Duration::from_millis(delay));
            };

            Ok (())
        }
    };
    ($resp:ident : $response:ty, $run_func:block) => {
        fn run (&self, dev: &mut LinuxI2CDevice) -> Result<$response> {
            let cmd = self.get_command_string();

            let _w = write_to_ezo(dev, &cmd)
                .chain_err(|| "Error writing to EZO device.")?;

            let delay = self.get_delay();

            if delay > 0 {
                thread::sleep(Duration::from_millis(delay));
            };

            let mut data_buffer = [0u8; MAX_DATA];

            let _r = dev.read(&mut data_buffer)
                .chain_err(|| ErrorKind::I2CRead)?;

            let resp_string = match response_code(data_buffer[0]) {
                ResponseCode::Success => {
                    match data_buffer.iter().position(|&c| c == 0) {
                        Some(len) => {
                            string_from_response_data(&data_buffer[1...len])
                                .chain_err(|| ErrorKind::MalformedResponse)
                        }
                        _ => return Err(ErrorKind::MalformedResponse.into()),
                    }
                }

                ResponseCode::Pending => return Err(ErrorKind::PendingResponse.into()),

                ResponseCode::DeviceError => return Err(ErrorKind::DeviceErrorResponse.into()),

                ResponseCode::NoDataExpected => return Err(ErrorKind::NoDataExpectedResponse.into()),

                ResponseCode::UnknownError => return Err(ErrorKind::MalformedResponse.into()),
            };
            let $resp = resp_string?;
            $run_func
        }
    };
}

/// Short-hand for writing valid `impl` of commands
///
/// Implement your own version of `MAX_DATA` wherever you are implementing
/// the `define_command!` macro, to override.
///
/// Implement your own version of `trait Command`  wherever you are implementing
/// the `define_command!` macro, to override.
#[macro_export]
macro_rules! define_command_impl {
    ($name:ident, $command_string:block, $delay:expr) => {
        impl Command for $name {
            type Response = ();

            fn get_command_string(&self) -> String {
                $command_string
            }

            fn get_delay(&self) -> u64 {
                $delay
            }

            command_run_fn! { NoAck }
        }
    };
    ($cmd:ident : $name:ident($data:ty), $command_string:block, $delay:expr) => {
        impl Command for $name {
            type Response = ();

            fn get_command_string(&self) -> String {
                let $cmd = &self.0;
                $command_string
            }

            fn get_delay(&self) -> u64 {
                $delay
            }

            command_run_fn! { NoAck }
        }
    };
    ($name:ident, $command_string:block, $delay:expr, Ack) => {
        impl Command for $name {
            type Response = ();

            fn get_command_string(&self) -> String {
                $command_string
            }

            fn get_delay(&self) -> u64 {
                $delay
            }

            command_run_fn! { Ack }
        }
    };
    ($cmd:ident : $name:ident($data:ty), $command_string:block, $delay:expr, Ack) => {
        impl Command for $name {
            type Response = ();

            fn get_command_string(&self) -> String {
                let $cmd = &self.0;
                $command_string
            }

            fn get_delay(&self) -> u64 {
                $delay
            }

            command_run_fn! { Ack }
        }
    };
    ($name:ident, $command_string:block, $delay:expr,
     $resp:ident : $response:ty, $run_func:block) => {
        impl Command for $name {
            type Response = $response;

            fn get_command_string(&self) -> String {
                $command_string
            }

            fn get_delay(&self) -> u64 {
                $delay
            }

            command_run_fn! { $resp: $response, $run_func }
        }
    };
    ($cmd:ident : $name:ident($data:ty), $command_string:block, $delay:expr,
     $resp:ident : $response:ty, $run_func:block) => {
        impl Command for $name {
            type Response = $response;

            fn get_command_string(&self) -> String {
                let $cmd = &self.0;
                $command_string
            }

            fn get_delay(&self) -> u64 {
                $delay
            }

            command_run_fn! { $resp: $response, $run_func }
        }
    };
}

/// Short-hand for writing valid commands
///
/// Implement your own version of `MAX_DATA` wherever you are implementing
/// the `define_command!` macro, to override.
///
/// Implement your own version of `trait Command`  wherever you are implementing
/// the `define_command!` macro, to override.
///
/// ## Examples
///
/// ### COMMANDS WITH DOCS
///
/// A typical preable includes:
///
/// ```text
/// # #[macro_use] extern crate ezo_common;
/// # extern crate error_chain;
/// # extern crate i2cdev;
/// # use std::thread;
/// # use std::time::Duration;
/// # use i2cdev::linux::LinuxI2CDevice;
/// # use ezo_common::{MAX_DATA, Command, write_to_ezo};
/// # use ezo_common::errors::*;
/// ```
#[macro_export]
macro_rules! define_command {
    // DOCUMENTED COMMANDS
    // ===================
    // {
    //   doc: "docstring",
    //   Name, cmd_string_block, delay
    // }
    (doc : $doc:tt,
     $name:ident, $command_string:block, $delay:expr) => {
        #[ doc = $doc ]
        pub struct $name;

        define_command_impl!($name, $command_string, $delay);
    };

    // {
    //   doc: "docstring",
    //   Name, cmd_string_block, delay, Ack
    // }
    (doc : $doc:tt,
     $name:ident, $command_string:block, $delay:expr, Ack) => {
        #[ doc = $doc ]
        pub struct $name;

        define_command_impl!($name, $command_string, $delay, Ack);
    };

    // {
    //   doc: "docstring",
    //   Name, cmd_string_block, delay,
    //   _data: ResponseType, resp_expr
    // }
    (doc : $doc:tt,
     $name:ident, $command_string:block, $delay:expr,
     $resp:ident : $response:ty, $run_func:block) => {
        #[ doc = $doc ]
        pub struct $name;

        define_command_impl! {
            $name, $command_string, $delay,
            $resp: $response, $run_func
        }
    };

    // {
    //   doc: "docstring",
    //   cmd: Name(type), cmd_string_block, delay
    // }
    (doc : $doc:tt,
     $cmd:ident : $name:ident($data:ty), $command_string:block, $delay:expr) => {
        #[ doc = $doc ]
        pub struct $name(pub $data);

        define_command_impl! {
            $cmd: $name($data), $command_string, $delay
        }
    };

    // {
    //   doc: "docstring",
    //   cmd: Name(type), cmd_string_block, delay, Ack
    // }
    (doc : $doc:tt,
     $cmd:ident : $name:ident($data:ty), $command_string:block, $delay:expr, Ack) => {
        #[ doc = $doc ]
        pub struct $name(pub $data);

        define_command_impl! {
            $cmd: $name($data), $command_string, $delay, Ack
        }
    };

    // {
    //   doc: "docstring",
    //   cmd: Name(type), cmd_string_block, delay,
    //   _data: ResponseType, resp_expr
    // }
    (doc : $doc:tt,
     $cmd:ident : $name:ident($data:ty), $command_string:block, $delay:expr,
     $resp:ident : $response:ty, $run_func:block) => {
        #[ doc = $doc ]
        pub struct $name(pub $data);

        define_command_impl! {
            $cmd: $name($data), $command_string, $delay,
            $resp: $response, $run_func
        }
    };

    // NOTE: We need to remove this duplication
    // UNDOCUMENTED COMMANDS
    // ===================
    // {
    //   Name, cmd_string_block, delay
    // }
    ($name:ident, $command_string:block, $delay:expr) => {
        pub struct $name;

        define_command_impl!($name, $command_string, $delay);
    };

    // {
    //   Name, cmd_string_block, delay, Ack
    // }
    ($name:ident, $command_string:block, $delay:expr, Ack) => {
        pub struct $name;

        define_command_impl!($name, $command_string, $delay, Ack);
    };

    // {
    //   Name, cmd_string_block, delay,
    //   _data: ResponseType, resp_expr
    // }
    ($name:ident, $command_string:block, $delay:expr,
     $resp:ident : $response:ty, $run_func:block) => {
        pub struct $name;

        define_command_impl! {
            $name, $command_string, $delay,
            $resp: $response, $run_func
        }
    };

    // {
    //   cmd: Name(type), cmd_string_block, delay
    // }
    ($cmd:ident : $name:ident($data:ty), $command_string:block, $delay:expr) => {
        pub struct $name(pub $data);

        define_command_impl! {
            $cmd: $name($data), $command_string, $delay
        }
    };

    // {
    //   cmd: Name(type), cmd_string_block, delay, Ack
    // }
    ($cmd:ident : $name:ident($data:ty), $command_string:block, $delay:expr, Ack) => {
        pub struct $name(pub $data);

        define_command_impl! {
            $cmd: $name($data), $command_string, $delay, Ack
        }
    };

    // {
    //   cmd: Name(type), cmd_string_block, delay,
    //   _data: ResponseType, resp_expr
    // }
    ($cmd:ident : $name:ident($data:ty), $command_string:block, $delay:expr,
     $resp:ident : $response:ty, $run_func:block) => {
        pub struct $name(pub $data);

        define_command_impl! {
            $cmd: $name($data), $command_string, $delay,
            $resp: $response, $run_func
        }
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
        assert_eq!(string_from_response_data(&b"hell\xef\0"[..]).unwrap(),
                   "hello");
    }

    fn assert_converts_to_malformed_response(data: &[u8]) {
        let result = string_from_response_data(&data);

        match result {
            Err(Error(ErrorKind::MalformedResponse, _)) => (),
            _ => unreachable!(),
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
    fn macro_creates_impl_for_noack_simple_command() {
        pub struct ControlCommand;

        define_command_impl! {
            ControlCommand, { "cmd".to_string() }, 0
        }
        assert_eq!(ControlCommand.get_command_string(), "cmd");
        assert_eq!(ControlCommand.get_delay(), 0);
    }

    #[test]
    fn macro_creates_impl_for_noack_input_command() {
        pub struct InputCommand(u32);

        define_command_impl! {
            cmd: InputCommand(u32), { format!("cmd,{}", cmd) }, 0
        }
        assert_eq!(InputCommand(43).get_command_string(), "cmd,43");
        assert_eq!(InputCommand(43).get_delay(), 0);
    }

    #[test]
    fn macro_creates_impl_for_noack_simple_command_with_response() {
        pub struct ControlCommand;

        define_command_impl! {
            ControlCommand, { "cmd".to_string() }, 0,
            _resp: u32, { Ok (0u32) }
        }
        assert_eq!(ControlCommand.get_command_string(), "cmd");
        assert_eq!(ControlCommand.get_delay(), 0);
    }

    #[test]
    fn macro_creates_impl_for_noack_input_command_with_response() {
        pub struct InputCommand(u32);

        define_command_impl! {
            cmd: InputCommand(u32), { format!("cmd,{}", cmd) }, 0,
            _resp: (), { Ok ( () ) }
        }
        assert_eq!(InputCommand(43).get_command_string(), "cmd,43");
        assert_eq!(InputCommand(43).get_delay(), 0);
    }

    #[test]
    fn macro_creates_impl_for_ack_simple_command() {
        pub struct ControlCommand;

        define_command_impl! {
            ControlCommand, { "cmd".to_string() }, 0, Ack
        }
        assert_eq!(ControlCommand.get_command_string(), "cmd");
        assert_eq!(ControlCommand.get_delay(), 0);
    }

    #[test]
    fn macro_creates_impl_for_ack_input_command() {
        pub struct InputCommand(u32);

        define_command_impl! {
            cmd: InputCommand(u32), { format!("cmd,{}", cmd) }, 0, Ack
        }
        assert_eq!(InputCommand(43).get_command_string(), "cmd,43");
        assert_eq!(InputCommand(43).get_delay(), 0);
    }

    #[test]
    fn macro_creates_noack_simple_command() {
        define_command! {
            ControlCommand, { "cmd".to_string() }, 1000
        }
        assert_eq!(ControlCommand.get_command_string(), "cmd");
        assert_eq!(ControlCommand.get_delay(), 1000);
    }

    #[test]
    fn macro_creates_noack_input_command() {
        define_command! {
            cmd: InputCommand(f32), { format!("cmd,{:.*}", 2, cmd) }, 0
        }
        assert_eq!(InputCommand(3.285).get_command_string(), "cmd,3.29");
        assert_eq!(InputCommand(3.285).get_delay(), 0);
    }

    #[test]
    fn macro_creates_ack_simple_command() {
        define_command! {
            ControlCommand, { "cmd".to_string() }, 1000, Ack
        }
        assert_eq!(ControlCommand.get_command_string(), "cmd");
        assert_eq!(ControlCommand.get_delay(), 1000);
    }

    #[test]
    fn macro_creates_ack_input_command() {
        define_command! {
            cmd: InputCommand(f32), { format!("cmd,{:.*}", 2, cmd) }, 0, Ack
        }
        assert_eq!(InputCommand(3.285).get_command_string(), "cmd,3.29");
        assert_eq!(InputCommand(3.285).get_delay(), 0);
    }

    #[test]
    fn macro_creates_simple_command_with_response() {
        define_command! {
            ControlCommand, { "cmd".to_string() }, 1000,
            _data: u32, { Ok (0u32) }
        }
        assert_eq!(ControlCommand.get_command_string(), "cmd");
        assert_eq!(ControlCommand.get_delay(), 1000);
    }

    #[test]
    fn macro_creates_input_command_with_response() {
        define_command! {
            cmd: InputCommand(u8), { format!("cmd,{}", cmd) }, 140,
            _data: (), { Ok (()) }
        }
        assert_eq!(InputCommand(0x7F).get_command_string(), "cmd,127");
        assert_eq!(InputCommand(0x7F).get_delay(), 140);
    }

    #[test]
    fn macro_creates_noack_simple_command_with_docs() {
        define_command! {
            doc: "docstring here",
            ControlCommand, { "cmd".to_string() }, 1000
        }
        assert_eq!(ControlCommand.get_command_string(), "cmd");
        assert_eq!(ControlCommand.get_delay(), 1000);
    }

    #[test]
    fn macro_creates_noack_input_command_with_docs() {
        define_command! {
            doc: "docstring here",
            cmd: InputCommand(f32), { format!("cmd,{:.*}", 2, cmd) }, 0
        }
        assert_eq!(InputCommand(3.285).get_command_string(), "cmd,3.29");
        assert_eq!(InputCommand(3.285).get_delay(), 0);
    }

    #[test]
    fn macro_creates_ack_simple_command_with_docs() {
        define_command! {
            doc: "docstring here",
            ControlCommand, { "cmd".to_string() }, 1000, Ack
        }
        assert_eq!(ControlCommand.get_command_string(), "cmd");
        assert_eq!(ControlCommand.get_delay(), 1000);
    }

    #[test]
    fn macro_creates_ack_input_command_with_docs() {
        define_command! {
            doc: "docstring here",
            cmd: InputCommand(f32), { format!("cmd,{:.*}", 2, cmd) }, 0, Ack
        }
        assert_eq!(InputCommand(3.285).get_command_string(), "cmd,3.29");
        assert_eq!(InputCommand(3.285).get_delay(), 0);
    }

    #[test]
    fn macro_creates_simple_command_with_response_with_docs() {
        define_command! {
            doc: "docstring here",
            ControlCommand, { "cmd".to_string() }, 1000,
            _data: u32, { Ok (0u32) }
        }
        assert_eq!(ControlCommand.get_command_string(), "cmd");
        assert_eq!(ControlCommand.get_delay(), 1000);
    }

    #[test]
    fn macro_creates_input_command_with_response_with_docs() {
        define_command! {
            doc: "docstring here",
            cmd: InputCommand(u8), { format!("cmd,{}\0", cmd) }, 140,
            _data: (), { Ok (()) }
        }
        assert_eq!(InputCommand(0x7F).get_command_string(), "cmd,127\0");
        assert_eq!(InputCommand(0x7F).get_delay(), 140);
    }
}
