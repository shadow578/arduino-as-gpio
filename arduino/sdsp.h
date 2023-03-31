
//
// HAL
//
// single byte serial read
uint8_t sdsp_serial_read_blocking();

// single byte serial write
void sdsp_serial_write(uint8_t data);

//
// Protocol Constants
//
#define SDSP_PKG_START_BYTE 0x7B
#define SDSP_PKG_END_BYTE 0x7D

typedef enum ReadResult
{
    OK,
    NO_START,
    NO_END,
    CHECKSUM_MISMATCH,
    DATA_TOO_LONG,
    RECIPIENT_MISMATCH,
} read_result_t;

//
// SDSP Implementation
//
uint16_t sdsp_crc16(uint8_t *pkg, uint16_t len)
{
    uint16_t crc = 0x0000;
    for (uint16_t i = 0; i < len; i++)
    {
        crc ^= pkg[i];
        for (uint8_t j = 0; j < 8; j++)
        {
            if (crc & 0x0001)
            {
                crc >>= 1;
                crc ^= 0x1021;
            }
            else
            {
                crc >>= 1;
            }
        }
    }

    return crc;
}

void sdsp_ffwd_to_end(uint16_t len)
{
    // read at least len bytes
    for (uint16_t i = 0; i < len; i++)
    {
        sdsp_serial_read_blocking();
    }

    // fast-forward to the next packet end marker
    while (sdsp_serial_read_blocking() != SDSP_PKG_END_BYTE)
        ;
}

read_result_t sdsp_read_packet(uint8_t *buffer,
                               uint16_t buffer_len,
                               uint16_t &pkg_len,
                               uint8_t &pkg_sender_id,
                               uint8_t own_id)
{
    // check for the start of packet
    uint16_t i = 0;
    if ((buffer[i++] = sdsp_serial_read_blocking()) != SDSP_PKG_START_BYTE)
        return NO_START;

    // read sender and receiver IDs
    pkg_sender_id = sdsp_serial_read_blocking();
    uint8_t pkg_receiver_id = sdsp_serial_read_blocking();
    buffer[i++] = pkg_sender_id;
    buffer[i++] = pkg_receiver_id;

    // read packet data length
    buffer[i++] = sdsp_serial_read_blocking(); // MSB
    buffer[i++] = sdsp_serial_read_blocking(); // LSB
    pkg_len = (buffer[i - 2] << 8) | buffer[i - 1];

    // ensure the packet will fit into the buffer
    uint16_t buffer_used_len = (pkg_len + 8);
    if (buffer_used_len > buffer_len)
    {
        // will not fit, fast-forward to the end of the packet
        sdsp_ffwd_to_end(pkg_len);
        return DATA_TOO_LONG;
    }

    // ensure the packet is for us
    if (pkg_receiver_id != own_id && pkg_receiver_id != 0xFF)
    {
        // not for us, fast-forward to the end of the packet
        sdsp_ffwd_to_end(pkg_len);
        return RECIPIENT_MISMATCH;
    }

    // read the packet data
    for (uint16_t j = 0; j < pkg_len; j++)
    {
        buffer[i++] = sdsp_serial_read_blocking();
    }

    // read the checksum
    buffer[i++] = 0; // MSB
    buffer[i++] = 0; // LSB
    uint16_t chksum = (sdsp_serial_read_blocking() << 8) | sdsp_serial_read_blocking();

    // read the end of packet
    if ((buffer[i++] = sdsp_serial_read_blocking()) != SDSP_PKG_END_BYTE)
        return NO_END;

    // validate the checksum
    uint16_t expected_chksum = sdsp_crc16(buffer, buffer_used_len);
    if (chksum != expected_chksum)
        return CHECKSUM_MISMATCH;

    // all OK, remove the prologue from the buffer
    for (int i = 0; i < buffer_used_len; i++)
    {
        buffer[i] = buffer[i + 5];
    }

    return OK;
}

void sdsp_write_packet(uint8_t *body,
                       uint16_t body_len,
                       uint8_t sender_id,
                       uint8_t receiver_id)
{
    // allocate a buffer for the packet
    uint16_t pkg_len = body_len + 8;
    uint8_t pkg[pkg_len];

    // assemble packet:
    uint16_t i = 0;
    // prolouge
    pkg[i++] = SDSP_PKG_START_BYTE;
    pkg[i++] = sender_id;
    pkg[i++] = receiver_id;
    pkg[i++] = (body_len >> 8) & 0xFF;
    pkg[i++] = body_len & 0xFF;

    // body
    for (uint16_t j = 0; j < body_len; j++)
    {
        pkg[i++] = body[j];
    }

    // epilogue
    pkg[i++] = 0x00; // checksum MSB
    pkg[i++] = 0x00; // checksum LSB
    pkg[i++] = SDSP_PKG_END_BYTE;

    // calculate and write the checksum
    uint16_t chksum = sdsp_crc16(pkg, pkg_len);
    pkg[i - 3] = (chksum >> 8) & 0xFF;
    pkg[i - 2] = chksum & 0xFF;

    // write the packet
    for (uint16_t j = 0; j < pkg_len; j++)
    {
        sdsp_serial_write(pkg[j]);
    }
}
