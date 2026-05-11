$drive = Get-WmiObject Win32_LogicalDisk -Filter "DeviceID='C:'"
$total = [math]::Round($drive.Size / 1GB, 2)
$free = [math]::Round($drive.FreeSpace / 1GB, 2)
$used = [math]::Round(($drive.Size - $drive.FreeSpace) / 1GB, 2)
$percent = [math]::Round($drive.FreeSpace / $drive.Size * 100, 2)
Write-Host "C盘总空间: $total GB"
Write-Host "已用空间: $used GB"
Write-Host "可用空间: $free GB"
Write-Host "剩余比例: $percent%"
