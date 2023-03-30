# Arduino-as-GPIO Protocol Specification

the Arduino-as-GPIO protocol is a simple protocol that allows you to use your Arduino as GPIO for any Computer. The Protocol is a simple binary protocol that is designed to be flexible and fast.
The Protocol is made up of two distinctive layers, one which handles seperation of packets (carrier) and one which handles the actual commands (command).

## Carrier Protocol

the carrier protocol is the packet seperation layer of the protocol. It is responsible for seperating packets from each other and for detecting malformed packets.

### Packet Structure

the protocol consists of tree main sections: a packet prologue, a packet body and a packet epilogue. The packet prologue and epilogue are used to identify the start and end of a packet. The packet body contains the actual data of the packet.

#### Packet Prologue

the packet prologue consists of a single start byte followed by a length. The start byte is always `0x7B` (`'{'`).
The length is a two byte integer that specifies the length of the packet body.

#### Packet Body

the packet body contains the actual data of the packet.

#### Packet Epilogue

the packet epilogue consists of a two-byte checksum and a single end byte. The checksum is a CRC-16 checksum of the packet body, with a initial value of `0x0000` and a polynomial of `0x1021`. The end byte is always `0x7D` (`'}'`).

## Command Protocol

the command protocol is the command layer of the protocol. It is responsible for handling the actual commands.
the command data is contained in the packet body of the carrier protocol.

### Packet Types

the first byte of the packet body specifies the type of packet. depending on the type of packet, the packet body contains different data.

| Packet Type Byte | Operation      | Direction          |
| ---------------- | -------------- | ------------------ |
| `0x01`           | read request   | Desktop -> Arduino |
| `0x02`           | write request  | Desktop -> Arduino |
| `0x03`           | read response  | Arduino -> Desktop |
| `0x04`           | write response | Arduino -> Desktop |
| `0x05`           | error response | Arduino -> Desktop |

### Read Request

the read request packet body consists of a single-byte pin number that specifies the pin to operate on and a single-byte flags field.

> [0x01][pin][flags]

| Flag Bit # | Description              |
| ---------- | ------------------------ |
| 1 (LSB)    | enable pullup resistor   |
| 2          | enable pulldown resistor |
| 3          | analog read              |
| 4          | reserved                 |
| 5          | reserved                 |
| 6          | reserved                 |
| 7          | reserved                 |
| 8 (MSB)    | reserved                 |

### Write Request

the write request packet body consists of a single-byte pin number that specifies the pin to operate on, a single-byte flags field and a two-byte value field.

> [0x02][pin][value][flags]

| Flag Bit # | Description  |
| ---------- | ------------ |
| 1 (LSB)    | analog write |
| 2          | reserved     |
| 3          | reserved     |
| 4          | reserved     |
| 5          | reserved     |
| 6          | reserved     |
| 7          | reserved     |
| 8 (MSB)    | reserved     |

### Read Response

the read response packet body consists of a two-byte value field that contains the value read from the pin.

> [0x03][value]

### Write Response

the write response contains no additional data.

> [0x04]

### Error Response

the error response packet body consists of a single-byte error code.

> [0x05][error-code]

#### Error Codes

| Error Code | Description               |
| ---------- | ------------------------- |
| `0x01`     | malformed packet received |
| `0x02`     | invalid packet type       |
| `0x03`     | invalid pin value         |
