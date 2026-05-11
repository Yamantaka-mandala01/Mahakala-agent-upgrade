$drive = Get-WmiObject Win32_LogicalDisk -Filter "DeviceID='C:'"
$free = [math]::Round($drive.FreeSpace / 1GB, 2)
$used = [math]::Round(($drive.Size - $drive.FreeSpace) / 1GB, 2)
$total = [math]::Round($drive.Size / 1GB, 2)
Write-Host ("C: Total = " + $total + " GB")
Write-Host ("C: Used = " + $used + " GB")
Write-Host ("C: Free = " + $free + " GB")
