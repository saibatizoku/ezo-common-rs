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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
