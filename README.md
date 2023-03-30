# Arduino-as-GPIO

use your Arduino as GPIO for any Computer you can attach a Arduino to.
Faster than most other solutions i've found, with just 16 ms delay between running the command and the GPIO pin changing.

## Why?

I wanted to control lights on my 3D-Printer using Octoprint, but have octoprint running on a old Laptop that doesn't have any GPIO pins. So I decided to use my Arduino as GPIO for the Laptop.

## How?

The Arduino is connected to the Laptop via USB. The Arduino is running a program that listens for commands and executes them. The commands are sent from the Laptop to the Arduino via Serial.

## Usage

```
<COMMAND> ::= agpio <PORT> [--baud <BAUD>] [--no-exit-code] <READ_ARGS>|<WRITE_ARGS>

READ_ARGS ::= <READ_DIGITAL_ARGS>|<READ_ANALOG_ARGS>
READ_DIGITAL_ARGS ::= <PIN> [--inverted] [--pullup]
READ_ANALOG_ARGS ::= <PIN> --analog [--inverted]

WRITE_ARGS ::= <WRITE_DIGITAL_ARGS>|<WRITE_ANALOG_ARGS>
WRITE_DIGITAL_ARGS ::= <PIN> <VALUE> [--inverted]
WRITE_ANALOG_ARGS ::= <PIN> <VALUE> --analog [--inverted]
```

### Examples

```bash
# read digital pin 13
$ agpio COM3 read 13

# set digital pin 13 to high
$ agpio COM3 write 13 1

# read digital pin 8 with pullup resistor enabled
$ agpio COM3 read 8 --pullup

# analog write a value of 120 to pin 9
$ gpio COM3 write 9 120 --analog
```
