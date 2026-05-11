$user = [Environment]::UserName
$tempPath = "C:\Users\$user\AppData\Local\Temp"

Write-Host "Cleaning Temp files..."
$count = 0
$totalSize = 0
Get-ChildItem $tempPath -Force -ErrorAction SilentlyContinue | ForEach-Object {
    try {
        if ($_.PSIsContainer) {
            Remove-Item $_.FullName -Recurse -Force -ErrorAction SilentlyContinue
        } else {
            Remove-Item $_.FullName -Force -ErrorAction SilentlyContinue
        }
        $count++
    } catch {}
}
Write-Host ("Cleaned " + $count + " items from Temp folder")
Write-Host ""

# Also clean Windows Temp
Write-Host "Cleaning Windows Temp..."
$winTemp = "C:\Windows\Temp"
if (Test-Path $winTemp) {
    $count2 = 0
    Get-ChildItem $winTemp -Force -ErrorAction SilentlyContinue | ForEach-Object {
        try {
            if ($_.PSIsContainer) {
                Remove-Item $_.FullName -Recurse -Force -ErrorAction SilentlyContinue
            } else {
                Remove-Item $_.FullName -Force -ErrorAction SilentlyContinue
            }
            $count2++
        } catch {}
    }
    Write-Host ("Cleaned " + $count2 + " items from Windows Temp")
}

# Check disk space after cleaning
$drive = Get-WmiObject Win32_LogicalDisk -Filter "DeviceID='C:'"
$free = [math]::Round($drive.FreeSpace / 1GB, 2)
Write-Host ""
Write-Host "Current free space on C: " + $free + " GB"
