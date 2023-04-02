use super::{as_request_type, as_response_type, Error, Request};

//
// Read Request Constants
//
const TYPE_READ: u8 = 0x01;

const FLAG_READ_PULLUP: u8 = 1 << 0;
const FLAG_READ_PULLDOWN: u8 = 1 << 1;
const FLAG_READ_ANALOG: u8 = 1 << 2;
const FLAG_READ_INVERT: u8 = 1 << 3;
const FLAG_READ_DIRECT: u8 = 1 << 4;

//
// Read Request Implementation
//
#[derive(Debug)]
pub struct ReadRequest {
    pub pin: u8,
    pub pullup: bool,
    pub pulldown: bool,
    pub analog: bool,
    pub invert: bool,
    pub direct: bool,
}

#[derive(Debug)]
pub struct ReadResponse {
    pub value: u16,
}

impl ReadRequest {
    pub fn new(
        pin: u8,
        pullup: bool,
        pulldown: bool,
        analog: bool,
        invert: bool,
        direct: bool,
    ) -> ReadRequest {
        ReadRequest {
            pin,
            pullup,
            pulldown,
            analog,
            invert,
            direct,
        }
    }
}

impl Request<ReadResponse> for ReadRequest {
    fn get_packet_body(&self) -> Vec<u8> {
        // set flags
        let mut flags = 0;
        if self.pullup {
            flags |= FLAG_READ_PULLUP;
        }
        if self.pulldown {
            flags |= FLAG_READ_PULLDOWN;
        }
        if self.analog {
            flags |= FLAG_READ_ANALOG;
        }
        if self.invert {
            flags |= FLAG_READ_INVERT;
        }
        if self.direct {
            flags |= FLAG_READ_DIRECT;
        }

        // assemble packet body
        return vec![
            as_request_type!(TYPE_READ), //TYPE
            self.pin,                    // PIN
            flags,                       // FLAGS
        ];
    }

    fn parse_response(&self, packet_body: &Vec<u8>) -> Result<ReadResponse, Error> {
        // ensure length is correct
        if packet_body.len() != 3 {
            return Err(Error::ResponseMismatch);
        }

        // ensure type is read response
        if packet_body[0] != as_response_type!(TYPE_READ) {
            return Err(Error::ResponseMismatch);
        }

        // read value
        let value = ((packet_body[1] as u16) << 8) | (packet_body[2] as u16);
        return Ok(ReadResponse { value });
    }
}
