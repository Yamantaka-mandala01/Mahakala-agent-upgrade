$user = [Environment]::UserName
Write-Host "=== Check Residual Directories ==="
Write-Host "Current user: $user"
Write-Host ""

$paths = @(
    "C:\ProgramData",
    "C:\Users\$user\AppData\Local",
    "C:\Users\$user\AppData\Roaming",
    "C:\Users\$user\AppData\Local\Temp"
)

foreach ($p in $paths) {
    if (Test-Path $p) {
        $files = Get-ChildItem $p -Recurse -Force -ErrorAction SilentlyContinue | Where-Object { !$_.PSIsContainer }
        $totalSize = ($files | Measure-Object -Property Length -Sum).Sum
        $sizeMB = [math]::Round($totalSize / 1MB, 2)
        $sizeGB = [math]::Round($totalSize / 1GB, 2)
        if ($sizeGB -ge 1) {
            Write-Host ($p + " : " + $sizeGB + " GB")
        } else {
            Write-Host ($p + " : " + $sizeMB + " MB")
        }
    } else {
        Write-Host ($p + " : Not found")
    }
}
