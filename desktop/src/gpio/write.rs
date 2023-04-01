use super::{Error, Request};

//
// Write Request Constants
//
const TYPE_WRITE_REQUEST: u8 = 0x02;
const TYPE_WRITE_RESPONSE: u8 = 0x04;

const FLAG_WRITE_ANALOG: u8 = 1 << 1;
const FLAG_WRITE_INVERT: u8 = 1 << 2;

//
// Write Request Implementation
//
#[derive(Debug)]
pub struct WriteRequest {
    pub pin: u8,
    pub value: u16,
    pub analog: bool,
    pub invert: bool,
}

#[derive(Debug)]
pub struct WriteResponse {}

impl WriteRequest {
    pub fn new(pin: u8, value: u16, analog: bool, invert: bool) -> WriteRequest {
        WriteRequest {
            pin,
            value,
            analog,
            invert,
        }
    }
}

impl Request<WriteResponse> for WriteRequest {
    fn get_packet_body(&self) -> Vec<u8> {
        // set flags
        let mut flags = 0;
        if self.analog {
            flags |= FLAG_WRITE_ANALOG;
        }
        if self.invert {
            flags |= FLAG_WRITE_INVERT;
        }

        // assemble packet body
        return vec![
            TYPE_WRITE_REQUEST,        // TYPE
            self.pin,                  // PIN
            (self.value << 8) as u8,   // VALUE (MSB)
            (self.value & 0xFF) as u8, // VALUE (LSB)
            flags,                     // FLAGS
        ];
    }

    fn parse_response(&self, packet_body: &Vec<u8>) -> Result<WriteResponse, Error> {
        // ensure length is correct
        if packet_body.len() != 1 {
            return Err(Error::ResponseMismatch);
        }

        // ensure type is write response
        return if packet_body[0] == TYPE_WRITE_RESPONSE {
            Ok(WriteResponse {})
        } else {
            Err(Error::ResponseMismatch)
        };
    }
}
