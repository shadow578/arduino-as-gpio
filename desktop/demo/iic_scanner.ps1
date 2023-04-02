param (
    [string] $Port = "COM3",
    [string] $Binary = "..\target\release\agpio.exe"
)

function Test-DeviceAddress([int] $Address) {
    & $Binary $Port i2c $Address write --stop 2>&1 | Out-Null
    return $LASTEXITCODE -eq 0
}

function Main {
    for ($i = 0; $i -lt 128; $i++) {
        if (Test-DeviceAddress $i) {
            Write-Host "Found device at address $i"
        }
    }
}
Main