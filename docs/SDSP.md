# SDSP: The Small Device Serial Protocol

SDSP is a simple serial protocol designed for use with small devices such as the Arduino. It is designed to be simple and easy to implement.
features of the protocol include:

- simple packet structure
- variable payload length
- multi-device addressing with sender and receiver IDs
- included checksum of the entire packet

## Packet Structure

the protocol consists of tree main sections: a packet prologue, a packet body and a packet epilogue. The packet prologue and epilogue are used to identify the start and end of a packet. The packet body contains the actual data of the packet.

    [START][SENDER_ID][RECEIVER_ID][LENGTH][BODY][CHECKSUM][END]
     1b     1b         1b           2b      n     2b        1b

### Packet Prologue

the packet prologue consists of a single start byte, followed by a sender ID, a receiver ID and a length. The start byte is always `0x7B` (`'{'`).
both the sender ID and receiver ID are single-byte values. The sender ID specifies the ID of the device that sent the packet. The receiver ID specifies the ID of the device that the packet is intended for. If the receiver ID is `0xFF`, the packet is intended for all devices. A receiver ID of `0x00` is reserved for future use.
the length is a two byte integer that specifies the length of the packet body.

### Packet Body

the packet body contains the actual data of the packet.

### Packet Epilogue

the packet epilogue consists of a two-byte checksum and a single end byte. The checksum is a CRC-16 checksum of the entrire packet, with a initial value of `0x0000` and a polynomial of `0x1021`. The end byte is always `0x7D` (`'}'`).

Since the checksum is calculated over the entire packet, it has to be set to a placeholder value of `0x0000` when calculating the checksum.
