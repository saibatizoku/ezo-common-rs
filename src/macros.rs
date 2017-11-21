/// The most common `fn run` implementation.
#[macro_export]
macro_rules! command_run_fn_common {
    ( $self:ident, $dev:ident ) => {
        let cmd = $self.get_command_string();

        let _w = write_to_ezo($dev, &cmd)
            .chain_err(|| "Error writing to EZO device.")?;

        let delay = $self.get_delay();

        if delay > 0 {
            thread::sleep(Duration::from_millis(delay));
        };
    };
}

/// Implements `fn run(dev: &mut LinuxI2CDevice) -> Result<$response>` for
/// `define_command_impl!`.
#[macro_export]
macro_rules! command_run_fn {
    (Ack) => {
        fn run (&self, dev: &mut LinuxI2CDevice) -> ::std::result::Result<ResponseStatus, Self::Error> {

            command_run_fn_common!(self, dev);

            let mut data_buffer = [0u8; MAX_DATA];

            let _r = dev.read(&mut data_buffer)
                .chain_err(|| ErrorKind::I2CRead)?;

            match response_code(data_buffer[0]) {
                ResponseCode::Success => Ok(ResponseStatus::Ack),

                ResponseCode::Pending => Err(ErrorKind::PendingResponse.into()),

                ResponseCode::DeviceError => Err(ErrorKind::DeviceErrorResponse.into()),

                ResponseCode::NoDataExpected => Err(ErrorKind::NoDataExpectedResponse.into()),

                ResponseCode::UnknownError => Err(ErrorKind::MalformedResponse.into()),
            }
        }
    };
    (NoAck) => {
        fn run (&self, dev: &mut LinuxI2CDevice) -> ::std::result::Result<ResponseStatus, Self::Error> {

            command_run_fn_common!(self, dev);

            Ok (ResponseStatus::None)
        }
    };
    ($resp:ident : $response:ty, $run_func:block) => {
        fn run (&self, dev: &mut LinuxI2CDevice) -> ::std::result::Result<$response, Self::Error> {

            command_run_fn_common!(self, dev);

            let mut data_buffer = [0u8; MAX_DATA];

            let _r = dev.read(&mut data_buffer)
                .chain_err(|| ErrorKind::I2CRead)?;

            let resp_string = match response_code(data_buffer[0]) {
                ResponseCode::Success => {
                    match data_buffer.iter().position(|&c| c == 0) {
                        Some(len) => {
                            string_from_response_data(&data_buffer[1..=len])
                                .chain_err(|| ErrorKind::MalformedResponse)
                        }
                        _ => bail!(ErrorKind::MalformedResponse),
                    }
                }

                ResponseCode::Pending => bail!(ErrorKind::PendingResponse),

                ResponseCode::DeviceError => bail!(ErrorKind::DeviceErrorResponse),

                ResponseCode::NoDataExpected => bail!(ErrorKind::NoDataExpectedResponse),

                ResponseCode::UnknownError => bail!(ErrorKind::MalformedResponse),
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
            type Error = super::errors::Error;
            type Response = ResponseStatus;

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
            type Error = super::errors::Error;
            type Response = ResponseStatus;

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
            type Error = super::errors::Error;
            type Response = ResponseStatus;

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
            type Error = super::errors::Error;
            type Response = ResponseStatus;

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
            type Error = super::errors::Error;
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
            type Error = super::errors::Error;
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
        #[derive(Debug, PartialEq)]
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
        #[derive(Debug, PartialEq)]
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
        #[derive(Debug, PartialEq)]
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
        #[derive(Debug, PartialEq)]
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
        #[derive(Debug, PartialEq)]
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
        #[derive(Debug, PartialEq)]
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
        #[derive(Debug, PartialEq)]
        pub struct $name;

        define_command_impl!($name, $command_string, $delay);
    };

    // {
    //   Name, cmd_string_block, delay, Ack
    // }
    ($name:ident, $command_string:block, $delay:expr, Ack) => {
        #[derive(Debug, PartialEq)]
        pub struct $name;

        define_command_impl!($name, $command_string, $delay, Ack);
    };

    // {
    //   Name, cmd_string_block, delay,
    //   _data: ResponseType, resp_expr
    // }
    ($name:ident, $command_string:block, $delay:expr,
     $resp:ident : $response:ty, $run_func:block) => {
        #[derive(Debug, PartialEq)]
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
        #[derive(Debug, PartialEq)]
        pub struct $name(pub $data);

        define_command_impl! {
            $cmd: $name($data), $command_string, $delay
        }
    };

    // {
    //   cmd: Name(type), cmd_string_block, delay, Ack
    // }
    ($cmd:ident : $name:ident($data:ty), $command_string:block, $delay:expr, Ack) => {
        #[derive(Debug, PartialEq)]
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
        #[derive(Debug, PartialEq)]
        pub struct $name(pub $data);

        define_command_impl! {
            $cmd: $name($data), $command_string, $delay,
            $resp: $response, $run_func
        }
    };
}
