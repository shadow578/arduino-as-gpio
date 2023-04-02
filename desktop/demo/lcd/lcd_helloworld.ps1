#
# short demo of using arduino-as-GPIO to drive a 16x2 LCD display connected via PCF8574 IIC expander
# Set the $Port variable to the serial port of your Arduino
# Set the $Address variable to the IIC address of your PCF8574 expander
#
# only tested with PowerShell 7.3!
#
param (
    [string] $Port = "COM3",
    [string] $Binary = ".\target\release\agpio.exe",
    [int] $Address = 0x3E
)

# import lcd module
. $PSScriptRoot\lcd.ps1

# lcd module HAL: write to PCF8574 function
$global:IICWriteCount = 0;
function Write-PCF8574([byte] $Byte) {
    & $Binary $Port i2c $Address write $Byte --stop | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to write to IIC"
    }


    $global:IICWriteCount += 1
    Write-Host "`rIIC write count: $global:IICWriteCount" -NoNewline
}

function Main {
    # initialize the LCD and turn on the backlight
    Initialize-LCD
    Set-LCDBacklight -On

    # print "Hello World" on the LCD, for 5 seconds
    Write-LCD -String "Hello World"
    Start-Sleep -Seconds 5

    # clear the LCD and turn off the cursor
    Clear-LCDDisplay
    Set-LCDDisplay -On -Cursor:$false -Blink:$false

    # print the computer name and ip address on the LCD, for 5 seconds
    Write-LCD -String "$env:COMPUTERNAME"

    Set-LCDCursor -Row 1 -Column 0
    $ipAddress = (Get-NetIPAddress | Where-Object { $_.AddressState -eq "Preferred" -and $_.ValidLifetime -lt "24:00:00" }).IPAddress
    Write-LCD -String "$ipAddress"

    Start-Sleep -Seconds 5

    # count down from 9 to 0 on the LCD, updating only the changed digit
    Clear-LCDDisplay
    Write-LCD -String "Countdown: 9"
    for ($i = 9; $i -ge 0; $i--) {
        Set-LCDCursor -Row 0 -Column 11
        Write-LCD -String $i
        Start-Sleep -Seconds 1
    }

    # show a loading bar on the LCD
    Clear-LCDDisplay
    Write-LCD -String "    Loading"
    Set-LCDCursor -Row 1 -Column 0
    Write-LCD -String "["
    Set-LCDCursor -Row 1 -Column 15
    Write-LCD -String "]"
    Set-LCDCursor -Row 1 -Column 1
    for ($i = 0; $i -lt 14; $i++) {
        Write-LCD -String "="
        Start-Sleep -Milliseconds 250
    }

    Start-sleep -Seconds 1

    # use the LCD as a clock forever
    Clear-LCDDisplay
    Set-LCDCursor -Row 1 -Column 3
    Write-LCD -String (Get-Date -Format "dd.MM.yyyy")
    while ($true) {
        Set-LCDCursor -Row 0 -Column 4
        Write-LCD -String (Get-Date -Format "HH:mm:ss")
        #Start-Sleep -Seconds 1
    }
}
Main
