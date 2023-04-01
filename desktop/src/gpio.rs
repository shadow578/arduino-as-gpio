pub mod error;
pub mod read;
pub mod write;

use crate::sdsp;
use serialport::SerialPort;
use std::time::Duration;

//
// Common GPIO functionality
//

/// common GPIO error types
#[derive(PartialEq, Debug)]
pub enum Error {
    // error in SDSP layer
    SDSPError {
        kind: sdsp::ReadError,
    },

    /// error on the remote GPIO controller
    RemoteError {
        code: u8,
    },

    /// error in the host controller (client app)
    HostError {
        code: u8,
    },

    /// ERR_INVALID_PIN response
    InvalidPin,

    /// remote controller response does not match the request
    ResponseMismatch,
}

/// common GPIO request type
pub trait Request<ResponseType>: std::fmt::Debug {
    /// get the SDSP packet body for this request
    fn get_packet_body(&self) -> Vec<u8>;

    /// attempt to parse the response packet body into a response type
    fn parse_response(&self, packet_body: &Vec<u8>) -> Result<ResponseType, Error>;
}

/// GPIO host controller implementation
pub struct HostController {
    port: Box<dyn SerialPort>,
    id: u8,
    read_timeout: Duration,
    max_retries: i32,
}
impl HostController {
    pub fn new(
        port: Box<dyn SerialPort>,
        id: u8,
        read_timeout: Option<Duration>,
        max_retries: Option<i32>,
    ) -> HostController {
        HostController {
            port: port,
            id,
            read_timeout: read_timeout.unwrap_or(Duration::from_millis(100)),
            max_retries: max_retries.unwrap_or(2),
        }
    }

    /// send a request to the GPIO controller with id `recipient_id`, with automatic retries
    pub fn send<ResponseType>(
        &mut self,
        request: &dyn Request<ResponseType>,
        recipient_id: u8,
    ) -> Result<ResponseType, Error> {
        let mut response: Result<ResponseType, Error>;
        let mut tries = 0;
        loop {
            // send the request and read the response
            response = self.send_single(request, recipient_id);

            // retry only on certain errors
            if let Err(error) = &response {
                match error {
                    Error::SDSPError { kind: _ } => {}
                    Error::RemoteError { code: _ } => {}
                    Error::HostError { code: _ } => {}
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
            if tries > self.max_retries {
                break;
            }

            // wait a bit before retrying
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        return response;
    }

    fn send_single<ResponseType>(
        &mut self,
        request: &dyn Request<ResponseType>,
        recipient_id: u8,
    ) -> Result<ResponseType, Error> {
        // build the packet
        let pkg_body = request.get_packet_body();
        let mut pkg = sdsp::Packet::new(self.id, recipient_id, pkg_body);

        // send the packet using SDSP
        let write_result = sdsp::write_packet(&mut self.port, &mut pkg);
        if write_result.is_err() {
            return Err(Error::RemoteError { code: 0xff });
        }

        // read the response from the controller
        let response_pkg = sdsp::read_packet(&mut self.port, self.id, self.read_timeout);
        if let Err(kind) = response_pkg {
            return Err(Error::SDSPError { kind });
        }

        // parse the response
        let response_pkg_body = response_pkg.unwrap().body;
        let response = request.parse_response(&response_pkg_body);

        // if response parsing failed, attempt to parse it as an error response
        if response.is_err() {
            if let Ok(err) = error::ErrorRequest::new().parse_response(&response_pkg_body) {
                return Err(err);
            }
        }

        return response;
    }
}
