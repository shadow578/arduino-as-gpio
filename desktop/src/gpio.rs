use std::num::Wrapping;

//
// Protocol Constants
//
const PKG_START_BYTE: u8 = b'{';
const PKG_END_BYTE: u8 = b'}';
const PKG_CHKSUM_PLACEHOLDER_VALUE: u8 = 0x00;
const PKG_RESPONSE_ERROR_MASK: u8 = 0x80;

const CMD_DIGITAL_READ: u8 = 0x1;
const CMD_DIGITAL_READ_PULLUP: u8 = 0x2;
const CMD_DIGITAL_WRITE: u8 = 0x3;

const CMD_ANALOG_READ: u8 = 0x4;
const CMD_ANALOG_WRITE: u8 = 0x5;

//
// Data Types
//
#[derive(PartialEq)]
pub enum CommandKind {
    Read,
    Write,
}

pub struct Command {
    pub kind: CommandKind,
    pub value: u8,
    pub analog: bool,
    pub pullup: bool,
    pub pin: u8,
}

pub struct Response {
    pub command: Command,
    pub value: u8,
    pub error: bool,
}

//
// Public API
//
pub fn write(port: &mut dyn serialport::SerialPort, command: Command) -> Response {
    // write the package
    write_package(port, &command);

    // read the response
    let mut response = read_package(port);

    // validate the response matches the written command
    if response.command.kind != command.kind
        || response.command.pullup != command.pullup
        || response.command.analog != command.analog
    {
        response.error = true;
    }

    return response;
}

//
// Internal API
//
fn write_package(port: &mut dyn serialport::SerialPort, command: &Command) {
    // resolve command byte
    let cmd = match command.kind {
        CommandKind::Read => {
            if command.analog {
                CMD_ANALOG_READ
            } else {
                if command.pullup {
                    CMD_DIGITAL_READ_PULLUP
                } else {
                    CMD_DIGITAL_READ
                }
            }
        }
        CommandKind::Write => {
            if command.analog {
                CMD_ANALOG_WRITE
            } else {
                CMD_DIGITAL_WRITE
            }
        }
    };

    // write the package to the serial port
    write_package_raw(port, cmd, command.pin, command.value);
}

fn read_package(port: &mut dyn serialport::SerialPort) -> Response {
    // read raw response package
    let mut pkg: [u8; 5] = [0; 5];
    read_package_raw(port, &mut pkg);

    // unpack package
    let [_, cmd, result, _, _] = pkg;

    // check if error bit is set
    if (cmd & PKG_RESPONSE_ERROR_MASK) != 0 {
        eprintln!("GPIO response indicates error");
        return Response {
            command: Command {
                kind: CommandKind::Read,
                value: 0,
                analog: false,
                pullup: false,
                pin: 0,
            },
            value: 0,
            error: true,
        };
    }

    // resolve command
    let command: CommandKind;
    let pullup: bool;
    let analog: bool;
    match cmd {
        CMD_DIGITAL_READ => {
            command = CommandKind::Read;
            pullup = false;
            analog = false;
        }
        CMD_DIGITAL_READ_PULLUP => {
            command = CommandKind::Read;
            pullup = true;
            analog = false;
        }
        CMD_DIGITAL_WRITE => {
            command = CommandKind::Write;
            pullup = false;
            analog = false;
        }
        CMD_ANALOG_READ => {
            command = CommandKind::Read;
            pullup = false;
            analog = true;
        }
        CMD_ANALOG_WRITE => {
            command = CommandKind::Write;
            pullup = false;
            analog = true;
        }
        _ => {
            // invalid command
            eprintln!("GPIO response contains invalid command");
            return Response {
                command: Command {
                    kind: CommandKind::Read,
                    value: 0,
                    analog: false,
                    pullup: false,
                    pin: 0,
                },
                value: 0,
                error: true,
            };
        }
    }

    // build the response
    return Response {
        command: Command {
            kind: command,
            value: 0,
            analog: analog,
            pullup: pullup,
            pin: 0,
        },
        value: result,
        error: false,
    };
}

//
// RAW
//
fn write_package_raw(port: &mut dyn serialport::SerialPort, cmd: u8, pin: u8, val: u8) {
    // calculate checksum, overflowing u8 arithmetic
    let checksum: u8 = (Wrapping(PKG_START_BYTE)
        + Wrapping(cmd)
        + Wrapping(pin)
        + Wrapping(val)
        + Wrapping(PKG_CHKSUM_PLACEHOLDER_VALUE)
        + Wrapping(PKG_END_BYTE))
    .0;

    // build and write package
    let pkg: [u8; 6] = [PKG_START_BYTE, cmd, pin, val, checksum, PKG_END_BYTE];
    port.write(&pkg).expect("Failed to write to port");
}

fn read_package_raw(port: &mut dyn serialport::SerialPort, data: &mut [u8; 5]) -> bool {
    // read response from serial port
    port.read(data).expect("Failed to read from port");
    let [start, cmd, result, checksum, end] = data.to_owned();

    // validate start and end bytes
    if start != PKG_START_BYTE || end != PKG_END_BYTE {
        return false;
    }

    // calculate expected checksum
    let expected_checksum: u8 = (Wrapping(PKG_START_BYTE)
        + Wrapping(cmd)
        + Wrapping(result)
        + Wrapping(PKG_CHKSUM_PLACEHOLDER_VALUE)
        + Wrapping(PKG_END_BYTE))
    .0;

    // validate checksum
    return checksum != expected_checksum;
}
