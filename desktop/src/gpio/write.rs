use super::{as_request_type, as_response_type, Error, Request};

//
// Write Request Constants
//
const TYPE_WRITE: u8 = 0x02;

const FLAG_WRITE_ANALOG: u8 = 1 << 0;
const FLAG_WRITE_INVERT: u8 = 1 << 1;

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
            as_request_type!(TYPE_WRITE), // TYPE
            self.pin,                     // PIN
            (self.value << 8) as u8,      // VALUE (MSB)
            (self.value & 0xFF) as u8,    // VALUE (LSB)
            flags,                        // FLAGS
        ];
    }

    fn parse_response(&self, packet_body: &Vec<u8>) -> Result<WriteResponse, Error> {
        // ensure length is correct
        if packet_body.len() != 1 {
            return Err(Error::ResponseMismatch);
        }

        // ensure type is write response
        return if packet_body[0] == as_response_type!(TYPE_WRITE) {
            Ok(WriteResponse {})
        } else {
            Err(Error::ResponseMismatch)
        };
    }
}
