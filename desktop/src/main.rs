pub mod gpio;
pub mod sdsp;
use clap::{Parser, Subcommand};
use std::time::Duration;

#[derive(Subcommand, Debug)]
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
        #[arg(short, long)]
        pullup: bool,

        /// enable pulldown resistor? (only digital read)
        #[arg(short, long)]
        pulldown: bool,
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
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    /// the serial port to use for communication
    port: String,

    /// subcommand to use
    #[command(subcommand)]
    command: Command,

    /// the baud rate to use for communication
    #[arg(short, long)]
    baud: Option<u32>,

    /// by default, the exit code is 0 if HIGH, 1 if LOW. use this flag to disable this behavior.
    #[arg(short, long)]
    no_exit_code: bool,

    /// how many times to retry sending a command
    #[arg(short, long)]
    retries: Option<i32>,
}

fn main() {
    // parse command line arguments
    let args = Args::parse();

    // open port
    let baud = args.baud.unwrap_or(115200);
    let mut port = serialport::new(args.port.clone(), baud)
        .timeout(Duration::from_millis(100))
        .open()
        .unwrap_or_else(|e| {
            eprintln!("Failed to open serial port: {}", e);
            std::process::exit(128);
        });

    // get send retries
    let send_retries = args.retries.unwrap_or(3);

    // TODO device ids
    let own_id = 0xAB;
    let recipient_id = 0xCA;

    // handle command
    let response;
    match args.command {
        Command::Read {
            pin,
            analog,
            inverted,
            pullup,
            pulldown,
        } => {
            // send command and get response
            response = gpio::write(
                port.as_mut(),
                gpio::Request::Read {
                    pin,
                    analog,
                    pullup,
                    pulldown,
                },
                own_id,
                recipient_id,
                send_retries,
            );

            // TODO invert result
            //if inverted {
            //    if analog {
            //        response.value = 255 - response.value;
            //    } else {
            //        response.value = if response.value == 0 { 1 } else { 0 };
            //    }
            //}
        }
        Command::Write {
            pin,
            mut value,
            inverted,
            analog,
        } => {
            // invert value
            if inverted {
                if analog {
                    value = 255 - value;
                } else {
                    value = if value == 0 { 1 } else { 0 };
                }
            }

            // send command and get response
            response = gpio::write(
                port.as_mut(),
                gpio::Request::Write { pin, value, analog },
                own_id,
                recipient_id,
                send_retries,
            );
        }
    }

    // check for error and exit if there is one
    if let Err(response_error) = response {
        match response_error {
            gpio::Error::SDSPError { kind } => {
                eprintln!(
                    "Communication failure with GPIO controller on port {} (SDSP error: {:?})",
                    args.port, kind
                );
            }
            gpio::Error::ControllerError { code } => {
                eprintln!(
                "Communication failure with GPIO controller on port {} (Controller error={:#02x})",
                args.port, code
            );
            }
            gpio::Error::ClientError { code } => {
                eprintln!(
                    "Communication failure with GPIO controller on port {} (Client error={:#02x})",
                    args.port, code
                );
            }
            gpio::Error::InvalidPin => {
                eprintln!("Invalid pin number");
            }
            gpio::Error::ResponseMismatch => {
                eprintln!("GPIO Controller response differs from expected response. This could indicate a failing controller, spotty communication, or something else.");
            }
        }

        std::process::exit(128);
    } else {
        match response.unwrap() {
            gpio::Response::Read { value } => {
                // print result
                println!("{}", value);

                // exit without error code
                if args.no_exit_code {
                    std::process::exit(0);
                }

                // exit with error code 0 if HIGH, 1 if LOW
                std::process::exit(if value == 0 { 1 } else { 0 });
            }
            gpio::Response::Write => {
                // print result
                println!("OK");
                std::process::exit(0);
            }
        }
    }
}
