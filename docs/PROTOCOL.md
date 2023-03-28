# Arduino-as-GPIO Protocol Specification

the Arduino-as-GPIO protocol is a simple protocol that allows you to use your Arduino as GPIO for any Computer. The Protocol is a simple binary protocol that is designed to be simple and fast.

## Packet Structure

the protocol consists of tree main sections: a packet prologue, a packet body and a packet epilogue. The packet prologue and epilogue are used to identify the start and end of a packet. The packet body contains the actual data of the packet.

### Packet Prologue

the packet prologue is a single byte that is used to identify the start of a packet. The packet prologue is always `0x7B` (`'{'`).

### Packet Body

the packet body contains the actual data of the packet.
the packet body differs between command and response packets. See the [packet types](#packet-types) section for more information.

### Packet Epilogue

the packet epilogue consists of two bytes. The first byte is a checksum of the entire packet. The second byte is always `0x7D` (`'}'`).

The checksum is calculated by adding all bytes of the packet together and truncating the result to a single byte. The checksum is calculated including the packet prologue and epilogue.
Since the checksum contains the checksum field itself, the checksum field is set to `0x00` when calculating the checksum.

## Packet Types

the protocol consists of two packet types: command packets and response packets. Command packets are sent from the desktop app to the arduino. Response packets are sent from the arduino to the desktop app in response to a command packet.

### Command Packet

the command packet's body consists of a single command byte, a single-byte pin number that specifies the pin to operate on and a single-byte value that specifies the value to set the pin to. The command byte specifies the type of operation to perform on the pin. The pin number specifies the pin to operate on. The value specifies the value to set the pin to.
on read operations, the value is ignored.

| Command Byte | Operation                 |
| ------------ | ------------------------- |
| `0x01`       | digital read              |
| `0x02`       | digital read, with pullup |
| `0x03`       | digital write             |
| `0x04`       | analog read               |
| `0x05`       | analog write              |

### Response Packet

the response packet's body consists of a single command byte, and a single-byte result value.
the command byte specifies the type of operation that was performed on the pin. The result value specifies the result of the operation.
on write operations, the result value is set to the value that was written to the pin.
on read operations, the result value is set to the value that was read from the pin.

the command byte uses the same values as the command byte in the command packet.

#### Error

if an error occurs, the most significant bit of the command byte is set to `1`. The result value is set to a error code.

| Error Code | Description                      |
| ---------- | -------------------------------- |
| `0x01`     | malformed packet received        |
| `0x02`     | invalid packet checksum receivec |
| `0x03`     | invalid pin value                |
| `0x04`     | invalid command byte             |
