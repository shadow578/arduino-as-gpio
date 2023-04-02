use super::{as_response_type, Error, Request};

//
// Error Response Constants
//
const TYPE_ERROR: u8 = 0x7f;

//const ERR_MALFORMED_PACKET: u8 = 0x01;
//const ERR_INVALID_TYPE: u8 = 0x02;
const ERR_INVALID_PIN: u8 = 0x03;

//
// Error Response Implementation
//
#[derive(Debug)]
pub struct ErrorRequest {}
impl ErrorRequest {
    pub fn new() -> ErrorRequest {
        ErrorRequest {}
    }
}

impl Request<Error> for ErrorRequest {
    fn get_packet_body(&self) -> Vec<u8> {
        panic!("ErrorRequest is parse only")
    }

    fn parse_response(&self, packet_body: &Vec<u8>) -> Result<Error, Error> {
        // ensure length is correct
        if packet_body.len() != 2 {
            return Err(Error::ResponseMismatch);
        }

        // ensure type is error response
        if packet_body[0] != as_response_type!(TYPE_ERROR) {
            return Err(Error::ResponseMismatch);
        }

        // check error code and create OK Error
        let error_code = packet_body[1];
        match error_code {
            ERR_INVALID_PIN => return Ok(Error::InvalidPin),
            _ => return Ok(Error::RemoteError { code: error_code }),
        }
    }
}
