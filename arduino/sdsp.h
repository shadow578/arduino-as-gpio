
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
} read_result_t;

//
// SDSP Implementation
//
uint16_t sdsp_crc16(uint8_t pkg[], size_t len)
{
    uint16_t crc = 0x0000;
    for (size_t i = 0; i < len; i++)
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

read_result_t sdsp_read_packet(uint8_t *pkg, uint16_t max_len, uint16_t &pkg_len)
{
    // check for the start of packet
    if (sdsp_serial_read_blocking() != SDSP_PKG_START_BYTE)
        return NO_START;

    // read packet data length
    pkg_len = sdsp_serial_read_blocking() << 8;
    pkg_len |= (uint16_t) sdsp_serial_read_blocking();

    // ensure the packet will fit into the buffer
    if (pkg_len > max_len)
    {
        // fast-forward to the end of the packet
        while (sdsp_serial_read_blocking() != SDSP_PKG_END_BYTE)
            ;
        return DATA_TOO_LONG;
    }

    // read the packet data
    for (size_t i = 0; i < pkg_len; i++)
    {
        pkg[i] = sdsp_serial_read_blocking();
    }

    // read the checksum
    uint16_t chksum = sdsp_serial_read_blocking() << 8;
    chksum |= sdsp_serial_read_blocking();

    // read the end of packet
    if (sdsp_serial_read_blocking() != SDSP_PKG_END_BYTE)
        return NO_END;

    // validate the checksum
    uint16_t expectedChksum = sdsp_crc16(pkg, pkg_len);
    if (chksum != expectedChksum)
        return CHECKSUM_MISMATCH;

    // all OK
    return OK;
}

void sdsp_write_packet(uint8_t *pkg, size_t len)
{
    // write the start of packet
    sdsp_serial_write(SDSP_PKG_START_BYTE);

    // write the packet data length
    sdsp_serial_write((len >> 8) & 0xFF);
    sdsp_serial_write(len & 0xFF);

    // write the packet data
    for (size_t i = 0; i < len; i++)
    {
        sdsp_serial_write(pkg[i]);
    }

    // write the checksum
    uint16_t chksum = sdsp_crc16(pkg, len);
    sdsp_serial_write((chksum >> 8) & 0xFF);
    sdsp_serial_write(chksum & 0xFF);

    // write the end of packet
    sdsp_serial_write(SDSP_PKG_END_BYTE);
}
