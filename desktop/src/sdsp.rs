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
    RecipientMismatch,
    InvalidPacket,
    Timeout,
}

#[derive(Debug)]
pub struct PacketInfo {
    pub sender_id: u8,
    pub receiver_id: u8,
    pub body: Vec<u8>,

    /// internal, set to 0 when creating a new packet
    pub body_len: u16,

    /// internal, set to 0 when creating a new packet
    pub checksum: u16,
}

pub fn read_packet(
    port: &mut dyn SerialPort,
    own_id: u8,
    timeout: Duration,
) -> Result<PacketInfo, ReadError> {
    #[derive(Debug)]
    enum ReadState {
        StartByte,
        SenderID,
        ReceiverID,
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
    const MAX_BUF_SIZE: usize = 64;
    let mut pkg = PacketInfo {
        sender_id: 0,
        receiver_id: 0,
        body_len: 0,
        body: Vec::new(),
        checksum: 0,
    };
    let mut state = ReadState::StartByte;
    loop {
        // check timeout
        if start.elapsed() > timeout {
            return Err(ReadError::Timeout);
        }

        // figure out how many bytes are available to read
        let bytes_available =
            std::cmp::min(port.bytes_to_read().unwrap_or(0) as usize, MAX_BUF_SIZE);
        if bytes_available == 0 {
            continue;
        }

        // read n bytes from port
        let mut buf = vec![0; bytes_available];
        let bytes_read = port.read(&mut buf).unwrap_or(0);

        // handle every byte using state machine
        for i in 0..bytes_read {
            let byte = buf[i];
            match state {
                ReadState::StartByte => {
                    // wait for start byte
                    if byte == PKG_START_BYTE {
                        state = ReadState::SenderID;
                    }
                }
                ReadState::SenderID => {
                    // read sender ID
                    pkg.sender_id = byte;
                    state = ReadState::ReceiverID;
                }
                ReadState::ReceiverID => {
                    // read receiver ID
                    pkg.receiver_id = byte;
                    state = ReadState::LengthMSB;
                }
                ReadState::LengthMSB => {
                    // read length MSB into pkg_len
                    pkg.body_len = (byte as u16) << 8;
                    state = ReadState::LengthLSB;
                }
                ReadState::LengthLSB => {
                    // read length LSB into pkg_len
                    pkg.body_len |= byte as u16;
                    state = ReadState::Body;
                }
                ReadState::Body => {
                    // push bytes into pkg until pkg_len is reached
                    pkg.body.push(byte);
                    if (pkg.body.len() as u16) == pkg.body_len {
                        state = ReadState::ChecksumMSB;
                    }
                }
                ReadState::ChecksumMSB => {
                    // read checksum MSB into pkg_chsum
                    pkg.checksum = (byte as u16) << 8;
                    state = ReadState::ChecksumLSB;
                }
                ReadState::ChecksumLSB => {
                    // read checksum LSB into pkg_chsum
                    pkg.checksum |= byte as u16;
                    state = ReadState::EndByte;
                }
                ReadState::EndByte => {
                    // check end byte
                    if byte == PKG_END_BYTE {
                        // calculate and check checksum
                        let pkg_for_checksum = assemble_packet(&pkg);
                        if sdsp_crc16(&pkg_for_checksum) == pkg.checksum {
                            // check recipient ID
                            if pkg.receiver_id == own_id || pkg.receiver_id == 0xff {
                                return Ok(pkg);
                            } else {
                                return Err(ReadError::RecipientMismatch);
                            }
                        } else {
                            return Err(ReadError::ChecksumMismatch);
                        }
                    } else {
                        return Err(ReadError::InvalidPacket);
                    }
                }
            }
        }
    }
}

pub fn write_packet(
    port: &mut dyn SerialPort,
    packet: &mut PacketInfo,
) -> Result<usize, std::io::Error> {
    // init internal fields of packet
    packet.body_len = packet.body.len() as u16;
    packet.checksum = 0;

    // assemble packet
    let mut pkg_data = assemble_packet(packet);

    // calculate checksum
    let crc = sdsp_crc16(&pkg_data);
    let pkg_len = pkg_data.len();
    pkg_data[pkg_len - 3] = (crc >> 8) as u8;
    pkg_data[pkg_len - 2] = (crc & 0xFF) as u8;

    // write packet to port
    return port.write(&pkg_data);
}

//
// Internal Functions
//
fn assemble_packet(pkg: &PacketInfo) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::new();

    // prologue
    data.push(PKG_START_BYTE);
    data.push(pkg.sender_id);
    data.push(pkg.receiver_id);
    data.push((pkg.body_len >> 8) as u8);
    data.push((pkg.body_len & 0xFF) as u8);

    // body
    data.append(&mut pkg.body.clone());

    // epilogue
    //data.push((pkg.checksum >> 8) as u8);
    //data.push((pkg.checksum & 0xFF) as u8);
    data.push(0);
    data.push(0);
    data.push(PKG_END_BYTE);

    return data;
}

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
