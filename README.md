# Arduino-as-GPIO

use your Arduino as GPIO for any Computer.

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
