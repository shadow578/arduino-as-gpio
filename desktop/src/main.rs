pub mod gpio;
pub mod sdsp;

use clap::{Parser, Subcommand};
use gpio::{
    iic::write::{IICResultCode, IICWriteRequest},
    read::ReadRequest,
    toggle::ToggleRequest,
    write::WriteRequest,
    Error, HostController,
};
use std::time::Duration;

//
// Clap argument structures
//
#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// read from a gpio pin
    Read {
        /// the pin to read from
        pin: u8,

        /// read a analog value?
        #[arg(short, long)]
        analog: bool,

        /// invert the value? (also affects analog read)
        #[arg(short, long)]
        inverted: bool,

        /// enable pullup resistor? (only digital read)
        #[arg(long)]
        pullup: bool,

        /// enable pulldown resistor? (only digital read)
        #[arg(long)]
        pulldown: bool,

        /// read the value currently output on the pin
        /// (digitalRead without setting pinMode first)
        #[arg(long)]
        direct: bool,
    },

    /// write to a gpio pin
    Write {
        /// the pin to write to
        pin: u8,

        /// the value to write. (0|1 for digital, 0-255 for analog)
        value: u16,

        /// invert the value? (also affects analog write)
        #[arg(short, long)]
        inverted: bool,

        /// write a analog value?
        #[arg(short, long)]
        analog: bool,
    },

    /// toggle a gpio pin
    Toggle {
        /// the pin to toggle
        pin: u8,
    },

    /// interact with I2C devices
    I2C {
        // the operation to perform
        #[command(subcommand)]
        command: I2CCommand,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum I2CCommand {
    /// write data to an I2C device
    Write {
        /// the address of the device to write to
        address: u8,

        /// the data to write
        data: Option<Vec<u8>>,

        /// send a stop condition after writing?
        #[arg(short, long)]
        stop: bool,
    },
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    /// the serial port to use for communication
    port: String,

    /// the baud rate to use for communication
    #[arg(short, long)]
    baud: Option<u32>,

    /// by default, the exit code is 0 if HIGH, 1 if LOW. use this flag to disable this behavior.
    #[arg(short, long)]
    no_exit_code: bool,

    /// how many times to retry sending a command
    #[arg(short, long)]
    retries: Option<i32>,

    /// the address of the host controller
    #[arg(short, long)]
    own_id: Option<u8>,

    /// the address of the target controller. if not specified, defaults to a broadcast (only a valid strategy if a single controller is attached)
    target_id: Option<u8>,

    /// subcommand to use
    #[command(subcommand)]
    command: Command,
}

fn main() {
    // parse command line args
    let mut args = Args::parse();

    // create the host controller instance
    let mut host = create_host_controller(&args);

    // resolve target id, default to boardcast (only a valid strategy for single device networks)
    args.target_id = Some(args.target_id.unwrap_or(0xFF));

    // execute the command
    execute_command(&mut host, args.command.clone(), &args);

    // exit with success code
    std::process::exit(0);
}

fn create_host_controller(args: &Args) -> HostController {
    // create the serial port
    let port = serialport::new(args.port.clone(), args.baud.unwrap_or(115200))
        .timeout(Duration::from_millis(100))
        .open()
        .unwrap_or_else(|e| {
            eprintln!("Failed to open serial port: {}", e);
            std::process::exit(128);
        });

    // create the host controller instance
    HostController::new(
        port,
        args.own_id.unwrap_or(0xAA),
        Some(Duration::from_millis(100)),
        args.retries,
    )
}

fn execute_command(host: &mut HostController, command: Command, args: &Args) {
    match command {
        Command::Read {
            pin,
            analog,
            inverted,
            pullup,
            pulldown,
            direct,
        } => {
            // create the request
            let request = ReadRequest::new(pin, pullup, pulldown, analog, inverted, direct);

            // send the request and handle the response
            let response = host.send(&request, args.target_id.unwrap());
            match response {
                Ok(response) => {
                    // print the response value
                    println!("{}", response.value);

                    // exit with the correct code
                    if !args.no_exit_code {
                        std::process::exit(response.value as i32);
                    }
                }
                Err(error) => {
                    print_gpio_error_and_exit(error);
                }
            }
        }
        Command::Write {
            pin,
            value,
            inverted,
            analog,
        } => {
            // create the request
            let request = WriteRequest::new(pin, value, analog, inverted);

            // send the request
            let response = host.send(&request, args.target_id.unwrap());
            match response {
                Ok(_) => {
                    println!("{}", value);
                }
                Err(error) => {
                    print_gpio_error_and_exit(error);
                }
            }
        }
        Command::Toggle { pin } => {
            // create the request
            let request = ToggleRequest::new(pin);

            // send the request
            let response = host.send(&request, args.target_id.unwrap());
            match response {
                Ok(response) => {
                    // print the response value
                    println!("{}", response.new_value);

                    // exit with the correct code
                    if !args.no_exit_code {
                        std::process::exit(response.new_value as i32);
                    }
                }
                Err(error) => {
                    print_gpio_error_and_exit(error);
                }
            }
        }
        Command::I2C { command } => {
            execute_i2c_command(host, command, &args);
        }
    }
}

fn execute_i2c_command(host: &mut HostController, command: I2CCommand, args: &Args) {
    match command {
        I2CCommand::Write {
            address,
            data,
            stop,
        } => {
            // create the request
            let request = IICWriteRequest::new(address, data.unwrap_or(vec![]), stop);

            // send the request
            let response = host.send(&request, args.target_id.unwrap());
            match response {
                Ok(response) => match response.result_code {
                    IICResultCode::Success => {
                        println!("Success");
                    }
                    IICResultCode::DataTooLong => {
                        eprintln!("i2c write failed: data too long reported by Wire");
                        std::process::exit(128);
                    }
                    IICResultCode::NACKOnAddress => {
                        eprintln!("i2c write failed: NACK on address reported by Wire");
                        std::process::exit(128);
                    }
                    IICResultCode::NACKOnData => {
                        eprintln!("i2c write failed: NACK on data reported by Wire");
                        std::process::exit(128);
                    }
                    IICResultCode::Other => {
                        eprintln!("i2c write failed: other error reported by Wire");
                        std::process::exit(128);
                    }
                    IICResultCode::Timeout => {
                        eprintln!("i2c write failed: timeout reported by Wire");
                        std::process::exit(128);
                    }
                    IICResultCode::Unknown { result_code } => {
                        eprintln!(
                            "i2c write failed: unknown error {:#04x} reported by Wire",
                            result_code
                        );
                        std::process::exit(128);
                    }
                },
                Err(error) => {
                    print_gpio_error_and_exit(error);
                }
            }
        }
    }
}

fn print_gpio_error_and_exit(error: Error) -> ! {
    // print a nice error message
    match error {
        Error::SDSPError { kind } => {
            eprintln!(
                "communication failed with remote controller (SDSP error: {:?})",
                kind
            );
        }
        Error::RemoteError { code } => {
            eprintln!("remote controller returned error code {:#04x}", code);
        }
        Error::HostError { code } => {
            eprintln!("host controller returned error code {:#04x}", code);
        }
        Error::InvalidPin => {
            eprintln!("the pin number is invalid for the requested operation");
        }
        Error::ResponseMismatch => {
            eprintln!("the response from the remote controller did not match the expected response. this could be caused by a communication issue or a incompatible controller");
        }
    };

    // exit with error code
    std::process::exit(128);
}
