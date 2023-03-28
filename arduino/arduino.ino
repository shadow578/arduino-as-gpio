//
// Basic Configuration
//
#define COMM_BAUD_RATE 115200

#define MIN_PIN_NUMBER 2
#define MAX_PIN_NUMBER 13

//
// Macros
//
#define IS_VALID_PIN(x) ((x) >= MIN_PIN_NUMBER && (x) <= MAX_PIN_NUMBER)

//
// Protocol Constants
//
#define PKG_START_BYTE '{'
#define PKG_END_BYTE '}'
#define PKG_CHKSUM_PLACEHOLDER_VALUE 0x00
#define PKG_RESPONSE_ERROR_MASK 0x80

#define CMD_DIGITAL_READ 0x1
#define CMD_DIGITAL_READ_PULLUP 0x2
#define CMD_DIGITAL_WRITE 0x3
#define CMD_ANALOG_READ 0x4
#define CMD_ANALOG_WRITE 0x5

#define ERR_MALFORMED_PACKAGE 0x1
#define ERR_INVALID_CHECKSUM 0x2
#define ERR_INVALID_PIN 0x3
#define ERR_INVALID_COMMAND 0x4

void sendResponse(uint8_t cmd, uint8_t result, bool success)
{
    // set error bit
    cmd = (cmd & ~PKG_RESPONSE_ERROR_MASK) | (success ? 0 : PKG_RESPONSE_ERROR_MASK);

    // send package
    uint8_t chksum = PKG_START_BYTE + cmd + result + PKG_CHKSUM_PLACEHOLDER_VALUE + PKG_END_BYTE;
    Serial.write(PKG_START_BYTE);
    Serial.write(cmd);
    Serial.write(result);
    Serial.write(chksum);
    Serial.write(PKG_END_BYTE);
    Serial.flush();
}

void executeCommand(uint8_t cmd, uint8_t pin, uint8_t val)
{
    // validate pin number
    if (!IS_VALID_PIN(pin))
    {
        sendResponse(cmd, ERR_INVALID_PIN, false);
        return;
    }

    // execute command
    switch (cmd)
    {
    case CMD_DIGITAL_READ:
    case CMD_DIGITAL_READ_PULLUP:
        pinMode(pin, cmd == CMD_DIGITAL_READ_PULLUP ? INPUT_PULLUP : INPUT);
        sendResponse(cmd, digitalRead(pin) ? 1 : 0, true);
        break;
    case CMD_DIGITAL_WRITE:
        pinMode(pin, OUTPUT);
        digitalWrite(pin, val);
        sendResponse(cmd, val, true);
        break;
    case CMD_ANALOG_READ:
        pinMode(pin, INPUT);
        sendResponse(cmd, analogRead(pin) & 0xFF, true);
        break;
    case CMD_ANALOG_WRITE:
        pinMode(pin, OUTPUT);
        analogWrite(pin, val);
        sendResponse(cmd, val, true);
        break;
    default:
        // invalid command, ignore it
        sendResponse(cmd, ERR_INVALID_COMMAND, false);
        break;
    }
}

void acceptPackage()
{
    // check for start of packet
    uint8_t start = Serial.read();
    if (start != PKG_START_BYTE)
    {
        return;
    }

    // wait for the remaining data
    while (Serial.available() < 5)
        ;

    // read the packet data
    uint8_t cmd = Serial.read();
    uint8_t pin = Serial.read();
    uint8_t val = Serial.read();

    // read package epilogue
    uint8_t chksum = Serial.read();
    uint8_t end = Serial.read();
    if (end != PKG_END_BYTE)
    {
        // invalid package, ignore it
        sendResponse(cmd, ERR_MALFORMED_PACKAGE, false);
        return;
    }

    // validate checksum
    uint8_t expectedChksum = PKG_START_BYTE + cmd + pin + val + PKG_CHKSUM_PLACEHOLDER_VALUE + PKG_END_BYTE;
    if (chksum != expectedChksum)
    {
        // invalid checksum, ignore it
        sendResponse(cmd, ERR_INVALID_CHECKSUM, false);
        return;
    }

    // execute command
    executeCommand(cmd, pin, val);
}

void setup()
{
    // initialize serial
    Serial.begin(COMM_BAUD_RATE);
}

void loop()
{
    if (Serial.available() > 0)
    {
        acceptPackage();
    }
}
