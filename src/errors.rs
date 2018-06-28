//! Errors for EZO sensor chips.
use failure::{Backtrace, Context, Fail};
use std::fmt::{self, Display};

#[derive(Debug)]
pub struct EzoError {
    inner: Context<ErrorKind>,
}

#[derive(Copy, Clone, Eq, Debug, Fail, PartialEq)]
pub enum ErrorKind {
    #[fail(display = "could not parse Baud command")]
    BaudParse,
    #[fail(display = "could not parse bps rate")]
    BpsRateParse,
    #[fail(display = "command parse failed")]
    CommandParse,
    #[fail(display = "the device responded with an error")]
    DeviceErrorResponse,
    #[fail(display = "response was not obtainable")]
    I2CRead,
    #[fail(display = "response is not a valid nul-terminated UTF-8 string")]
    MalformedResponse,
    #[fail(display = "the device has no data to respond")]
    NoDataExpectedResponse,
    #[fail(display = "response was not yet available")]
    PendingResponse,
    #[fail(display = "could not parse response")]
    ResponseParse,
    #[fail(display = "Command could not be read")]
    UnreadableCommand,
    #[fail(display = "Command could not be written to I2C device")]
    UnwritableCommand,
}

impl Fail for EzoError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for EzoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl EzoError {
    pub fn kind(&self) -> ErrorKind {
        *self.inner.get_context()
    }
}

impl From<ErrorKind> for EzoError {
    fn from(kind: ErrorKind) -> EzoError {
        EzoError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for EzoError {
    fn from(inner: Context<ErrorKind>) -> EzoError {
        EzoError { inner: inner }
    }
}
