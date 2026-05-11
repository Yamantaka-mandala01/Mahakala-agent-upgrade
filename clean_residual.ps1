Write-Host "=== 清理已卸载程序的残留文件夹 ==="
Write-Host ""

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
        Write-Host ("正在删除: " + $t)
        try {
            Remove-Item $t -Recurse -Force -ErrorAction SilentlyContinue
            if (Test-Path $t) {
                Write-Host ("  删除失败，需要管理员权限: " + $t)
            } else {
                Write-Host ("  删除成功!")
            }
        } catch {
            Write-Host ("  删除出错: " + $_.Exception.Message)
        }
    } else {
        Write-Host ("目录不存在: " + $t)
    }
}

Write-Host ""
Write-Host "=== 清理其他常见残留 ==="
# Clean prefetch files
$prefetch = "C:\Windows\Prefetch"
if (Test-Path $prefetch) {
    Write-Host ("正在清理Prefetch...")
    Get-ChildItem $prefetch -Force -ErrorAction SilentlyContinue | Where-Object { !$_.PSIsContainer } | ForEach-Object {
        Remove-Item $_.FullName -Force -ErrorAction SilentlyContinue
    }
}

# Clean recent documents
$recent = "C:\Users\" + [Environment]::UserName + "\AppData\Roaming\Microsoft\Windows\Recent"
if (Test-Path $recent) {
    Write-Host ("正在清理Recent文件...")
    Get-ChildItem $recent -Force -ErrorAction SilentlyContinue | Where-Object { !$_.PSIsContainer } | ForEach-Object {
        Remove-Item $_.FullName -Force -ErrorAction SilentlyContinue
    }
}

Write-Host ""
Write-Host "清理完成!"
