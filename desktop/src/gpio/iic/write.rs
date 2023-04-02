use crate::gpio::{as_request_type, as_response_type, Error, Request};

//
// IIC Write Request Constants
//
const TYPE_IIC_WRITE: u8 = 0x04;

const FLAG_IIC_STOP: u8 = 1 << 0;

const RESULT_CODE_SUCCESS: u8 = 0x00;
const RESULT_CODE_DATA_TOO_LONG: u8 = 0x01;
const RESULT_CODE_NACK_ON_ADDRESS: u8 = 0x02;
const RESULT_CODE_NACK_ON_DATA: u8 = 0x03;
const RESULT_CODE_OTHER: u8 = 0x04;
const RESULT_CODE_TIMEOUT: u8 = 0x05;

//
// IIC Write Request Implementation
//
#[derive(Debug)]
pub struct IICWriteRequest {
    pub address: u8,
    pub data: Vec<u8>,
    pub stop: bool,
}

#[derive(Debug)]
pub struct IICWriteResponse {
    pub result_code: IICResultCode,
}

#[derive(Debug)]
pub enum IICResultCode {
    Success,
    DataTooLong,
    NACKOnAddress,
    NACKOnData,
    Other,
    Timeout,
    Unknown { result_code: u8 },
}

impl IICWriteRequest {
    pub fn new(address: u8, data: Vec<u8>, stop: bool) -> IICWriteRequest {
        IICWriteRequest {
            address,
            data,
            stop,
        }
    }
}

impl Request<IICWriteResponse> for IICWriteRequest {
    fn get_packet_body(&self) -> Vec<u8> {
        // set flags
        let mut flags = 0;
        if self.stop {
            flags |= FLAG_IIC_STOP;
        }

        // assemble packet body
        let mut body = vec![
            as_request_type!(TYPE_IIC_WRITE), //TYPE
            self.address,                     // ADDRESS
            flags,                            // FLAGS
        ];
        body.extend_from_slice(&self.data); // DATA
        return body;
    }

    fn parse_response(&self, packet_body: &Vec<u8>) -> Result<IICWriteResponse, Error> {
        // ensure length is correct
        if packet_body.len() != 2 {
            return Err(Error::ResponseMismatch);
        }

        // ensure type is iic write response
        if packet_body[0] != as_response_type!(TYPE_IIC_WRITE) {
            return Err(Error::ResponseMismatch);
        }

        // parse result code
        let result_code_raw = packet_body[1];
        let result_code = parse_result_code(result_code_raw);

        // parse response
        return Ok(IICWriteResponse { result_code });
    }
}

fn parse_result_code(result_code: u8) -> IICResultCode {
    return match result_code {
        RESULT_CODE_SUCCESS => IICResultCode::Success,
        RESULT_CODE_DATA_TOO_LONG => IICResultCode::DataTooLong,
        RESULT_CODE_NACK_ON_ADDRESS => IICResultCode::NACKOnAddress,
        RESULT_CODE_NACK_ON_DATA => IICResultCode::NACKOnData,
        RESULT_CODE_OTHER => IICResultCode::Other,
        RESULT_CODE_TIMEOUT => IICResultCode::Timeout,
        _ => IICResultCode::Unknown { result_code },
    };
}
