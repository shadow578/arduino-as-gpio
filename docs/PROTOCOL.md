# Arduino-as-GPIO Protocol Specification

the Arduino-as-GPIO protocol is a simple protocol that allows you to use your Arduino as GPIO for any Computer. The Protocol is a simple binary protocol that is designed to be flexible and fast.
The Protocol is made up of two distinctive layers, one which handles seperation of packets ( [SDSP](/docs/SDSP.md)) and one which handles the actual commands (described in this document).

## Command Protocol

the command protocol is responsible for handling the actual commands.
the command data is contained in the packet body of the carrier protocol.

### Packet Types

the first byte of the packet body specifies the type of packet. depending on the type of packet, the packet body contains different data.

| Packet Type Byte | Operation       | Direction          |
| ---------------- | --------------- | ------------------ |
| `0x01`           | read request    | Desktop -> Arduino |
| `0x03`           | read response   | Arduino -> Desktop |
| `0x02`           | write request   | Desktop -> Arduino |
| `0x04`           | write response  | Arduino -> Desktop |
| `0x05`           | error response  | Arduino -> Desktop |
| `0x06`           | toggle request  | Desktop -> Arduino |
| `0x07`           | toggle response | Arduino -> Desktop |

### Read Request

the read request packet body consists of a single-byte pin number that specifies the pin to operate on and a single-byte flags field.
a read request is answered with a read response packet.

a read request roughly corrosponds to the following arduino code:

```cpp
pinMode(pin, INPUT); // or INPUT_PULLUP or INPUT_PULLDOWN depending on the flags
digitalRead(pin);
// ... or ...
analogRead(pin)
```

    [0x01][pin][flags]
     1b    1b   1b

| Flag Bit # | Name     | Description                                   |
| ---------- | -------- | --------------------------------------------- |
| 1 (LSB)    | PULLUP   | enable pullup resistor                        |
| 2          | PULLDOWN | enable pulldown resistor                      |
| 3          | ANALOG   | analog read                                   |
| 4          | INVERT   | invert the value                              |
| 5          | DIRECT   | call digitalRead without setting the pin mode |
| 6          | -        | reserved                                      |
| 7          | -        | reserved                                      |
| 8 (MSB)    | -        | reserved                                      |

### Read Response

the read response packet body consists of a two-byte value field that contains the value read from the pin.

    [0x03][value]
     1b   2b

### Write Request

the write request packet body consists of a single-byte pin number that specifies the pin to operate on, a single-byte flags field and a two-byte value field.
a write request is answered with a write response packet.

a write request roughly corrosponds to the following arduino code:

```cpp
pinMode(pin, OUTPUT);
digitalWrite(pin, value == 0 ? LOW : HIGH);
// ... or ...
analogWrite(pin, value)
```

    [0x02][pin][value][flags]
     1b    1b   2b    1b

| Flag Bit # | Name   | Description      |
| ---------- | ------ | ---------------- |
| 1 (LSB)    | ANALOG | analog write     |
| 2          | INVERT | invert the value |
| 3          | -      | reserved         |
| 4          | -      | reserved         |
| 5          | -      | reserved         |
| 6          | -      | reserved         |
| 7          | -      | reserved         |
| 8 (MSB)    | -      | reserved         |

### Write Response

the write response contains no additional data.

    [0x04]
     1b

### Toggle Request

the toggle request packet body only consists of a single-byte pin number that specifies the pin to operate on.
the toggle request only works on digital pins and toggles the pin between HIGH and LOW.

a toggle request roughly corrosponds to the following arduino code:

```cpp
pinMode(pin, OUTPUT);
digitalWrite(pin, !digitalRead(pin));
```

    [0x06][pin]
     1b    1b

### Toggle Response

the toggle response contains a single-byte value field that contains the value of the pin after the toggle operation.

    [0x07][value]
     1b    1b

### Error Response

the error response packet body consists of a single-byte error code.

    [0x05][error-code]
     1b    1b

#### Error Codes

| Error Code | Description               |
| ---------- | ------------------------- |
| `0x01`     | malformed packet received |
| `0x02`     | invalid packet type       |
| `0x03`     | invalid pin value         |
