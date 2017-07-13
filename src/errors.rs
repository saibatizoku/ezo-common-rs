error_chain! {
    errors {
        // The response is not nul-terminated, or it is not valid ASCII/UTF-8
        MalformedResponse {
            description ("malformed response")
            display ("response is not a valid nul-terminated UTF-8 string")
        }
    }
}
