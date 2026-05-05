$body = @{ 
    message = '请帮我计算 123 乘以 456'; 
    session_id = 'test-session' 
} | ConvertTo-Json

Write-Host "发送请求..."
Write-Host "请求体: $body"

$response = Invoke-RestMethod -Uri 'http://127.0.0.1:8081/api/chat' -Method Post -Body ([System.Text.Encoding]::UTF8.GetBytes($body)) -ContentType 'application/json'

Write-Host "响应:"
$response | ConvertTo-Json
