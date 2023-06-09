use super::{as_request_type, as_response_type, Error, Request};

//
// Toggle Request Constants
//
const TYPE_TOGGLE: u8 = 0x03;

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
            as_request_type!(TYPE_TOGGLE), // TYPE
            self.pin,                      // PIN
        ];
    }

    fn parse_response(&self, packet_body: &Vec<u8>) -> Result<ToggleResponse, Error> {
        // ensure length is correct
        if packet_body.len() != 2 {
            return Err(Error::ResponseMismatch);
        }

        // ensure type is correct
        if packet_body[0] != as_response_type!(TYPE_TOGGLE) {
            return Err(Error::ResponseMismatch);
        }

        // parse response
        return Ok(ToggleResponse {
            new_value: packet_body[1],
        });
    }
}
