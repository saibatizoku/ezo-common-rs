//! Create the Error, ErrorKind, ResultExt, and Result types.
error_chain! {
    errors {
        // The unsuccessful response code
        I2CRead {
            description ("unsuccessful device read")
            display ("response was not obtainable")
        }
        BaudParse {
            description ("could not parse Baud command")
        }
        BpsRateParse {
            description ("could not parse bps rate")
        }
        CommandParse {
            description ("command parse failed")
        }
        // The response is not nul-terminated, or it is not valid ASCII/UTF-8
        MalformedResponse {
            description ("malformed response")
            display ("response is not a valid nul-terminated UTF-8 string")
        }
        // The response could not be parsed
        ResponseParse {
            description ("could not parse response")
        }
        // Any response code that is `Pending`
        PendingResponse {
            description ("response is pending")
            display ("response was not yet available")
        }
        // Any response code that is `DeviceError`
        DeviceErrorResponse {
            description ("response is error")
            display ("the device responded with an error")
        }
        // Any response code that is `NoDataExpected`
        NoDataExpectedResponse {
            description ("no data was expected")
            display ("the device has no data to respond")
        }
    }
}
