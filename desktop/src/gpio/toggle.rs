use super::{Error, Request};

//
// Toggle Request Constants
//
const TYPE_TOGGLE_REQUEST: u8 = 0x06;
const TYPE_TOGGLE_RESPONSE: u8 = 0x07;

//
// Toggle Request Implementation
//
#[derive(Debug)]
pub struct ToggleRequest {
    pub pin: u8,
}

#[derive(Debug)]
pub struct ToggleResponse {
    pub new_value: u8,
}

impl ToggleRequest {
    pub fn new(pin: u8) -> ToggleRequest {
        ToggleRequest { pin }
    }
}

impl Request<ToggleResponse> for ToggleRequest {
    fn get_packet_body(&self) -> Vec<u8> {
        // assemble packet body
        return vec![
            TYPE_TOGGLE_REQUEST, // TYPE
            self.pin,            // PIN
        ];
    }

    fn parse_response(&self, packet_body: &Vec<u8>) -> Result<ToggleResponse, Error> {
        // ensure length is correct
        if packet_body.len() != 2 {
            return Err(Error::ResponseMismatch);
        }

        // ensure type is correct
        if packet_body[0] != TYPE_TOGGLE_RESPONSE {
            return Err(Error::ResponseMismatch);
        }

        // parse response
        return Ok(ToggleResponse {
            new_value: packet_body[1],
        });
    }
}
