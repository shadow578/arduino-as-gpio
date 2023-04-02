$global:LCDBacklightOn = $false

#
# Public API
#

<#
.SYNOPSIS
Initialize the LCD display
#>
function Initialize-LCD() {
    # -- init 4 bit mode --
    # D7=0, D6=0, D5=1, D4=1, DB=0, RS, RW=0 EN=1
    Write-LCDNibble -Nibble 0b0011 -RS
    Write-LCDNibble -Nibble 0b0011

    # again
    Write-LCDNibble -Nibble 0b0011 -RS
    Write-LCDNibble -Nibble 0b0011

    # again
    Write-LCDNibble -Nibble 0b0011 -RS
    Write-LCDNibble -Nibble 0b0011

    # switch to 4-bit mode
    Write-LCDNibble -Nibble 0b0010 -RS
    Write-LCDNibble -Nibble 0b0010

    # -- setup display --
    # 4 bit mode; 2 lines; char5x8
    Write-LCDByte -Byte 0b000101000

    # -- turn display off --
    Set-LCDDisplay

    # -- clear display--
    Clear-LCDDisplay

    # -- set cursor direction --
    # D7-D4=0; D3-D0=0110 print left to right
    Write-LCDByte -Byte 0b00000110

    # -- turn on display and backlight --
    Set-LCDDisplay -On -Cursor -Blink
}

<#
.SYNOPSIS
Set LCD parameters

.PARAMETER On
turn display on / off

.PARAMETER Cursor
turn cursor on / off

.PARAMETER Blink
turn cursor blink on / off
#>
function Set-LCDDisplay([switch] $On, [switch] $Cursor, [switch] $Blink) {
    # D7-D4=0; D3=1; D2=display_on, D1=cursor_on, D0=cursor_blink
    $Byte = 0b00001000
    if ($On) {
        $Byte = $Byte -bor (1 -shl 2)
    }
    if ($Cursor) {
        $Byte = $Byte -bor (1 -shl 1)
    }
    if ($Blink) {
        $Byte = $Byte -bor (1 -shl 0)
    }

    Write-LCDByte -Byte $Byte
}

<#
.SYNOPSIS
set lcd cursor position


.PARAMETER Row
cursor row

.PARAMETER Column
cursor column
#>
function Set-LCDCursor([int] $Row, [int] $Column) {
    [byte]$CMD = 1 -shl 7
    switch ($Row) {
        0 { $CMD = $CMD -bor 0b00000000 }
        1 { $CMD = $CMD -bor 0b01000000 }
        2 { $CMD = $CMD -bor 0b00010100 }
        3 { $CMD = $CMD -bor 0b01010100 }
        default { throw "Invalid row $Row" }
    }

    if ($Column -gt 16) {
        throw "Invalid column $Column"
    }

    $CMD = $CMD -bor $Column
    Write-LCDByte -Byte $CMD -BL -RS
}

<#
.SYNOPSIS
clear lcd display
#>
function Clear-LCDDisplay() {
    # D0=display_clear
    Write-LCDByte -Byte 0b00000001

    # D1=home cursor
    Write-LCDByte -Byte 0b00000010
}

<#
.SYNOPSIS
write a single character to the LCD

.PARAMETER Character
the char to print
#>
function Write-LCDCharacter([char] $Character) {
    Write-LCDByte -Byte $Character -EN
}

<#
.SYNOPSIS
write a string to the LCD

.PARAMETER String
the string to write
#>
function Write-LCD([string] $String) {
    foreach ($Character in $String.ToCharArray()) {
        Write-LCDCharacter -Character $Character
    }
}

function Set-LCDBacklight([switch] $On) {
    $global:LCDBacklightOn = $On
}

#
# Internal API
#
function Write-LCDNibble([byte] $Nibble, 
    [switch] $BL, 
    [switch] $RS, 
    [switch] $RW, 
    [switch] $EN) {
    # RS, RW, EN are on PCF8574 pins 0, 1, 2
    $Byte = ($Nibble -band 0xF) -shl 4
    if ($BL -or $global:LCDBacklightOn) {
        $Byte = $Byte -bor (1 -shl 3)
    }
    if ($RS) {
        $Byte = $Byte -bor (1 -shl 2)
    }
    if ($RW) {
        $Byte = $Byte -bor (1 -shl 1)
    }
    if ($EN) {
        $Byte = $Byte -bor (1 -shl 0)
    }

    Write-PCF8574 -Byte $Byte
}

function Write-LCDByte([byte] $Byte, [switch] $BL, [switch] $RW, [switch] $EN) {
    # pulse RS on every nibble
    Write-LCDNibble -Nibble ($Byte -shr 4) -BL:$BL -RS:$true -RW:$RW -EN:$EN 
    Write-LCDNibble -Nibble ($Byte -shr 4) -BL:$BL -RS:$false -RW:$RW -EN:$EN

    Write-LCDNibble -Nibble ($Byte -shr 0) -BL:$BL -RS:$true -RW:$RW -EN:$EN 
    Write-LCDNibble -Nibble ($Byte -shr 0) -BL:$BL -RS:$false -RW:$RW -EN:$EN
}

