//
// Configuration
//
#define COMM_BAUD_RATE 115200
#define OWN_DEVICE_ID 0xCA

#define IS_VALID_DIGITAL_PIN(x) ((x) >= 2 && (x) <= 13)
#define IS_VALID_ANALOG_PIN(x) ((x) >= 0 && (x) <= 8)

#define PIN_NO_TO_DIGITAL_PIN(x) (x)
#define PIN_NO_TO_ANALOG_PIN(x) (A0 + x)

//
// SDSP
//
#include "sdsp.h"
uint8_t sdsp_serial_read_blocking()
{
    while (Serial.available() == 0)
        ;
    return Serial.read();
}

void sdsp_serial_write(uint8_t data)
{
    Serial.write(data);
}

//
// Command protocol constants
//
#define TYPE_READ_REQUEST 0x01
#define TYPE_WRITE_REQUEST 0x02
#define TYPE_READ_RESPONSE 0x03
#define TYPE_WRITE_RESPONSE 0x04
#define TYPE_ERROR_RESPONSE 0x05

union read_requests_flags
{
    uint8_t flags;
    struct
    {
        uint8_t pullup : 1;
        uint8_t pulldown : 1;
        uint8_t analog : 1;
        uint8_t reserved : 5;
    };
};

union write_requests_flags
{
    uint8_t flags;
    struct
    {
        uint8_t analog : 1;
        uint8_t reserved : 7;
    };
};

#define ERR_MALFORMED_PACKET 0x01
#define ERR_INVALID_TYPE 0x02
#define ERR_INVALID_PIN 0x03

//
// Implementation
//
#define PKG_BUFFER_LEN 32
uint8_t pkg_buffer[PKG_BUFFER_LEN];

void send_error_response(uint8_t error_code, uint8_t to)
{
    uint8_t pkg[2] = {TYPE_ERROR_RESPONSE, error_code};
    sdsp_write_packet(pkg, 2, OWN_DEVICE_ID, to);
}

void handle_packet(uint8_t pkg_buffer[], uint16_t pkg_len, uint8_t from)
{
    // read packet type
    if (pkg_len < 1)
    {
        send_error_response(ERR_MALFORMED_PACKET, from);
        return;
    }

    uint8_t type = pkg_buffer[0];

    // handle packet types
    switch (type)
    {
    case TYPE_READ_REQUEST:
    {
        // ensure packet length is correct
        if (pkg_len != 3)
            break;
        uint8_t pin = pkg_buffer[1];
        read_requests_flags flags = {pkg_buffer[2]};

        // ensure pin is valid
        if (!flags.analog && !IS_VALID_DIGITAL_PIN(pin))
            break;
        if (flags.analog && !IS_VALID_ANALOG_PIN(pin))
            break;

        // convert pin number to pin
        pin = flags.analog ? PIN_NO_TO_ANALOG_PIN(pin) : PIN_NO_TO_DIGITAL_PIN(pin);

        // set pin mode
        if (flags.pullup)
        {
            pinMode(pin, INPUT_PULLUP);
        }
        else if (flags.pulldown)
        {
#ifdef INPUT_PULLDOWN
            pinMode(pin, INPUT_PULLDOWN);
#else
            pinMode(pin, INPUT);
#endif
        }
        else
        {
            pinMode(pin, INPUT);
        }

        // read pin value
        uint16_t value = 0;
        if (flags.analog)
        {
            value = analogRead(pin);
        }
        else
        {
            value = digitalRead(pin);
        }

        // send response
        uint8_t response[4] = {TYPE_READ_RESPONSE, pin, (uint8_t)(value >> 8), (uint8_t)value};
        sdsp_write_packet(response, 4, OWN_DEVICE_ID, from);
        return;
    }
    case TYPE_WRITE_REQUEST:
    {
        // ensure packet length is correct
        if (pkg_len != 5)
            break;
        uint8_t pin = pkg_buffer[1];
        uint16_t value = (pkg_buffer[2] << 8) | pkg_buffer[3];
        write_requests_flags flags = {pkg_buffer[4]};

        // ensure pin is valid
        if (!flags.analog && !IS_VALID_DIGITAL_PIN(pin))
            break;
        if (flags.analog && !IS_VALID_ANALOG_PIN(pin))
            break;

        // convert pin number to pin
        pin = flags.analog ? PIN_NO_TO_ANALOG_PIN(pin) : PIN_NO_TO_DIGITAL_PIN(pin);

        // set pin mode
        pinMode(pin, OUTPUT);

        // write pin value
        if (flags.analog)
        {
            analogWrite(pin, value);
        }
        else
        {
            digitalWrite(pin, value);
        }

        // send response
        uint8_t response[1] = {TYPE_WRITE_RESPONSE};
        sdsp_write_packet(response, 1, OWN_DEVICE_ID, from);
        return;
    }
    default:
        break;
    }

    // default to error
    send_error_response(ERR_INVALID_TYPE, from);
}

void setup()
{
    Serial.begin(COMM_BAUD_RATE);
}

void loop()
{
    // accept sdsp package
    uint16_t pkg_len = 0;
    uint8_t sender_id = 0;
    read_result_t result = sdsp_read_packet(pkg_buffer, PKG_BUFFER_LEN, &pkg_len, &sender_id, OWN_DEVICE_ID);

    switch (result)
    {
    case OK:
    {
        // handle package
        handle_packet(pkg_buffer, pkg_len, sender_id);
        break;
    }
    case NO_END:
    case CHECKSUM_MISMATCH:
    case DATA_TOO_LONG:
        // send error response if package is malformed
        send_error_response(ERR_MALFORMED_PACKET, sender_id);
        break;
    case NO_START:
    case RECIPIENT_MISMATCH:
    default:
        break;
    }
}
