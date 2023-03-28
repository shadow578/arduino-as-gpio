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

#define PKG_START_BYTE '{'
#define PKG_END_BYTE '}'
#define PKG_CHKSUM_PLACEHOLDER_VALUE 0x00
#define PKG_RESPONSE_ERROR_MASK 0x80

#define CMD_DIGITAL_READ 0x1
#define CMD_DIGITAL_READ_PULLUP 0x2
#define CMD_DIGITAL_WRITE 0x3

#define CMD_ANALOG_READ 0x4
#define CMD_ANALOG_WRITE 0x5

void sendResponse(uint8_t cmd, uint8_t result, bool success)
{
    // set success bit
    cmd = (cmd & ~PKG_RESPONSE_ERROR_MASK) | (success ? 0 : PKG_RESPONSE_ERROR_MASK);

    // send package
    uint8_t chksum = PKG_START_BYTE + cmd + result + PKG_CHKSUM_PLACEHOLDER_VALUE + PKG_END_BYTE;
    Serial.write(PKG_START_BYTE);
    Serial.write(cmd);
    Serial.write(result);
    Serial.write(chksum);
    Serial.write(PKG_END_BYTE);
}

void executeCommand(uint8_t cmd, uint8_t pin, uint8_t val)
{
    // validate pin number
    if (!IS_VALID_PIN(pin))
    {
        sendResponse(cmd, 0, false);
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
        sendResponse(cmd, 0, false);
        break;
    }
}

void acceptPackage()
{
    // wait for start of packet
    while (Serial.available() > 0 && Serial.read() != PKG_START_BYTE)
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
        sendResponse(cmd, 0, false);
        return;
    }

    // validate checksum
    uint8_t expectedChksum = PKG_START_BYTE + cmd + pin + val + PKG_CHKSUM_PLACEHOLDER_VALUE + PKG_END_BYTE;
    if (chksum != expectedChksum)
    {
        // invalid checksum, ignore it
        sendResponse(cmd, 0, false);
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
    acceptPackage();
}
