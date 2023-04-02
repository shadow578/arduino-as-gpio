# Arduino-as-GPIO Protocol Specification

the Arduino-as-GPIO protocol is a simple protocol that allows you to use your Arduino as GPIO for any Computer. The Protocol is a simple binary protocol that is designed to be flexible and fast.
The Protocol is made up of two distinctive layers, one which handles seperation of packets ( [SDSP](/docs/SDSP.md)) and one which handles the actual commands (described in this document).

## Command Protocol

the command protocol is responsible for handling the actual commands.
the command data is contained in the packet body of the carrier protocol.

### Packet Types

the first byte of the packet body specifies the type of packet. depending on the type of packet, the packet body contains different data.
the most significant bit of the packet type signifies if the packet is a request or a response, and is set to 0 for requests and 1 for responses.

| Request Type ID | Response Type ID | Operation             |
| --------------- | ---------------- | --------------------- |
| `0x01`          | `0x81`           | read                  |
| `0x02`          | `0x82`           | write                 |
| `0x03`          | `0x83`           | toggle                |
| `0x04`          | `0x84`           | i2c write data        |
| `0x7f`          | `0xff`           | error (response only) |

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

#### Read Response

the read response packet body consists of a two-byte value field that contains the value read from the pin.

    [0x81][value]
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

#### Write Response

the write response contains no additional data.

    [0x82]
     1b

### Toggle Request

the toggle request packet body only consists of a single-byte pin number that specifies the pin to operate on.
the toggle request only works on digital pins and toggles the pin between HIGH and LOW.

a toggle request roughly corrosponds to the following arduino code:

```cpp
pinMode(pin, OUTPUT);
digitalWrite(pin, !digitalRead(pin));
```

    [0x03][pin]
     1b    1b

#### Toggle Response

the toggle response contains a single-byte value field that contains the value of the pin after the toggle operation.

    [0x83][value]
     1b    1b

### I2C Write Data Request

the i2c write data request packet body consists of a single-byte address field that specifies the address of the i2c device to write to and a single-byte flags field, followed by a variable number of bytes that contain the data to write to the i2c device.
the data may be empty.

a i2c write data request roughly corrosponds to the following arduino code:

```cpp
Wire.beginTransmission(address);
for (int i = 0; i < data.length; i++) {
    Wire.write(data[i]);
}
Wire.endTransmission(STOP FLAG);
```

    [0x04][address][flags][data...]
     1b    1b       1b     n bytes

| Flag Bit # | Name | Description                                                                                                                                                                      |
| ---------- | ---- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1 (LSB)    | STOP | send a stop message and release the bus after transmission. see [Wire documentation](https://www.arduino.cc/reference/en/language/functions/communication/wire/endtransmission/) |
| 2          | -    | reserved                                                                                                                                                                         |
| 3          | -    | reserved                                                                                                                                                                         |
| 4          | -    | reserved                                                                                                                                                                         |
| 5          | -    | reserved                                                                                                                                                                         |
| 6          | -    | reserved                                                                                                                                                                         |
| 7          | -    | reserved                                                                                                                                                                         |
| 8 (MSB)    | -    | reserved                                                                                                                                                                         |

#### I2C Write Data Response

the i2c write data response contains a single-byte result field that contains the result or error code of the i2c write operation.

    [0x84][result]
     1b    1b

the result code contains the return value of the `Wire.endTransmission()` function.
the following table lists the possible result codes according to the [Wire documentation](https://www.arduino.cc/reference/en/language/functions/communication/wire/endtransmission/).

| Result Code | Description                                    |
| ----------- | ---------------------------------------------- |
| `0x00`      | success                                        |
| `0x01`      | error: data too long to fit in transmit buffer |
| `0x02`      | error: received NACK on transmit of address    |
| `0x03`      | error: received NACK on transmit of data       |
| `0x04`      | error: other error                             |
| `0x05`      | error: timeout                                 |

### Error Response

the error response packet body consists of a single-byte error code.

    [0xff][error-code]
     1b    1b

#### Error Codes

| Error Code | Description               |
| ---------- | ------------------------- |
| `0x01`     | malformed packet received |
| `0x02`     | invalid packet type       |
| `0x03`     | invalid pin value         |
