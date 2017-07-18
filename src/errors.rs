//! Create the Error, ErrorKind, ResultExt, and Result types.
error_chain! {
    errors {
        // The unsuccessful response code
        I2CRead {
            description ("unsuccessful device read")
            display ("response was not obtainable")
        }
        // The unsuccessful response code
        UnsuccessfulResponse {
            description ("unsuccessful response code")
            display ("response code was not successful")
        }
        // The response is not nul-terminated, or it is not valid ASCII/UTF-8
        MalformedResponse {
            description ("malformed response")
            display ("response is not a valid nul-terminated UTF-8 string")
        }
    }
}
