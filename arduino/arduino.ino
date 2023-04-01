//
// Configuration
//
#define COMM_BAUD_RATE 115200
#define OWN_DEVICE_ID 0xCA

#define IS_DIGITAL_PIN(x) ((x) >= 2 && (x) <= 13)
#define IS_ANALOG_PIN(x) ((x) >= A0 && (x) < A7)
#define IS_PWM_PIN(x) (x == 3 || x == 5 || x == 6 || x == 9 || x == 10 || x == 11)

// only analog pins can use analogRead()
#define IS_VALID_PIN_FOR_ANALOG_READ(x) IS_ANALOG_PIN(x)

// both analog and digital pins can use digitalRead()
#define IS_VALID_PIN_FOR_DIGITAL_READ(x) IS_DIGITAL_PIN(x) || IS_ANALOG_PIN(x)

// both analog and digital pins can use digitalWrite()
#define IS_VALID_PIN_FOR_DIGITAL_WRITE(x) IS_DIGITAL_PIN(x) || IS_ANALOG_PIN(x)

// on most arduinos, only some pins can do analogWrite() (PWM pins)
#define IS_VALID_PIN_FOR_ANALOG_WRITE(x) IS_PWM_PIN(x)

// invert the value from analogRead()
#define INVERT_ANALOG_READ_VALUE(x) (1024 - x)

// invert the value supplied to analogWrite()
#define INVERT_ANALOG_WRITE_VALUE(x) (255 - x)

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
#define TYPE_TOGGLE_REQUEST 0x06
#define TYPE_TOGGLE_RESPONSE 0x07

#define FLAG_READ_PULLUP (1 << 0)
#define FLAG_READ_PULLDOWN (1 << 1)
#define FLAG_READ_ANALOG (1 << 2)
#define FLAG_READ_INVERT (1 << 3)
#define FLAG_READ_DIRECT (1 << 4)

#define FLAG_WRITE_ANALOG (1 << 1)
#define FLAG_WRITE_INVERT (1 << 2)

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
        {
            send_error_response(ERR_MALFORMED_PACKET, from);
            break;
        }
        uint8_t pin = pkg_buffer[1];
        uint8_t flags = pkg_buffer[2];

        // ensure pin is valid
        bool analog = flags & FLAG_READ_ANALOG;
        if ((!analog && !IS_VALID_PIN_FOR_DIGITAL_READ(pin)) || (analog && !IS_VALID_PIN_FOR_ANALOG_READ(pin)))
        {
            send_error_response(ERR_INVALID_PIN, from);
            break;
        }

        // do not set pin mode in DIRECT mode
        if (flags & FLAG_READ_DIRECT)
        {
            // DIRECT mode cannot be used with analog, pullup or pulldown
            if (analog || flags & (FLAG_READ_PULLUP) || flags & FLAG_READ_PULLDOWN)
            {
                send_error_response(ERR_INVALID_TYPE, from);
                break;
            }
        }
        else
        {

            // set pin mode
            if (analog)
            {
                pinMode(pin, INPUT_PULLUP);
            }
            else if (flags & FLAG_READ_PULLDOWN)
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
        }

        // read pin value
        uint16_t value = 0;
        if (analog)
        {
            value = analogRead(pin);

            // invert value if needed
            value = (flags & FLAG_READ_INVERT) ? INVERT_ANALOG_READ_VALUE(value) : value;
        }
        else
        {
            value = digitalRead(pin);

            // invert value if needed
            value = (flags & FLAG_READ_INVERT) ? !value : value;
        }

        // send response
        uint8_t response[3] = {TYPE_READ_RESPONSE, (uint8_t)(value >> 8), (uint8_t)value};
        sdsp_write_packet(response, 3, OWN_DEVICE_ID, from);
        return;
    }
    case TYPE_WRITE_REQUEST:
    {
        // ensure packet length is correct
        if (pkg_len != 5)
        {
            send_error_response(ERR_MALFORMED_PACKET, from);
            break;
        }
        uint8_t pin = pkg_buffer[1];
        uint16_t value = (pkg_buffer[2] << 8) | pkg_buffer[3];
        uint8_t flags = {pkg_buffer[4]};

        // ensure pin is valid
        bool analog = flags & FLAG_WRITE_ANALOG;
        if ((!analog && !IS_VALID_PIN_FOR_DIGITAL_WRITE(pin)) || (analog && !IS_VALID_PIN_FOR_ANALOG_WRITE(pin)))
        {
            send_error_response(ERR_INVALID_PIN, from);
            break;
        }

        // set pin mode
        pinMode(pin, OUTPUT);

        // write pin value
        if (analog)
        {
            // invert value if needed
            value = (flags & FLAG_WRITE_INVERT) ? INVERT_ANALOG_WRITE_VALUE(value) : value;

            analogWrite(pin, value);
        }
        else
        {
            // invert value if needed
            value = (flags & FLAG_WRITE_INVERT) ? !value : value;

            digitalWrite(pin, value == 0x0 ? LOW : HIGH);
        }

        // send response
        uint8_t response[1] = {TYPE_WRITE_RESPONSE};
        sdsp_write_packet(response, 1, OWN_DEVICE_ID, from);
        return;
    }
    case TYPE_TOGGLE_REQUEST:
    {
        // ensure packet length is correct
        if (pkg_len != 2)
        {
            send_error_response(ERR_MALFORMED_PACKET, from);
            break;
        }
        uint8_t pin = pkg_buffer[1];

        // ensure pin is valid
        if (!IS_VALID_PIN_FOR_DIGITAL_WRITE(pin))
        {
            send_error_response(ERR_INVALID_PIN, from);
            break;
        }

        // set pin mode
        pinMode(pin, OUTPUT);

        // read current value DIRECT
        uint8_t value = digitalRead(pin);

        // update and write value
        value = value == 0x0 ? HIGH : LOW;
        digitalWrite(pin, value);

        // send response
        uint8_t response[2] = {TYPE_TOGGLE_RESPONSE, value};
        sdsp_write_packet(response, 2, OWN_DEVICE_ID, from);
        return;
    }
    default:
        send_error_response(ERR_INVALID_TYPE, from);
        return;
    }

    // default to malformed packet error
    send_error_response(ERR_MALFORMED_PACKET, from);
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
    read_result_t result = sdsp_read_packet(pkg_buffer, PKG_BUFFER_LEN, pkg_len, sender_id, OWN_DEVICE_ID);

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
