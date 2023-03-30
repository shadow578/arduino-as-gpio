pub mod gpio;

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
    },

    /// write to a gpio pin
    Write {
        /// the pin to write to
        pin: u8,

        /// the value to write. (0|1 for digital, 0-255 for analog)
        value: u8,

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

    // handle command
    let mut response;
    match args.command {
        Command::Read {
            pin,
            analog,
            inverted,
            pullup,
        } => {
            // send command and get response
            response = gpio::write(
                port.as_mut(),
                gpio::Command {
                    kind: gpio::CommandKind::Read,
                    pin,
                    value: 0,
                    analog,
                    pullup,
                },
                send_retries,
            );

            // invert result
            if inverted {
                if analog {
                    response.value = 255 - response.value;
                } else {
                    response.value = if response.value == 0 { 1 } else { 0 };
                }
            }
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
                gpio::Command {
                    kind: gpio::CommandKind::Write,
                    pin,
                    value,
                    analog,
                    pullup: false,
                },
                send_retries,
            );
        }
    }

    // check for error and exit if there is one
    if response.error != gpio::ErrorKind::None {
        match response.error {
            gpio::ErrorKind::CommunicationError { code } => {
                eprintln!(
                    "Communication failure with GPIO controller on port {} (code={:#02x})",
                    args.port, code
                );
            }
            gpio::ErrorKind::InvalidPin => {
                eprintln!("Invalid pin number");
            }
            gpio::ErrorKind::ResponseMismatch => {
                eprintln!("GPIO Controller response differs from expected response. This could indicate a failing controller, spotty communication, or something else.");
            }
            _ => {
                panic!("you win. Open a issue to claim your cookie.")
            }
        }

        std::process::exit(128);
    }

    // print result
    println!("{}", response.value);

    // exit without error code
    if args.no_exit_code {
        std::process::exit(0);
    }

    // exit with error code 0 if HIGH, 1 if LOW
    std::process::exit(if response.value == 0 { 1 } else { 0 });
}
