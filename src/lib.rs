//! Shared code for EZO sensor chips. These chips are used for sensing aquatic
//! media.

#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;

/// Use error-chain for error-handling.
pub mod errors {
    error_chain!{}
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
