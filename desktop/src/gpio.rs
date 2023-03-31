use std::time::Duration;

use crate::sdsp::{self, PacketInfo};

//
// Protocol Constants
//
pub const TYPE_READ_REQUEST: u8 = 0x01;
pub const TYPE_WRITE_REQUEST: u8 = 0x02;
pub const TYPE_READ_RESPONSE: u8 = 0x03;
pub const TYPE_WRITE_RESPONSE: u8 = 0x04;
pub const TYPE_ERROR_RESPONSE: u8 = 0x05;

pub const FLAG_READ_PULLUP: u8 = 1 << 0;
pub const FLAG_READ_PULLDOWN: u8 = 1 << 1;
pub const FLAG_READ_ANALOG: u8 = 1 << 2;

pub const FLAG_WRITE_ANALOG: u8 = 1 << 1;

pub const ERR_MALFORMED_PACKET: u8 = 0x01;
pub const ERR_INVALID_TYPE: u8 = 0x02;
pub const ERR_INVALID_PIN: u8 = 0x03;

//
// Data Types
//
#[derive(Debug)]
pub enum Request {
    /// read request
    Read {
        pin: u8,
        pullup: bool,
        pulldown: bool,
        analog: bool,
    },
    /// write request
    Write { pin: u8, value: u16, analog: bool },
}

#[derive(Debug)]
pub enum Response {
    /// read response
    Read { value: u16 },

    /// write response
    Write,
}

#[derive(PartialEq, Debug)]
pub enum Error {
    // error in SDSP layer
    SDSPError {
        kind: sdsp::ReadError,
    },

    /// error on the GPIO controller
    ControllerError {
        code: u8,
    },

    /// error in the client app
    ClientError {
        code: u8,
    },

    /// ERR_INVALID_PIN
    InvalidPin,

    /// controller response does not match the request
    ResponseMismatch,
}

//
// Public API
//
pub fn write(
    port: &mut dyn serialport::SerialPort,
    command: Request,
    own_id: u8,
    recipient_id: u8,
    max_retries: i32,
) -> Result<Response, Error> {
    // do send/receive with retries
    let mut tries = 0;
    let mut response: Result<Response, Error>;
    loop {
        //println!("Sending command: {:?}; Retry {}", command, tries);

        // write the request packet
        write_request(port, &command, own_id, recipient_id);

        // read the response
        response = read_response(port, own_id, Duration::from_millis(100));

        // expect a read request to be answered with a read response
        //if let Ok(response) = response {
        //    if request == Request::Read && response != Response::Read {
        //        response = Err(Error::ResponseMismatch);
        //    }
        //
        //}

        // retry only on certain errors (= possibly transient)
        if let Err(error) = &response {
            match error {
                Error::SDSPError { kind: _ } => {}
                Error::ControllerError { code: _ } => {}
                Error::ClientError { code: _ } => {}
                Error::InvalidPin => {
                    break;
                }
                Error::ResponseMismatch => {}
            }
        } else {
            // no error
            break;
        }

        // update retries
        tries += 1;
        if tries > max_retries {
            break;
        }

        // wait a bit before retrying
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    return response;
}

//
// Internal API
//
fn write_request(
    port: &mut dyn serialport::SerialPort,
    request: &Request,
    own_id: u8,
    recipient_id: u8,
) {
    // build packet
    let mut pkg = PacketInfo {
        sender_id: own_id,
        receiver_id: recipient_id,
        body: Vec::new(),
        body_len: 0,
        checksum: 0,
    };

    // build packet body
    match *request {
        Request::Read {
            pin,
            pullup,
            pulldown,
            analog,
        } => {
            let mut flags = 0;
            if pullup {
                flags |= FLAG_READ_PULLUP;
            }
            if pulldown {
                flags |= FLAG_READ_PULLDOWN;
            }
            if analog {
                flags |= FLAG_READ_ANALOG;
            }

            pkg.body = vec![
                TYPE_READ_REQUEST, //TYPE
                pin,               // PIN
                flags,             // FLAGS
            ];
        }
        Request::Write { pin, value, analog } => {
            let mut flags = 0;
            if analog {
                flags |= FLAG_WRITE_ANALOG;
            }

            pkg.body = vec![
                TYPE_WRITE_REQUEST,   // TYPE
                pin,                  // PIN
                (value << 8) as u8,   // VALUE (MSB)
                (value & 0xFF) as u8, // VALUE (LSB)
                flags,                // FLAGS
            ];
        }
    };

    // write the package
    sdsp::write_packet(port, &mut pkg).expect("Failed to write packet");
}

fn read_response(
    port: &mut dyn serialport::SerialPort,
    own_id: u8,
    timeout: Duration,
) -> Result<Response, Error> {
    // read the package data
    let pkg = match sdsp::read_packet(port, own_id, timeout) {
        Ok(pkg) => pkg.body,
        Err(err) => return Err(Error::SDSPError { kind: err }),
    };

    // check packet type and parse
    return match pkg.get(0) {
        Some(&TYPE_READ_RESPONSE) => parse_read_response(&pkg),
        Some(&TYPE_WRITE_RESPONSE) => parse_write_response(&pkg),
        Some(&TYPE_ERROR_RESPONSE) => parse_error_response(&pkg),
        _ => Err(Error::ClientError {
            code: ERR_INVALID_TYPE,
        }),
    };
}

fn parse_read_response(pkg: &Vec<u8>) -> Result<Response, Error> {
    if let Some(value_msb) = pkg.get(1) {
        if let Some(value_lsb) = pkg.get(2) {
            let value = ((*value_msb as u16) << 8) | (*value_lsb as u16);
            return Ok(Response::Read { value });
        }
    }

    return Err(Error::ClientError {
        code: ERR_MALFORMED_PACKET,
    });
}

fn parse_write_response(_pkg: &Vec<u8>) -> Result<Response, Error> {
    return Ok(Response::Write);
}

fn parse_error_response(pkg: &Vec<u8>) -> Result<Response, Error> {
    if let Some(error_code) = pkg.get(1) {
        return Err(Error::ControllerError { code: *error_code });
    }

    return Err(Error::ClientError {
        code: ERR_MALFORMED_PACKET,
    });
}
