param (
    [string] $Port = "COM3",
    [string] $Binary = "..\target\release\agpio.exe",
    [int] $Iterations = 100,
    [int] $Pin = 13
)

function Test-DigitalWrite {
    # 1. Measure time to execute 100 digital writes
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    for ($i = 0; $i -lt $Iterations; $i++) {
        & $Binary $Port write $Pin ($i % 2) | Out-Null
    }
    $sw.Stop()
    
    # 2. Calculate average time per command
    $avg = $sw.ElapsedMilliseconds / $Iterations
    
    # 3. Print result
    Write-Host "Total time for $Iterations digital writes: $($sw.ElapsedMilliseconds) ms"
    Write-Host "Average time per write: $avg ms"
}

function Test-AnalogWrite {
    # 1. Measure time to execute 100 digital writes
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    for ($i = 0; $i -lt $Iterations; $i++) {
        & $Binary $Port write $Pin ($i % 256) --analog | Out-Null
    }
    $sw.Stop()
    
    # 2. Calculate average time per command
    $avg = $sw.ElapsedMilliseconds / $Iterations
    
    # 3. Print result
    Write-Host "Total time for $Iterations analog writes: $($sw.ElapsedMilliseconds) ms"
    Write-Host "Average time per write: $avg ms"
}

function Test-Toggle {
    # 1. Measure time to execute 100 digital toggles
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    for ($i = 0; $i -lt $Iterations; $i++) {
        & $Binary $Port toggle $Pin | Out-Null
    }
    $sw.Stop()
    
    # 2. Calculate average time per command
    $avg = $sw.ElapsedMilliseconds / $Iterations
    
    # 3. Print result
    Write-Host "Total time for $Iterations digital toggles: $($sw.ElapsedMilliseconds) ms"
    Write-Host "Average time per write: $avg ms"
}

function Main {
    Test-DigitalWrite
    Write-Host "-----------------"
    Test-Toggle
    Write-Host "-----------------"
    Test-AnalogWrite
}
Main
