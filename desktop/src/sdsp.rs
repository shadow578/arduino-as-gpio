use serialport::SerialPort;
use std::time::Duration;

//
// SDSP Protocol Constants
//
const PKG_START_BYTE: u8 = 0x7B;
const PKG_END_BYTE: u8 = 0x7D;

//
// Public API
//
#[derive(PartialEq, Debug)]
pub enum ReadError {
    ChecksumMismatch,
    InvalidPacket,
    Timeout,
}

pub fn read_packet(port: &mut dyn SerialPort, timeout: Duration) -> Result<Vec<u8>, ReadError> {
    #[derive(Debug)]
    enum ReadState {
        StartByte,
        LengthMSB,
        LengthLSB,
        Body,
        ChecksumMSB,
        ChecksumLSB,
        EndByte,
    }

    // start timeout timer
    let start = std::time::Instant::now();

    // read packet in chunks of 64 bytes
    let mut buf: [u8; 64] = [0; 64];
    let mut pkg_body: Vec<u8> = Vec::new();
    let mut pkg_len: u16 = 0;
    let mut pkg_crc: u16 = 0;
    let mut state = ReadState::StartByte;
    loop {
        // read from port
        let bytes_read = port.read(&mut buf).unwrap_or(0);

        // handle every byte using state machine
        for i in 0..bytes_read {
            let byte = buf[i];
            match state {
                ReadState::StartByte => {
                    // wait for start byte
                    if byte == PKG_START_BYTE {
                        state = ReadState::LengthMSB;
                    }
                }
                ReadState::LengthMSB => {
                    // read length MSB into pkg_len
                    pkg_len = (byte as u16) << 8;
                    state = ReadState::LengthLSB;
                }
                ReadState::LengthLSB => {
                    // read length LSB into pkg_len
                    pkg_len |= byte as u16;
                    state = ReadState::Body;
                }
                ReadState::Body => {
                    // push bytes into pkg until pkg_len is reached
                    pkg_body.push(byte);
                    if (pkg_body.len() as u16) == pkg_len {
                        state = ReadState::ChecksumMSB;
                    }
                }
                ReadState::ChecksumMSB => {
                    // read checksum MSB into pkg_chsum
                    pkg_crc = (byte as u16) << 8;
                    state = ReadState::ChecksumLSB;
                }
                ReadState::ChecksumLSB => {
                    // read checksum LSB into pkg_chsum
                    pkg_crc |= byte as u16;
                    state = ReadState::EndByte;
                }
                ReadState::EndByte => {
                    // check end byte and checksum
                    if byte == PKG_END_BYTE {
                        if sdsp_crc16(&pkg_body) == pkg_crc {
                            return Ok(pkg_body);
                        } else {
                            return Err(ReadError::ChecksumMismatch);
                        }
                    } else {
                        return Err(ReadError::InvalidPacket);
                    }
                }
            }
        }

        // check timeout
        if start.elapsed() > timeout {
            return Err(ReadError::Timeout);
        }
    }
}

pub fn write_packet(
    port: &mut dyn SerialPort,
    body: &mut Vec<u8>,
) -> Result<usize, std::io::Error> {
    // build packet:
    let mut pkg: Vec<u8> = Vec::new();

    // prologue
    pkg.push(PKG_START_BYTE);
    pkg.push((body.len() >> 8) as u8);
    pkg.push((body.len() & 0xFF) as u8);

    // body
    pkg.append(&mut body.clone());

    // epilogue
    let crc = sdsp_crc16(&body);
    pkg.push((crc >> 8) as u8);
    pkg.push((crc & 0xFF) as u8);
    pkg.push(PKG_END_BYTE);

    // write packet to port
    return port.write(&pkg);
}

//
// Internal Functions
//
fn sdsp_crc16(data: &Vec<u8>) -> u16 {
    let mut crc: u16 = 0x0000;
    for byte in data {
        crc ^= *byte as u16;
        for _ in 0..8 {
            if (crc & 0x0001) != 0 {
                crc >>= 1;
                crc ^= 0x1021;
            } else {
                crc >>= 1;
            }
        }
    }

    return crc;
}
