# AI Chat API 测试脚本

# 配置 - 请在这里填写你的 API Key
$API_KEY = "your-api-key-here"  # 例如: sk-xxx 或 anthropic key
$API_URL = "https://api.deepseek.com/v1"  # DeepSeek API URL
$MODEL = "deepseek-chat"
$PROVIDER = "deepseek"

$BASE_URL = "http://127.0.0.1:8081"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "AI Chat API 测试" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 测试 1: 检查服务器状态
Write-Host "`n[测试 1] 检查服务器状态..." -ForegroundColor Yellow
try {
    $status = Invoke-RestMethod -Uri "$BASE_URL/api/status" -Method Get
    Write-Host "服务器状态: $($status.status)" -ForegroundColor Green
    Write-Host "模型: $($status.model)" -ForegroundColor Green
    Write-Host "可用工具: $($status.available_tools)" -ForegroundColor Green
} catch {
    Write-Host "错误: $_" -ForegroundColor Red
}

# 测试 2: 获取工具列表
Write-Host "`n[测试 2] 获取工具列表..." -ForegroundColor Yellow
try {
    $tools = Invoke-RestMethod -Uri "$BASE_URL/api/tools" -Method Get
    Write-Host "已注册工具数量: $($tools.tools.Count)" -ForegroundColor Green
    Write-Host "工具列表:" -ForegroundColor Green
    foreach ($tool in $tools.tools) {
        Write-Host "  - $($tool.function.name): $($tool.function.description)" -ForegroundColor Green
    }
} catch {
    Write-Host "错误: $_" -ForegroundColor Red
}

# 测试 3: 获取技能列表
Write-Host "`n[测试 3] 获取技能列表..." -ForegroundColor Yellow
try {
    $skills = Invoke-RestMethod -Uri "$BASE_URL/api/skills" -Method Get
    Write-Host "可用技能数量: $($skills.skills.Count)" -ForegroundColor Green
    Write-Host "技能列表:" -ForegroundColor Green
    foreach ($skill in $skills.skills) {
        Write-Host "  - $($skill.name) [$($skill.category)]: $($skill.desc)" -ForegroundColor Green
    }
} catch {
    Write-Host "错误: $_" -ForegroundColor Red
}

# 测试 4: 获取插件列表
Write-Host "`n[测试 4] 获取插件列表..." -ForegroundColor Yellow
try {
    $plugins = Invoke-RestMethod -Uri "$BASE_URL/api/plugins" -Method Get
    Write-Host "可用插件数量: $($plugins.plugins.Count)" -ForegroundColor Green
    Write-Host "插件列表:" -ForegroundColor Green
    foreach ($plugin in $plugins.plugins) {
        Write-Host "  - $($plugin.name) [$($plugin.status)]: $($plugin.desc)" -ForegroundColor Green
    }
} catch {
    Write-Host "错误: $_" -ForegroundColor Red
}

# 测试 5: AI 聊天（需要 API Key）
Write-Host "`n[测试 5] AI 聊天测试（需要 API Key）..." -ForegroundColor Yellow
if ($API_KEY -eq "your-api-key-here") {
    Write-Host "跳过: 请先配置 API Key" -ForegroundColor Magenta
} else {
    try {
        $chatBody = @{
            message = "请帮我计算 123 乘以 456"
            session_id = "test-session-1"
            apiKey = $API_KEY
            apiUrl = $API_URL
            model = $MODEL
            provider = $PROVIDER
        } | ConvertTo-Json

        Write-Host "发送消息: 请帮我计算 123 乘以 456" -ForegroundColor Yellow
        $response = Invoke-RestMethod -Uri "$BASE_URL/api/chat" -Method Post -Body ([System.Text.Encoding]::UTF8.GetBytes($chatBody)) -ContentType 'application/json'
        
        Write-Host "AI 响应:" -ForegroundColor Green
        Write-Host "$($response.response)" -ForegroundColor Green
    } catch {
        Write-Host "错误: $_" -ForegroundColor Red
    }
}

# 测试 6: AI 聊天 - 工具调用测试
Write-Host "`n[测试 6] AI 工具调用测试（需要 API Key）..." -ForegroundColor Yellow
if ($API_KEY -eq "your-api-key-here") {
    Write-Host "跳过: 请先配置 API Key" -ForegroundColor Magenta
} else {
    try {
        $toolChatBody = @{
            message = "请列出当前目录的文件"
            session_id = "test-session-2"
            apiKey = $API_KEY
            apiUrl = $API_URL
            model = $MODEL
            provider = $PROVIDER
        } | ConvertTo-Json

        Write-Host "发送消息: 请列出当前目录的文件" -ForegroundColor Yellow
        $response = Invoke-RestMethod -Uri "$BASE_URL/api/chat" -Method Post -Body ([System.Text.Encoding]::UTF8.GetBytes($toolChatBody)) -ContentType 'application/json'
        
        Write-Host "AI 响应:" -ForegroundColor Green
        Write-Host "$($response.response)" -ForegroundColor Green
    } catch {
        Write-Host "错误: $_" -ForegroundColor Red
    }
}

# 测试 7: 配置 API
Write-Host "`n[测试 7] 获取配置..." -ForegroundColor Yellow
try {
    $config = Invoke-RestMethod -Uri "$BASE_URL/api/config" -Method Get
    Write-Host "当前配置:" -ForegroundColor Green
    Write-Host "  提供商: $($config.provider)" -ForegroundColor Green
    Write-Host "  模型: $($config.model)" -ForegroundColor Green
    Write-Host "  语言: $($config.language)" -ForegroundColor Green
    Write-Host "  主题: $($config.theme)" -ForegroundColor Green
} catch {
    Write-Host "错误: $_" -ForegroundColor Red
}

# 测试 8: 会话管理
Write-Host "`n[测试 8] 会话管理测试..." -ForegroundColor Yellow
try {
    $sessions = Invoke-RestMethod -Uri "$BASE_URL/api/sessions" -Method Get
    Write-Host "会话数量: $($sessions.sessions.Count)" -ForegroundColor Green
    foreach ($session in $sessions.sessions) {
        Write-Host "  - ID: $($session.id), 标题: $($session.title)" -ForegroundColor Green
    }
} catch {
    Write-Host "错误: $_" -ForegroundColor Red
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "测试完成!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
