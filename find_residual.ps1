$user = [Environment]::UserName
Write-Host "=== 查找可能已卸载程序的残留文件夹 ==="
Write-Host ""

# Check common residual locations
$locations = @(
    "C:\Program Files",
    "C:\Program Files (x86)",
    "C:\ProgramData",
    "C:\Users\$user\AppData\Local",
    "C:\Users\$user\AppData\Roaming"
)

# Known software vendors that might have residuals
$vendorFolders = @(
    "C:\Users\$user\AppData\Local\Google",
    "C:\Users\$user\AppData\Local\Microsoft",
    "C:\Users\$user\AppData\Roaming\Microsoft",
    "C:\Users\$user\AppData\Local\Adobe",
    "C:\Users\$user\AppData\Roaming\Adobe",
    "C:\ProgramData\Microsoft",
    "C:\ProgramData\Package Cache",
    "C:\ProgramData\USOShared",
    "C:\ProgramData\SoftwareDistribution"
)

foreach ($loc in $vendorFolders) {
    if (Test-Path $loc) {
        $files = Get-ChildItem $loc -Recurse -Force -ErrorAction SilentlyContinue | Where-Object { !$_.PSIsContainer }
        $totalSize = ($files | Measure-Object -Property Length -Sum).Sum
        $sizeMB = [math]::Round($totalSize / 1MB, 2)
        if ($sizeMB -gt 10) {
            Write-Host ($loc + " : " + $sizeMB + " MB")
        } else {
            Write-Host ($loc + " : " + $sizeMB + " MB (较小)")
        }
    }
}

# Check for empty folders in Program Files that might indicate uninstalled programs
Write-Host ""
Write-Host "=== 检查可能已卸载程序的空文件夹 ==="
$progFiles = @("C:\Program Files", "C:\Program Files (x86)")
foreach ($pf in $progFiles) {
    if (Test-Path $pf) {
        Get-ChildItem $pf -Force -ErrorAction SilentlyContinue | Where-Object { $_.PSIsContainer } | ForEach-Object {
            $folder = $_.FullName
            $files = Get-ChildItem $folder -Recurse -Force -ErrorAction SilentlyContinue | Where-Object { !$_.PSIsContainer }
            $totalSize = ($files | Measure-Object -Property Length -Sum).Sum
            $sizeMB = [math]::Round($totalSize / 1MB, 2)
            if ($sizeMB -lt 1) {
                Write-Host ($folder + " : 可疑残留(小于1MB)")
            }
        }
    }
}
