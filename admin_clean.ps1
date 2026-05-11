$targets = @(
    "C:\Program Files\McAfee",
    "C:\Program Files\HPCommRecovery",
    "C:\Program Files\Uninstall Information",
    "C:\Program Files (x86)\Kingsoft Office Software",
    "C:\Program Files (x86)\Online Services",
    "C:\ProgramData\Package Cache"
)

foreach ($t in $targets) {
    if (Test-Path $t) {
        Write-Host ("Deleting: " + $t)
        takeown /f $t /r /d y 2>$null
        icacls $t /grant administrators:F /t 2>$null
        Remove-Item $t -Recurse -Force
        if (Test-Path $t) {
            Write-Host ("  FAILED: " + $t)
        } else {
            Write-Host ("  SUCCESS: " + $t)
        }
    }
}

$drive = Get-WmiObject Win32_LogicalDisk -Filter "DeviceID='C:'"
$free = [math]::Round($drive.FreeSpace / 1GB, 2)
Write-Host ("C: Free Space: " + $free + " GB")
