use super::{Error, Request};

//
// Read Request Constants
//
const TYPE_READ_REQUEST: u8 = 0x01;
const TYPE_READ_RESPONSE: u8 = 0x03;

const FLAG_READ_PULLUP: u8 = 1 << 0;
const FLAG_READ_PULLDOWN: u8 = 1 << 1;
const FLAG_READ_ANALOG: u8 = 1 << 2;

//
// Read Request Implementation
//
#[derive(Debug)]
pub struct ReadRequest {
    pub pin: u8,
    pub pullup: bool,
    pub pulldown: bool,
    pub analog: bool,
}

#[derive(Debug)]
pub struct ReadResponse {
    pub value: u16,
}

impl ReadRequest {
    pub fn new(pin: u8, pullup: bool, pulldown: bool, analog: bool) -> ReadRequest {
        ReadRequest {
            pin: pin,
            pullup: pullup,
            pulldown: pulldown,
            analog: analog,
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

        // assemble packet body
        return vec![
            TYPE_READ_REQUEST, //TYPE
            self.pin,          // PIN
            flags,             // FLAGS
        ];
    }

    fn parse_response(&self, packet_body: &Vec<u8>) -> Result<ReadResponse, Error> {
        // ensure length is correct
        if packet_body.len() != 3 {
            return Err(Error::ResponseMismatch);
        }

        // ensure type is read response
        if packet_body[0] != TYPE_READ_RESPONSE {
            return Err(Error::ResponseMismatch);
        }

        // read value
        let value = ((packet_body[1] as u16) << 8) | (packet_body[2] as u16);
        return Ok(ReadResponse { value });
    }
}
