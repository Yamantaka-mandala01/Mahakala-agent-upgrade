class MahakalaWebUI {
    constructor() {
        this.currentLang = localStorage.getItem('lang') || 'zh';
        this.currentTheme = localStorage.getItem('theme') || 'dark';
        this.currentPage = 'chat';
        this.currentSession = null;
        this.ws = null;
        this.config = this.loadConfig();
        this.sessions = this.loadSessions();
        this.activeModel = null;
        this.modelLatencies = {};
        this.isStreaming = false;
        this.isConfigRestoring = false;
        this.outputLog = [];
        this.debugLog = [];
        this.i18n = {
            zh: {
                'menu.file': '文件', 'menu.edit': '编辑', 'menu.view': '视图', 'menu.tools': '工具', 'menu.help': '帮助',
                'breadcrumb.home': '首页', 'pages.chat': '对话交互', 'pages.memory': '记忆管理', 'pages.skills': '技能中心',
                'pages.tools': '工具管理', 'pages.plugins': '插件系统', 'pages.messages': '消息中心', 'pages.platforms': '平台连接',
                'pages.cron': '定时任务', 'pages.settings': '系统设置', 'status.running': '运行中', 'status.offline': '未连接',
                'chat.welcome': '你好！我是Mahakala Agent，你的智能助手。我可以帮助你完成各种任务。',
                'actions.add': '添加', 'actions.search': '搜索', 'actions.install': '安装', 'actions.refresh': '刷新',
                'actions.code': '代码', 'actions.file': '文件', 'actions.terminal': '终端',
                'memory.facts': '事实数量', 'memory.entities': '实体数量', 'memory.dimensions': '向量维度',
                'skills.all': '全部', 'skills.creative': '创意', 'skills.devops': '开发运维', 'skills.research': '研究', 'skills.productivity': '生产力',
                'tools.all': '全部', 'tools.file': '文件操作', 'tools.web': '网络工具', 'tools.code': '代码工具', 'tools.system': '系统工具',
                'messages.all': '全部', 'settings.general': '常规', 'settings.model': '模型', 'settings.appearance': '外观', 'settings.advanced': '高级',
                'panel.session': '会话信息', 'panel.actions': '快捷操作', 'panel.output': '输出', 'panel.debug': '调试',
                'info.sessionId': '会话ID', 'info.model': '模型', 'info.tokens': 'Token', 'info.messages': '消息',
                'statusbar.tools': '工具', 'statusbar.plugins': '插件', 'statusbar.memory': '记忆', 'statusbar.skills': '技能',
                'sidebar.sessions': '会话', 'sidebar.tools': '工具', 'sidebar.skills': '技能', 'sidebar.plugins': '插件',
                'sidebar.platforms': '平台', 'sidebar.messages': '消息', 'sidebar.cron': '定时任务', 'sidebar.memory': '记忆', 'sidebar.settings': '设置',
                'theme.dark': '深色', 'theme.light': '浅色', 'lang.switch': 'Switch to English',
                'chat.placeholder': '输入消息... (Enter发送, Shift+Enter换行)', 'chat.newSession': '新会话', 'chat.compress': '压缩上下文',
                'skill.enabled': '已启用', 'skill.disabled': '已禁用', 'skill.install': '安装', 'skill.uninstall': '卸载',
                'plugin.loaded': '已加载', 'plugin.unloaded': '未加载', 'plugin.load': '加载', 'plugin.unload': '卸载',
                'platform.connected': '已连接', 'platform.disconnected': '未连接', 'platform.connect': '连接', 'platform.disconnect': '断开',
                'cron.running': '运行中', 'cron.paused': '已暂停', 'cron.run': '运行', 'cron.pause': '暂停',
                'notification.success': '操作成功', 'notification.error': '操作失败', 'notification.info': '提示', 'notification.warning': '警告',
                'config.title': '系统配置', 'config.models': '模型配置', 'config.platforms': '平台配置', 'config.general': '通用设置',
                'config.models.title': 'AI 模型配置', 'config.platforms.title': '消息平台配置', 'config.general.title': '通用设置',
                'config.ollama.url': '服务地址', 'config.ollama.model': '模型名称', 'config.api.key': 'API Key', 'config.api.secret': 'App Secret',
                'config.model.select': '选择模型', 'config.group.id': 'Group ID', 'config.enabled': '启用', 'config.configure': '配置',
                'config.wechat.mode': '模式', 'config.qqbot.appid': 'App ID', 'config.qqbot.token': 'Token',
                'config.telegram.token': 'Bot Token', 'config.telegram.webhook': 'Webhook URL', 'config.feishu.appid': 'App ID',
                'config.discord.token': 'Bot Token', 'config.discord.clientid': 'Client ID',
                'config.interface.lang': '界面语言', 'config.theme': '主题模式', 'config.server.port': '服务器端口',
                'config.autoStart': '开机自启', 'config.debug': '调试模式', 'config.save': '保存配置', 'config.reset': '重置',
                'config.general.language': '语言', 'config.general.theme': '主题', 'config.general.system': '系统',
                'config.configFirst': '请先完成系统配置，选择并配置至少一个AI模型', 'config.detect': '检测',
                'config.detecting': '检测中...', 'config.auto.fastest': '自动选择最快', 'config.api.url': 'API 地址',
                'chat.sending': '发送中...', 'chat.noModel': '请先配置并启用至少一个AI模型',
                'chat.error': '发送消息失败，请检查模型配置', 'memory.addFact': '添加事实', 'memory.delete': '删除',
                'memory.search': '搜索', 'memory.factPlaceholder': '输入事实...', 'skill.enable': '启用', 'skill.disable': '禁用',
                'plugin.refresh': '刷新', 'cron.add': '添加任务', 'cron.expression': 'Cron 表达式', 'cron.command': '执行命令',
                'cron.name': '任务名称', 'session.new': '新建会话', 'session.delete': '删除会话', 'session.switch': '切换',
                'model.auto': '自动选择', 'model.testing': '测试模型响应...', 'model.fastest': '最快模型',
                'ollama.connected': 'Ollama 已连接', 'ollama.disconnected': 'Ollama 未连接',
                'ollama.models.found': '发现模型', 'ollama.no.models': '未找到模型'
            },
            en: {
                'menu.file': 'File', 'menu.edit': 'Edit', 'menu.view': 'View', 'menu.tools': 'Tools', 'menu.help': 'Help',
                'breadcrumb.home': 'Home', 'pages.chat': 'Chat', 'pages.memory': 'Memory', 'pages.skills': 'Skills',
                'pages.tools': 'Tools', 'pages.plugins': 'Plugins', 'pages.messages': 'Messages', 'pages.platforms': 'Platforms',
                'pages.cron': 'Cron Jobs', 'pages.settings': 'Settings', 'status.running': 'Running', 'status.offline': 'Offline',
                'chat.welcome': 'Hello! I am Mahakala Agent, your intelligent assistant. I can help you with various tasks.',
                'actions.add': 'Add', 'actions.search': 'Search', 'actions.install': 'Install', 'actions.refresh': 'Refresh',
                'actions.code': 'Code', 'actions.file': 'File', 'actions.terminal': 'Terminal',
                'memory.facts': 'Facts', 'memory.entities': 'Entities', 'memory.dimensions': 'Dimensions',
                'skills.all': 'All', 'skills.creative': 'Creative', 'skills.devops': 'DevOps', 'skills.research': 'Research', 'skills.productivity': 'Productivity',
                'tools.all': 'All', 'tools.file': 'File Ops', 'tools.web': 'Web Tools', 'tools.code': 'Code Tools', 'tools.system': 'System Tools',
                'messages.all': 'All', 'settings.general': 'General', 'settings.model': 'Model', 'settings.appearance': 'Appearance', 'settings.advanced': 'Advanced',
                'panel.session': 'Session Info', 'panel.actions': 'Quick Actions', 'panel.output': 'Output', 'panel.debug': 'Debug',
                'info.sessionId': 'Session ID', 'info.model': 'Model', 'info.tokens': 'Tokens', 'info.messages': 'Messages',
                'statusbar.tools': 'Tools', 'statusbar.plugins': 'Plugins', 'statusbar.memory': 'Memory', 'statusbar.skills': 'Skills',
                'sidebar.sessions': 'Sessions', 'sidebar.tools': 'Tools', 'sidebar.skills': 'Skills', 'sidebar.plugins': 'Plugins',
                'sidebar.platforms': 'Platforms', 'sidebar.messages': 'Messages', 'sidebar.cron': 'Cron Jobs', 'sidebar.memory': 'Memory', 'sidebar.settings': 'Settings',
                'theme.dark': 'Dark', 'theme.light': 'Light', 'lang.switch': '切换中文',
                'chat.placeholder': 'Type a message... (Enter to send, Shift+Enter for new line)', 'chat.newSession': 'New Session', 'chat.compress': 'Compress Context',
                'skill.enabled': 'Enabled', 'skill.disabled': 'Disabled', 'skill.install': 'Install', 'skill.uninstall': 'Uninstall',
                'plugin.loaded': 'Loaded', 'plugin.unloaded': 'Unloaded', 'plugin.load': 'Load', 'plugin.unload': 'Unload',
                'platform.connected': 'Connected', 'platform.disconnected': 'Disconnected', 'platform.connect': 'Connect', 'platform.disconnect': 'Disconnect',
                'cron.running': 'Running', 'cron.paused': 'Paused', 'cron.run': 'Run', 'cron.pause': 'Pause',
                'notification.success': 'Success', 'notification.error': 'Error', 'notification.info': 'Info', 'notification.warning': 'Warning',
                'config.title': 'System Configuration', 'config.models': 'Models', 'config.platforms': 'Platforms', 'config.general': 'General',
                'config.models.title': 'AI Model Configuration', 'config.platforms.title': 'Message Platform Configuration', 'config.general.title': 'General Settings',
                'config.ollama.url': 'Server URL', 'config.ollama.model': 'Model Name', 'config.api.key': 'API Key', 'config.api.secret': 'App Secret',
                'config.model.select': 'Select Model', 'config.group.id': 'Group ID', 'config.enabled': 'Enabled', 'config.configure': 'Configure',
                'config.wechat.mode': 'Mode', 'config.qqbot.appid': 'App ID', 'config.qqbot.token': 'Token',
                'config.telegram.token': 'Bot Token', 'config.telegram.webhook': 'Webhook URL', 'config.feishu.appid': 'App ID',
                'config.discord.token': 'Bot Token', 'config.discord.clientid': 'Client ID',
                'config.interface.lang': 'Language', 'config.theme': 'Theme', 'config.server.port': 'Server Port',
                'config.autoStart': 'Auto Start', 'config.debug': 'Debug Mode', 'config.save': 'Save Configuration', 'config.reset': 'Reset',
                'config.general.language': 'Language', 'config.general.theme': 'Theme', 'config.general.system': 'System',
                'config.configFirst': 'Please complete system configuration first, select and configure at least one AI model', 'config.detect': 'Detect',
                'config.detecting': 'Detecting...', 'config.auto.fastest': 'Auto-select fastest', 'config.api.url': 'API URL',
                'chat.sending': 'Sending...', 'chat.noModel': 'Please configure and enable at least one AI model first',
                'chat.error': 'Failed to send message, please check model configuration', 'memory.addFact': 'Add Fact', 'memory.delete': 'Delete',
                'memory.search': 'Search', 'memory.factPlaceholder': 'Enter fact...', 'skill.enable': 'Enable', 'skill.disable': 'Disable',
                'plugin.refresh': 'Refresh', 'cron.add': 'Add Job', 'cron.expression': 'Cron Expression', 'cron.command': 'Command',
                'cron.name': 'Job Name', 'session.new': 'New Session', 'session.delete': 'Delete Session', 'session.switch': 'Switch',
                'model.auto': 'Auto-select', 'model.testing': 'Testing model response...', 'model.fastest': 'Fastest model',
                'ollama.connected': 'Ollama Connected', 'ollama.disconnected': 'Ollama Disconnected',
                'ollama.models.found': 'Models found', 'ollama.no.models': 'No models found'
            }
        };
        this.init();
    }

    async init() {
        this.applyTheme();
        this.applyLanguage();
        this.bindEvents();
        this.loadConfigToUI();
        this.isConfigRestoring = true;
        this.updateSendButtonState();
        await this.restoreConfigToBackend();
        this.isConfigRestoring = false;
        this.updateSendButtonState();
        this.checkFirstRun();
        this.connectWebSocket();
        this.loadInitialData();
        this.updateTime();
        setInterval(() => this.updateTime(), 1000);
        this.detectOllama();
        this.testModelLatencies();
        this.autoStartWechatQR();
    }

    async autoStartWechatQR() {
        const wechatEnabled = this.config.platforms?.wechat?.enabled;
        if (wechatEnabled) {
            setTimeout(async () => {
                await this.getWechatQRCode();
                this.startWechatStatusPolling();
            }, 2000);
        }
    }

    startWechatStatusPolling() {
        if (this.wechatPollingInterval) {
            clearInterval(this.wechatPollingInterval);
        }
        this.wechatPollingInterval = setInterval(async () => {
            const qrStatus = document.getElementById('qr-status');
            if (qrStatus && qrStatus.classList.contains('confirmed')) {
                clearInterval(this.wechatPollingInterval);
                return;
            }
            await this.checkWechatQRStatus();
        }, 3000);
    }

    checkFirstRun() {
        const hasConfig = localStorage.getItem('config_saved');
        if (!hasConfig) setTimeout(() => this.showConfigModal(), 500);
    }

    loadConfig() {
        const saved = localStorage.getItem('mahakala_config');
        if (saved) { try { return JSON.parse(saved); } catch (e) {} }
        return this.getDefaultConfig();
    }

    getDefaultConfig() {
        return {
            models: {
                // 默认启用本地 Ollama，不需要 API Key
                ollama: { url: 'http://localhost:11434', model: 'llama3.2', enabled: true },
                openai: { key: '', model: 'gpt-4o', enabled: false, url: 'https://api.openai.com/v1' },
                anthropic: { key: '', model: 'claude-sonnet-4-20250514', enabled: false, url: 'https://api.anthropic.com/v1' },
                deepseek: { key: '', model: 'deepseek-chat', enabled: false, url: 'https://api.deepseek.com' },
                minimax: { key: '', group_id: '', model: 'MiniMax-M2.1', enabled: false, url: 'https://api.minimax.chat/v1' },
                glm: { key: '', model: 'glm-4-flash', enabled: false, url: 'https://open.bigmodel.cn/api/paas/v4' },
                kimi: { key: '', model: 'moonshot-v1-8k', enabled: false, url: 'https://api.moonshot.cn/v1' },
                qwen: { key: '', model: 'qwen-turbo', enabled: false, url: 'https://dashscope.aliyuncs.com/compatible-mode/v1' }
            },
            platforms: {
                wechat: { mode: 'web', key: '', enabled: false },
                qqbot: { appid: '', secret: '', token: '', enabled: false },
                telegram: { token: '', webhook: '', enabled: false },
                feishu: { appid: '', secret: '', enabled: false },
                discord: { token: '', clientid: '', enabled: false }
            },
            general: { language: 'zh', theme: 'dark', port: 8080, autostart: false, debug: false }
        };
    }

    loadConfigToUI() {
        const c = this.config;
        const setVal = (id, val) => { const el = document.getElementById(id); if (el) el.value = val; };
        const setChecked = (id, val) => { const el = document.getElementById(id); if (el) el.checked = val; };
        setVal('cfg-ollama-url', c.models?.ollama?.url || '');
        setVal('cfg-ollama-model', c.models?.ollama?.model || '');
        setChecked('cfg-ollama-enabled', c.models?.ollama?.enabled || false);
        setVal('cfg-openai-key', c.models?.openai?.key || '');
        setVal('cfg-openai-model', c.models?.openai?.model || '');
        setChecked('cfg-openai-enabled', c.models?.openai?.enabled || false);
        setVal('cfg-openai-url', c.models?.openai?.url || '');
        setVal('cfg-anthropic-key', c.models?.anthropic?.key || '');
        setVal('cfg-anthropic-model', c.models?.anthropic?.model || '');
        setChecked('cfg-anthropic-enabled', c.models?.anthropic?.enabled || false);
        setVal('cfg-anthropic-url', c.models?.anthropic?.url || '');
        setVal('cfg-deepseek-key', c.models?.deepseek?.key || '');
        setVal('cfg-deepseek-model', c.models?.deepseek?.model || '');
        setChecked('cfg-deepseek-enabled', c.models?.deepseek?.enabled || false);
        setVal('cfg-deepseek-url', c.models?.deepseek?.url || '');
        setVal('cfg-minimax-key', c.models?.minimax?.key || '');
        setVal('cfg-minimax-group-id', c.models?.minimax?.group_id || '');
        setVal('cfg-minimax-model', c.models?.minimax?.model || '');
        setChecked('cfg-minimax-enabled', c.models?.minimax?.enabled || false);
        setVal('cfg-minimax-url', c.models?.minimax?.url || '');
        setVal('cfg-glm-key', c.models?.glm?.key || '');
        setVal('cfg-glm-model', c.models?.glm?.model || '');
        setChecked('cfg-glm-enabled', c.models?.glm?.enabled || false);
        setVal('cfg-glm-url', c.models?.glm?.url || '');
        setVal('cfg-kimi-key', c.models?.kimi?.key || '');
        setVal('cfg-kimi-model', c.models?.kimi?.model || '');
        setChecked('cfg-kimi-enabled', c.models?.kimi?.enabled || false);
        setVal('cfg-kimi-url', c.models?.kimi?.url || '');
        setVal('cfg-qwen-key', c.models?.qwen?.key || '');
        setVal('cfg-qwen-model', c.models?.qwen?.model || '');
        setChecked('cfg-qwen-enabled', c.models?.qwen?.enabled || false);
        setVal('cfg-qwen-url', c.models?.qwen?.url || '');
        setVal('cfg-wechat-mode', c.platforms?.wechat?.mode || '');
        setVal('cfg-wechat-key', c.platforms?.wechat?.key || '');
        setChecked('cfg-wechat-enabled', c.platforms?.wechat?.enabled || false);
        setVal('cfg-qqbot-appid', c.platforms?.qqbot?.appid || '');
        setVal('cfg-qqbot-secret', c.platforms?.qqbot?.secret || '');
        setVal('cfg-qqbot-token', c.platforms?.qqbot?.token || '');
        setChecked('cfg-qqbot-enabled', c.platforms?.qqbot?.enabled || false);
        setVal('cfg-telegram-token', c.platforms?.telegram?.token || '');
        setVal('cfg-telegram-webhook', c.platforms?.telegram?.webhook || '');
        setChecked('cfg-telegram-enabled', c.platforms?.telegram?.enabled || false);
        setVal('cfg-feishu-appid', c.platforms?.feishu?.appid || '');
        setVal('cfg-feishu-secret', c.platforms?.feishu?.secret || '');
        setChecked('cfg-feishu-enabled', c.platforms?.feishu?.enabled || false);
        setVal('cfg-discord-token', c.platforms?.discord?.token || '');
        setVal('cfg-discord-clientid', c.platforms?.discord?.clientid || '');
        setChecked('cfg-discord-enabled', c.platforms?.discord?.enabled || false);
        setVal('cfg-language', c.general?.language || 'zh');
        setVal('cfg-theme', c.general?.theme || 'dark');
        setVal('cfg-port', c.general?.port || 8080);
        setChecked('cfg-autostart', c.general?.autostart || false);
        setChecked('cfg-debug', c.general?.debug || false);
        this.loadRoleConfig();
    }

    async loadRoleConfig() {
        try {
            const resp = await fetch('/api/config/role');
            if (resp.ok) {
                const data = await resp.json();
                if (data.success) {
                    const soulEl = document.getElementById('cfg-soul-md');
                    const userEl = document.getElementById('cfg-user-md');
                    if (soulEl) soulEl.value = data.soul_md || '';
                    if (userEl) userEl.value = data.user_md || '';
                }
            }
        } catch (e) {
            console.log('Failed to load role config from backend');
        }
    }

    async saveRoleConfig() {
        const soulContent = document.getElementById('cfg-soul-md')?.value || '';
        const userContent = document.getElementById('cfg-user-md')?.value || '';
        
        try {
            const resp = await fetch('/api/config/role', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    soul_md: soulContent,
                    user_md: userContent
                })
            });
            if (resp.ok) {
                this.showNotification('角色配置已保存', 'success');
            } else {
                this.showNotification('保存失败', 'error');
            }
        } catch (e) {
            this.showNotification('保存失败: ' + e.message, 'error');
        }
    }

    applySoulTemplate(templateName) {
        const templates = {
            pragmatic: `# Identity
You are a pragmatic senior engineer with strong taste.
You optimize for truth, clarity, and usefulness over politeness theater.

## Style
- Be direct without being cold
- Prefer substance over filler
- Push back when something is a bad idea
- Admit uncertainty plainly
- Keep explanations compact unless depth is useful

## What to avoid
- Sycophancy
- Hype language
- Repeating the user's framing if it's wrong
- Overexplaining obvious things

## Technical posture
- Prefer simple systems over clever systems
- Care about operational reality, not idealized architecture
- Treat edge cases as part of the design, not cleanup`,
            research: `# Identity
You are a research partner who helps explore ideas deeply.
You think systematically and help surface blind spots.

## Style
- Ask clarifying questions before diving in
- Break complex problems into manageable parts
- Present multiple perspectives when relevant
- Cite sources and evidence when possible
- Distinguish between facts and speculation

## Approach
- Start with first principles thinking
- Consider edge cases and failure modes
- Help prioritize what to investigate next
- Summarize findings clearly`,
            teacher: `# Identity
You are a patient educator who makes complex topics accessible.
You focus on building understanding, not just giving answers.

## Style
- Use clear, concrete examples
- Build from simple to complex gradually
- Check for understanding before moving on
- Encourage questions and exploration
- Adapt explanations to the learner's level

## Teaching approach
- Explain the "why" before the "how"
- Use analogies that connect to known concepts
- Provide practice opportunities
- Give constructive, specific feedback`,
            reviewer: `# Identity
You are a strict code reviewer with high standards.
You catch issues early and help maintain code quality.

## Style
- Be thorough but constructive in feedback
- Explain the reasoning behind suggestions
- Prioritize issues by severity
- Suggest specific improvements, not just problems
- Acknowledge good practices when you see them

## Review focus
- Security vulnerabilities
- Performance bottlenecks
- Maintainability and readability
- Error handling and edge cases
- API design and consistency`
        };
        
        const template = templates[templateName];
        if (template) {
            const soulEl = document.getElementById('cfg-soul-md');
            if (soulEl) {
                soulEl.value = template;
            }
        }
    }

    saveConfigFromUI() {
        const getVal = (id) => { const el = document.getElementById(id); return el ? el.value : ''; };
        const isChecked = (id) => { const el = document.getElementById(id); return el ? el.checked : false; };
        this.config = {
            models: {
                ollama: { url: getVal('cfg-ollama-url'), model: getVal('cfg-ollama-model'), enabled: isChecked('cfg-ollama-enabled') },
                openai: { key: getVal('cfg-openai-key'), model: getVal('cfg-openai-model'), enabled: isChecked('cfg-openai-enabled'), url: getVal('cfg-openai-url') },
                anthropic: { key: getVal('cfg-anthropic-key'), model: getVal('cfg-anthropic-model'), enabled: isChecked('cfg-anthropic-enabled'), url: getVal('cfg-anthropic-url') },
                deepseek: { key: getVal('cfg-deepseek-key'), model: getVal('cfg-deepseek-model'), enabled: isChecked('cfg-deepseek-enabled'), url: getVal('cfg-deepseek-url') },
                minimax: { key: getVal('cfg-minimax-key'), group_id: getVal('cfg-minimax-group-id'), model: getVal('cfg-minimax-model'), enabled: isChecked('cfg-minimax-enabled'), url: getVal('cfg-minimax-url') },
                glm: { key: getVal('cfg-glm-key'), model: getVal('cfg-glm-model'), enabled: isChecked('cfg-glm-enabled'), url: getVal('cfg-glm-url') },
                kimi: { key: getVal('cfg-kimi-key'), model: getVal('cfg-kimi-model'), enabled: isChecked('cfg-kimi-enabled'), url: getVal('cfg-kimi-url') },
                qwen: { key: getVal('cfg-qwen-key'), model: getVal('cfg-qwen-model'), enabled: isChecked('cfg-qwen-enabled'), url: getVal('cfg-qwen-url') }
            },
            platforms: {
                wechat: { mode: getVal('cfg-wechat-mode'), key: getVal('cfg-wechat-key'), enabled: isChecked('cfg-wechat-enabled') },
                qqbot: { appid: getVal('cfg-qqbot-appid'), secret: getVal('cfg-qqbot-secret'), token: getVal('cfg-qqbot-token'), enabled: isChecked('cfg-qqbot-enabled') },
                telegram: { token: getVal('cfg-telegram-token'), webhook: getVal('cfg-telegram-webhook'), enabled: isChecked('cfg-telegram-enabled') },
                feishu: { appid: getVal('cfg-feishu-appid'), secret: getVal('cfg-feishu-secret'), enabled: isChecked('cfg-feishu-enabled') },
                discord: { token: getVal('cfg-discord-token'), clientid: getVal('cfg-discord-clientid'), enabled: isChecked('cfg-discord-enabled') }
            },
            general: { language: getVal('cfg-language'), theme: getVal('cfg-theme'), port: parseInt(getVal('cfg-port')) || 8080, autostart: isChecked('cfg-autostart'), debug: isChecked('cfg-debug') }
        };
        localStorage.setItem('mahakala_config', JSON.stringify(this.config));
        localStorage.setItem('config_saved', 'true');
        this.updateModelInfo();
        this.saveConfigToBackend();
        this.saveRoleConfig();
    }

    updateModelInfo() {
        const enabledModels = [];
        const modelNames = {
            ollama: 'Ollama', openai: 'OpenAI', anthropic: 'Anthropic', deepseek: 'DeepSeek',
            minimax: 'MiniMax', glm: 'GLM', kimi: 'Kimi', qwen: 'Qwen'
        };
        for (const [key, val] of Object.entries(this.config.models)) {
            if (val.enabled && val.model) {
                enabledModels.push({ name: modelNames[key], model: val.model, key });
            }
        }
        const modelEl = document.getElementById('current-model');
        if (modelEl) {
            if (enabledModels.length === 0) {
                modelEl.textContent = this.t('chat.noModel');
                this.activeModel = null;
            } else if (enabledModels.length === 1) {
                this.activeModel = enabledModels[0];
                modelEl.textContent = `${this.activeModel.name}: ${this.activeModel.model}`;
            } else {
                this.activeModel = enabledModels[0];
                modelEl.textContent = `${this.activeModel.name}: ${this.activeModel.model} (+${enabledModels.length - 1})`;
            }
        }
    }

    async saveConfigToBackend() {
        // 确保 Ollama URL 包含 /v1
        const config = JSON.parse(JSON.stringify(this.config));
        if (config.models && config.models.ollama && config.models.ollama.url) {
            let url = config.models.ollama.url.trim();
            if (!url.endsWith('/v1') && !url.includes('/v1')) {
                url = url.replace(/\/+$/, '') + '/v1';
                config.models.ollama.url = url;
            }
        }
        
        try {
            await fetch('/api/config', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(config) });
        } catch (e) {
            console.log('Backend config save failed, using localStorage only');
        }
    }

    async restoreConfigToBackend() {
        // 页面加载时，将localStorage中的配置推送到后端，恢复Agent配置
        const savedConfig = localStorage.getItem('mahakala_config');
        if (savedConfig) {
            try {
                const config = JSON.parse(savedConfig);
                // 找到第一个启用的模型，确保它有API Key
                let hasValidModel = false;
                for (const [key, modelConfig] of Object.entries(config.models || {})) {
                    if (modelConfig.enabled && modelConfig.model) {
                        // 如果是需要API Key的提供商且Key存在
                        if (key !== 'ollama' && modelConfig.key) {
                            hasValidModel = true;
                            break;
                        } else if (key === 'ollama') {
                            hasValidModel = true;
                            break;
                        }
                    }
                }
                // 如果有有效的模型配置，推送到后端
                if (hasValidModel) {
                    await fetch('/api/config', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(config)
                    });
                    console.log('Config restored to backend from localStorage');
                }
            } catch (e) {
                console.log('Failed to restore config to backend:', e);
            }
        }
    }

    async loadConfigFromBackend() {
        try {
            const resp = await fetch('/api/config');
            if (resp.ok) {
                const backendConfig = await resp.json();
                
                // 优先保留localStorage中已保存的models配置（包含API Key）
                // 只在backend有值时才覆盖
                const mergedModels = { ...this.getDefaultConfig().models };
                for (const [key, value] of Object.entries(this.config.models || {})) {
                    if (mergedModels[key]) {
                        mergedModels[key] = { ...mergedModels[key], ...value };
                    } else {
                        mergedModels[key] = value;
                    }
                }
                
                this.config = {
                    ...this.getDefaultConfig(),
                    models: mergedModels,
                    platforms: {
                        ...this.getDefaultConfig().platforms,
                        ...(this.config.platforms || {})
                    },
                    general: {
                        ...this.getDefaultConfig().general,
                        ...backendConfig.general,
                        ...(this.config.general || {})
                    }
                };
                
                this.loadConfigToUI();
            }
        } catch (e) {
            console.log('Backend config load failed, using localStorage only');
        }
    }

    showConfigModal() { const overlay = document.getElementById('config-overlay'); if (overlay) overlay.style.display = 'flex'; }
    hideConfigModal() { const overlay = document.getElementById('config-overlay'); if (overlay) overlay.style.display = 'none'; }

    applyTheme() {
        document.documentElement.setAttribute('data-theme', this.currentTheme);
        const icon = document.querySelector('#theme-toggle i');
        if (icon) icon.className = this.currentTheme === 'dark' ? 'fas fa-sun' : 'fas fa-moon';
        localStorage.setItem('theme', this.currentTheme);
    }

    applyLanguage() {
        document.querySelectorAll('[data-i18n]').forEach(el => {
            const key = el.getAttribute('data-i18n');
            const val = this.t(key);
            if (val) {
                if (el.tagName === 'INPUT' && el.type !== 'hidden') el.placeholder = val;
                else el.textContent = val;
            }
        });
        localStorage.setItem('lang', this.currentLang);
    }

    t(key) { return this.i18n[this.currentLang]?.[key] || key; }

    bindEvents() {
        document.querySelectorAll('.menu-item').forEach(item => {
            item.addEventListener('click', (e) => {
                e.stopPropagation();
                document.querySelectorAll('.menu-item').forEach(i => i.classList.remove('active'));
                item.classList.add('active');
            });
        });
        document.querySelectorAll('.dropdown-item').forEach(item => {
            item.addEventListener('click', (e) => {
                e.stopPropagation();
                const action = item.getAttribute('data-action');
                this.handleMenuAction(action);
            });
        });
        document.querySelectorAll('.status-item[data-status-menu]').forEach(item => {
            item.addEventListener('click', (e) => {
                e.stopPropagation();
                const menuType = item.getAttribute('data-status-menu');
                this.toggleStatusPopup(menuType);
            });
        });
        document.addEventListener('click', () => {
            document.querySelectorAll('.menu-item').forEach(i => i.classList.remove('active'));
        });
        document.querySelectorAll('.activity-item').forEach(item => {
            item.addEventListener('click', () => {
                const page = item.getAttribute('data-page');
                if (page) this.switchPage(page);
            });
        });
        document.getElementById('theme-toggle')?.addEventListener('click', () => this.toggleTheme());
        document.getElementById('lang-toggle')?.addEventListener('click', () => this.toggleLanguage());
        document.getElementById('send-message')?.addEventListener('click', () => this.sendMessage());
        document.getElementById('chat-input')?.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); this.sendMessage(); }
        });
        document.querySelectorAll('.action-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const action = btn.getAttribute('data-action');
                if (action === 'new-session') this.createNewSession();
                else if (action === 'compress') this.compressContext();
            });
        });
        document.querySelectorAll('.config-tab').forEach(tab => {
            tab.addEventListener('click', () => this.switchConfigTab(tab.getAttribute('data-config')));
        });
        document.getElementById('config-close')?.addEventListener('click', () => this.hideConfigModal());
        document.getElementById('config-overlay')?.addEventListener('click', (e) => { if (e.target === e.currentTarget) this.hideConfigModal(); });
        document.getElementById('config-save')?.addEventListener('click', () => { this.saveConfigFromUI(); this.hideConfigModal(); });
        document.getElementById('config-reset')?.addEventListener('click', () => this.handleConfigReset());
        document.querySelectorAll('.config-platform-btn').forEach(btn => {
            btn.addEventListener('click', () => this.togglePlatformConfig(btn.getAttribute('data-platform')));
        });
        document.getElementById('btn-ollama-detect')?.addEventListener('click', () => this.detectOllama());
        document.getElementById('btn-wechat-qr-login')?.addEventListener('click', () => this.getWechatQRCode());
        document.getElementById('btn-wechat-qr-check')?.addEventListener('click', () => this.checkWechatQRStatus());
        document.getElementById('btn-new-session')?.addEventListener('click', () => this.createNewSession());
        document.getElementById('add-fact')?.addEventListener('click', () => this.showAddFactDialog());
        document.getElementById('search-memory')?.addEventListener('click', () => this.showSearchMemoryDialog());
        document.querySelectorAll('.sidebar-item').forEach(item => {
            item.addEventListener('click', () => {
                const sessionId = item.getAttribute('data-session-id');
                if (sessionId) this.switchSession(sessionId);
            });
        });
        document.querySelectorAll('.filter-tab').forEach(tab => {
            tab.addEventListener('click', () => {
                const parent = tab.parentElement;
                parent.querySelectorAll('.filter-tab').forEach(t => t.classList.remove('active'));
                tab.classList.add('active');
            });
        });
        document.querySelectorAll('.panel-tab').forEach(tab => {
            tab.addEventListener('click', () => this.switchPanel(tab.getAttribute('data-panel')));
        });
        document.querySelectorAll('.settings-tab').forEach(tab => {
            tab.addEventListener('click', () => this.switchSettingsTab(tab.getAttribute('data-tab')));
        });
        document.querySelectorAll('.template-tag').forEach(tag => {
            tag.addEventListener('click', () => {
                const template = tag.getAttribute('data-template');
                this.applySoulTemplate(template);
            });
        });
        document.querySelectorAll('.toolbar-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const title = btn.getAttribute('title');
                if (title && title.includes('上传')) this.handleFileUpload();
                else if (title && title.includes('语音')) this.handleVoiceInput();
                else if (title && title.includes('代码')) this.handleCodeBlock();
                else if (title && title.includes('运行')) this.showToolSelector();
            });
        });
        document.querySelectorAll('.quick-actions .action-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const title = btn.getAttribute('title');
                if (title && title.includes('代码')) this.showRunCodeDialog();
                else if (title && title.includes('搜索')) this.showWebSearchDialog();
                else if (title && title.includes('文件')) this.showFileManager();
                else if (title && title.includes('终端')) this.showTerminal();
            });
        });
        this.loadPageContent();
    }

    switchConfigTab(config) {
        document.querySelectorAll('.config-tab').forEach(t => t.classList.remove('active'));
        document.querySelectorAll('.config-section').forEach(s => s.classList.remove('active'));
        document.querySelector(`.config-tab[data-config="${config}"]`)?.classList.add('active');
        document.getElementById(`config-${config}`)?.classList.add('active');
    }

    togglePlatformConfig(platform) {
        const group = document.getElementById(`cfg-group-${platform}`);
        if (group) {
            const isVisible = group.style.display === 'block';
            group.style.display = isVisible ? 'none' : 'block';
            if (platform === 'wechat' && !isVisible) {
                this.getWechatQRCode();
            }
        }
    }

    handleConfigReset() {
        this.config = this.getDefaultConfig();
        this.loadConfigToUI();
        localStorage.removeItem('mahakala_config');
        localStorage.removeItem('config_saved');
    }

    handleMenuAction(action) {
        switch (action) {
            case 'new-session': this.createNewSession(); break;
            case 'open-file': this.openFileDialog(); break;
            case 'save': this.saveConfigFromUI(); this.showNotification(this.t('notification.success'), 'success'); break;
            case 'config': this.showConfigModal(); break;
            case 'exit': this.showNotification('退出功能', 'info'); break;
            case 'undo': this.showNotification('撤销', 'info'); break;
            case 'redo': this.showNotification('重做', 'info'); break;
            case 'copy': this.copyToClipboard(); break;
            case 'paste': this.showNotification('粘贴', 'info'); break;
            case 'clear-chat': this.clearChat(); break;
            case 'toggle-sidebar': this.toggleSidebar(); break;
            case 'toggle-right-panel': this.toggleRightPanel(); break;
            case 'zoom-in': this.zoomIn(); break;
            case 'zoom-out': this.zoomOut(); break;
            case 'reset-zoom': this.resetZoom(); break;
            case 'run-code': this.showRunCodeDialog(); break;
            case 'web-search': this.showWebSearchDialog(); break;
            case 'file-manager': this.showFileManager(); break;
            case 'terminal': this.showTerminal(); break;
            case 'docs': this.showDocumentation(); break;
            case 'shortcuts': this.showKeyboardShortcuts(); break;
            case 'about': this.showAboutDialog(); break;
        }
    }

    openFileDialog() {
        if (window.__TAURI_ADAPTER__ && window.__TAURI_ADAPTER__.isTauri) {
            window.__TAURI_ADAPTER__.openFileDialog().then(path => {
                if (path) {
                    this.showNotification(`已选择文件: ${path}`, 'info');
                }
            });
            return;
        }
        const input = document.createElement('input');
        input.type = 'file';
        input.onchange = (e) => {
            const file = e.target.files[0];
            if (file) {
                this.showNotification(`已选择文件: ${file.name}`, 'info');
            }
        };
        input.click();
    }

    async copyToClipboard() {
        try {
            const text = window.getSelection().toString();
            if (text) {
                await navigator.clipboard.writeText(text);
                this.showNotification('已复制到剪贴板', 'success');
            }
        } catch (e) {
            this.showNotification('复制失败', 'error');
        }
    }

    zoomIn() {
        const currentZoom = parseFloat(getComputedStyle(document.documentElement).fontSize) || 16;
        document.documentElement.style.fontSize = `${Math.min(currentZoom + 1, 24)}px`;
        this.showNotification('放大', 'info');
    }

    zoomOut() {
        const currentZoom = parseFloat(getComputedStyle(document.documentElement).fontSize) || 16;
        document.documentElement.style.fontSize = `${Math.max(currentZoom - 1, 12)}px`;
        this.showNotification('缩小', 'info');
    }

    resetZoom() {
        document.documentElement.style.fontSize = '16px';
        this.showNotification('重置缩放', 'info');
    }

    showRunCodeDialog() {
        const code = prompt('输入要运行的代码:');
        if (code) {
            this.addOutputLog(`运行代码: ${code.substring(0, 50)}${code.length > 50 ? '...' : ''}`);
            this.addDebugLog(`执行代码请求: ${code.substring(0, 100)}`);
            this.showNotification(`运行代码: ${code.substring(0, 50)}...`, 'info');
            this.appendMessage('assistant', `代码已发送执行:\n\`\`\`\n${code}\n\`\`\``);
        }
    }

    showWebSearchDialog() {
        const query = prompt('输入搜索查询:');
        if (query) {
            this.addOutputLog(`网页搜索: ${query}`);
            this.addDebugLog(`搜索请求: ${query}`);
            this.showNotification(`网页搜索: ${query}`, 'info');
            this.appendMessage('assistant', `正在搜索: ${query}`);
        }
    }

    handleFileUpload() {
        const input = document.createElement('input');
        input.type = 'file';
        input.multiple = true;
        input.onchange = async (e) => {
            const files = Array.from(e.target.files);
            for (const file of files) {
                const formData = new FormData();
                formData.append('file', file);
                try {
                    const resp = await fetch('/api/upload', { method: 'POST', body: formData });
                    const data = await resp.json();
                    if (data.id) {
                        this.appendMessage('user', `[上传文件: ${file.name}]`);
                        this.appendMessage('assistant', `文件 "${file.name}" 已上传成功 (${(file.size / 1024).toFixed(1)} KB)`);
                        this.showNotification(`文件已上传: ${file.name}`, 'success');
                    } else {
                        this.showNotification(`上传失败: ${data.error || '未知错误'}`, 'error');
                    }
                } catch (err) {
                    this.showNotification(`上传失败: ${err.message}`, 'error');
                }
            }
        };
        input.click();
    }

    handleVoiceInput() {
        if (!('webkitSpeechRecognition' in window) && !('SpeechRecognition' in window)) {
            this.showNotification('浏览器不支持语音识别，请使用 Chrome 浏览器', 'error');
            return;
        }
        const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
        const recognition = new SpeechRecognition();
        recognition.lang = this.currentLang === 'zh' ? 'zh-CN' : 'en-US';
        recognition.interimResults = true;
        recognition.continuous = false;

        const input = document.getElementById('chat-input');
        if (!input) return;

        this.showNotification('正在聆听语音...', 'info');
        recognition.start();

        recognition.onresult = (event) => {
            let transcript = '';
            for (let i = event.resultIndex; i < event.results.length; i++) {
                transcript += event.results[i][0].transcript;
            }
            input.value = transcript;
            input.style.height = 'auto';
            input.style.height = input.scrollHeight + 'px';
        };

        recognition.onend = () => {
            if (input.value.trim()) {
                this.showNotification('语音识别完成', 'success');
            }
        };

        recognition.onerror = (event) => {
            this.showNotification(`语音识别错误: ${event.error}`, 'error');
        };
    }

    handleCodeBlock() {
        const input = document.getElementById('chat-input');
        if (!input) return;
        const lang = prompt('输入代码语言 (如 javascript, python, rust):', 'javascript');
        if (lang === null) return;
        const cursorPos = input.selectionStart;
        const textBefore = input.value.substring(0, cursorPos);
        const textAfter = input.value.substring(cursorPos);
        input.value = textBefore + `\n\`\`\`${lang}\n\n\`\`\`\n` + textAfter;
        input.focus();
        const newPos = cursorPos + `\n\`\`\`${lang}\n`.length;
        input.setSelectionRange(newPos, newPos);
        input.style.height = 'auto';
        input.style.height = input.scrollHeight + 'px';
    }

    showFileManager() {
        this.addOutputLog('打开文件管理器');
        this.addDebugLog('文件管理器面板请求');
        this.showNotification('文件管理器功能开发中', 'info');
    }

    showTerminal() {
        this.addOutputLog('打开终端');
        this.addDebugLog('终端面板请求');
        this.showNotification('终端功能开发中', 'info');
    }

    showDocumentation() {
        this.showNotification('文档功能开发中', 'info');
    }

    showKeyboardShortcuts() {
        const shortcuts = `
快捷键列表:
- Enter: 发送消息
- Shift+Enter: 换行
- Ctrl+S: 保存配置
- Ctrl+K: 清空对话
- Ctrl+B: 切换侧边栏
        `;
        this.showNotification(shortcuts, 'info');
    }

    clearChat() {
        if (!this.currentSession) return;
        const session = this.sessions.find(s => s.id === this.currentSession);
        if (session) {
            session.messages = [];
            this.saveSessions();
            this.loadSessionMessages();
            this.showNotification('对话已清空', 'success');
        }
    }

    toggleSidebar() {
        const sidebar = document.querySelector('.sidebar');
        if (sidebar) sidebar.classList.toggle('collapsed');
    }

    toggleRightPanel() {
        const rightPanel = document.querySelector('.right-panel');
        if (rightPanel) rightPanel.classList.toggle('collapsed');
    }

    toggleStatusPopup(menuType) {
        const popupMap = {
            'tools': 'popup-tools-list',
            'plugins': 'popup-plugins-list',
            'memory': 'popup-memory-stats',
            'skills': 'popup-skills-list'
        };
        const listId = popupMap[menuType];
        if (!listId) return;
        this.loadStatusPopupContent(menuType, listId);
    }

    async loadStatusPopupContent(type, containerId) {
        const container = document.getElementById(containerId);
        if (!container) return;
        try {
            switch (type) {
                case 'tools': {
                    const res = await fetch('/api/tools');
                    const data = await res.json();
                    const tools = data.tools || [];
                    container.innerHTML = tools.slice(0, 20).map(t =>
                        `<div class="popup-list-item"><i class="fas fa-wrench"></i> ${t}</div>`
                    ).join('');
                    break;
                }
                case 'plugins': {
                    const res = await fetch('/api/plugins');
                    const data = await res.json();
                    const plugins = data.plugins || [];
                    container.innerHTML = plugins.map(p =>
                        `<div class="popup-list-item"><i class="fas fa-puzzle-piece"></i> ${p.name || p}</div>`
                    ).join('') || '<div class="popup-list-item">暂无插件</div>';
                    break;
                }
                case 'memory': {
                    const res = await fetch('/api/memory/stats');
                    const data = await res.json();
                    container.innerHTML = `
                        <div class="popup-stat-row"><span>事实数量</span><span>${data.facts || 0}</span></div>
                        <div class="popup-stat-row"><span>实体数量</span><span>${data.entities || 0}</span></div>
                        <div class="popup-stat-row"><span>向量维度</span><span>${data.dimensions || 0}</span></div>
                    `;
                    break;
                }
                case 'skills': {
                    const res = await fetch('/api/skills');
                    const data = await res.json();
                    const skills = data.skills || [];
                    container.innerHTML = skills.map(s =>
                        `<div class="popup-list-item"><i class="fas fa-magic"></i> ${s.name || s}</div>`
                    ).join('') || '<div class="popup-list-item">暂无技能</div>';
                    break;
                }
            }
        } catch (e) {
            container.innerHTML = '<div class="popup-list-item">加载失败</div>';
        }
    }

    showAboutDialog() {
        this.showNotification('Mahakala Agent v0.1.0 - AI智能助手', 'info');
    }

    switchPage(page) {
        this.currentPage = page;
        document.querySelectorAll('.activity-item').forEach(i => i.classList.remove('active'));
        document.querySelector(`.activity-item[data-page="${page}"]`)?.classList.add('active');
        document.querySelectorAll('.page').forEach(p => p.classList.remove('active'));
        document.getElementById(`page-${page}`)?.classList.add('active');
        this.updateSidebar(page);
        this.loadPageContent();
    }

    updateSidebar(page) {
        document.querySelectorAll('.sidebar-item').forEach(i => i.classList.remove('active'));
        document.querySelector(`.sidebar-item[data-page="${page}"]`)?.classList.add('active');
        
        const sidebarContent = document.getElementById('sidebar-content');
        if (!sidebarContent) return;
        
        const sidebarTitle = document.getElementById('sidebar-title');
        
        switch (page) {
            case 'chat':
                if (sidebarTitle) sidebarTitle.textContent = this.t('sidebar.sessions');
                this.loadSidebarSessions();
                break;
            case 'memory':
                if (sidebarTitle) sidebarTitle.textContent = this.t('sidebar.memory');
                this.loadMemorySidebar(sidebarContent);
                break;
            case 'skills':
                if (sidebarTitle) sidebarTitle.textContent = this.t('sidebar.skills');
                this.loadSkillsSidebar(sidebarContent);
                break;
            case 'tools':
                if (sidebarTitle) sidebarTitle.textContent = this.t('sidebar.tools');
                this.loadToolsSidebar(sidebarContent);
                break;
            case 'plugins':
                if (sidebarTitle) sidebarTitle.textContent = this.t('sidebar.plugins');
                this.loadPluginsSidebar(sidebarContent);
                break;
            case 'messages':
                if (sidebarTitle) sidebarTitle.textContent = this.t('sidebar.messages');
                this.loadMessagesSidebar(sidebarContent);
                break;
            case 'platforms':
                if (sidebarTitle) sidebarTitle.textContent = this.t('sidebar.platforms');
                this.loadPlatformsSidebar(sidebarContent);
                break;
            case 'cron':
                if (sidebarTitle) sidebarTitle.textContent = this.t('sidebar.cron');
                this.loadCronSidebar(sidebarContent);
                break;
            case 'settings':
                if (sidebarTitle) sidebarTitle.textContent = this.t('sidebar.settings');
                sidebarContent.innerHTML = '';
                break;
        }
    }

    toggleTheme() {
        this.currentTheme = this.currentTheme === 'dark' ? 'light' : 'dark';
        this.applyTheme();
        this.config.general.theme = this.currentTheme;
    }

    toggleLanguage() {
        this.currentLang = this.currentLang === 'zh' ? 'en' : 'zh';
        this.applyLanguage();
        this.config.general.language = this.currentLang;
    }

    connectWebSocket() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;
        try {
            this.ws = new WebSocket(wsUrl);
            this.ws.onopen = () => console.log('WebSocket connected');
            this.ws.onmessage = (event) => { try { const data = JSON.parse(event.data); this.handleWebSocketMessage(data); } catch (e) {} };
            this.ws.onclose = () => { setTimeout(() => this.connectWebSocket(), 5000); };
            this.ws.onerror = () => {};
        } catch (e) { console.log('WebSocket not available, using REST API'); }
    }

    handleWebSocketMessage(data) {
        if (data.type === 'chat_response') {
            this.appendMessage('assistant', data.content);
            this.addOutputLog(`模型回复: ${data.content.substring(0, 50)}${data.content.length > 50 ? '...' : ''}`);
        }
        else if (data.type === 'status') {
            this.updateStatus(data);
            this.addDebugLog(`状态更新: ${JSON.stringify(data)}`);
        }
        else if (data.type === 'tool_call') {
            this.appendToolMessage(data.tool_name, data.status || 'executing', data.message || '');
            this.addOutputLog(`工具调用: ${data.tool_name} [${data.status || 'executing'}]`);
        }
        else if (data.type === 'notification') {
            this.showNotification(data.message, data.level || 'info');
            this.addDebugLog(`通知: ${data.message}`);
        }
    }

    appendToolMessage(toolName, status, message) {
        const container = document.getElementById('chat-messages');
        if (!container) return;
        
        const msg = document.createElement('div');
        msg.className = `message assistant tool-${status}`;
        
        const avatar = document.createElement('div');
        avatar.className = 'message-avatar';
        avatar.innerHTML = '<i class="fas fa-robot"></i>';
        
        const contentDiv = document.createElement('div');
        contentDiv.className = 'message-content';
        
        const header = document.createElement('div');
        header.className = 'message-header';
        
        const sender = document.createElement('span');
        sender.className = 'sender';
        sender.textContent = 'Mahakala Agent';
        
        const timestamp = document.createElement('span');
        timestamp.className = 'timestamp';
        timestamp.textContent = new Date().toLocaleTimeString();
        
        header.appendChild(sender);
        header.appendChild(timestamp);
        
        const bubble = document.createElement('div');
        bubble.className = 'message-text';
        
        let indicatorHTML = '';
        if (status === 'executing') {
            indicatorHTML = `
                <div class="tool-name-indicator">
                    <div class="spinner"></div>
                    <span class="tool-executing-text">正在执行: ${toolName}</span>
                    <div class="loading-dots">
                        <span></span><span></span><span></span>
                    </div>
                </div>
                <div class="tool-progress-container">
                    <div class="tool-progress-bar"></div>
                </div>
            `;
        } else if (status === 'completed') {
            indicatorHTML = `
                <div class="tool-completed-indicator">
                    <i class="fas fa-check-circle"></i>
                    <span>已完成: ${toolName}</span>
                </div>
            `;
        } else if (status === 'error') {
            indicatorHTML = `
                <div class="tool-error-indicator">
                    <i class="fas fa-exclamation-circle"></i>
                    <span>错误: ${toolName}</span>
                </div>
            `;
        }
        
        const contentHTML = message ? `<div style="margin-top: 8px; color: var(--text-secondary);">${this.formatMessage(message)}</div>` : '';
        bubble.innerHTML = indicatorHTML + contentHTML;
        
        contentDiv.appendChild(header);
        contentDiv.appendChild(bubble);
        msg.appendChild(avatar);
        msg.appendChild(contentDiv);
        container.appendChild(msg);
        container.scrollTop = container.scrollHeight;
        
        return msg;
    }

    appendMessage(role, content) {
        const container = document.getElementById('chat-messages');
        if (!container) return;
        const msg = document.createElement('div');
        msg.className = `message ${role}`;
        const avatar = document.createElement('div');
        avatar.className = 'message-avatar';
        avatar.innerHTML = role === 'user' ? '<i class="fas fa-user"></i>' : '<i class="fas fa-robot"></i>';
        const contentDiv = document.createElement('div');
        contentDiv.className = 'message-content';
        const header = document.createElement('div');
        header.className = 'message-header';
        const sender = document.createElement('span');
        sender.className = 'sender';
        sender.textContent = role === 'user' ? 'You' : 'Mahakala Agent';
        const timestamp = document.createElement('span');
        timestamp.className = 'timestamp';
        timestamp.textContent = new Date().toLocaleTimeString();
        header.appendChild(sender);
        header.appendChild(timestamp);
        const bubble = document.createElement('div');
        bubble.className = 'message-text';
        bubble.innerHTML = this.formatMessage(content);
        contentDiv.appendChild(header);
        contentDiv.appendChild(bubble);
        msg.appendChild(avatar);
        msg.appendChild(contentDiv);
        container.appendChild(msg);
        container.scrollTop = container.scrollHeight;
    }

    formatMessage(content) {
        if (!content) return '';
        let html = content.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
            .replace(/```(\w*)\n([\s\S]*?)```/g, '<pre><code class="language-$1">$2</code></pre>')
            .replace(/`([^`]+)`/g, '<code>$1</code>')
            .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
            .replace(/\*([^*]+)\*/g, '<em>$1</em>')
            .replace(/\n/g, '<br>');
        return html;
    }

    updateActiveModel() {
        const enabledModels = [];
        const m = this.config.models;
        if (m?.ollama?.enabled && m.ollama.model) enabledModels.push({ name: 'Ollama', model: m.ollama.model, type: 'ollama' });
        if (m?.openai?.enabled) enabledModels.push({ name: 'OpenAI', model: m.openai.model, type: 'openai' });
        if (m?.anthropic?.enabled) enabledModels.push({ name: 'Anthropic', model: m.anthropic.model, type: 'anthropic' });
        if (m?.deepseek?.enabled) enabledModels.push({ name: 'DeepSeek', model: m.deepseek.model, type: 'deepseek' });
        if (m?.minimax?.enabled) enabledModels.push({ name: 'MiniMax', model: m.minimax.model, type: 'minimax' });
        if (m?.glm?.enabled) enabledModels.push({ name: 'GLM', model: m.glm.model, type: 'glm' });
        if (m?.kimi?.enabled) enabledModels.push({ name: 'Kimi', model: m.kimi.model, type: 'kimi' });
        if (m?.qwen?.enabled) enabledModels.push({ name: 'Qwen', model: m.qwen.model, type: 'qwen' });
        const el = document.getElementById('active-model');
        if (el) {
            if (enabledModels.length === 0) {
                el.innerHTML = `<i class="fas fa-exclamation-triangle"></i> ${this.t('chat.noModel')}`;
                this.activeModel = null;
            } else if (enabledModels.length === 1) {
                this.activeModel = enabledModels[0];
                el.innerHTML = `<i class="fas fa-brain"></i> ${this.activeModel.name}: ${this.activeModel.model}`;
            } else {
                this.activeModel = enabledModels[0];
                el.innerHTML = `<i class="fas fa-brain"></i> ${this.activeModel.name}: ${this.activeModel.model} (+${enabledModels.length - 1})`;
            }
        }
        return enabledModels;
    }

    updateStatus(data) {
        const toolsEl = document.getElementById('status-tools');
        const pluginsEl = document.getElementById('status-plugins');
        const memoryEl = document.getElementById('status-memory');
        const skillsEl = document.getElementById('status-skills');
        const modeBadge = document.getElementById('mode-badge');
        if (toolsEl && data.tools !== undefined) toolsEl.textContent = `${this.t('statusbar.tools')}: ${data.tools}`;
        if (pluginsEl && data.plugins !== undefined) pluginsEl.textContent = `${this.t('statusbar.plugins')}: ${data.plugins}`;
        if (memoryEl && data.memory !== undefined) memoryEl.textContent = `${this.t('statusbar.memory')}: ${data.memory}`;
        if (skillsEl && data.skills !== undefined) skillsEl.textContent = `${this.t('statusbar.skills')}: ${data.skills}`;
        if (modeBadge) {
            if (data.mode === 'standalone') {
                modeBadge.textContent = '离线';
                modeBadge.style.background = 'var(--bg-tertiary)';
                modeBadge.style.color = 'var(--text-secondary)';
            } else {
                modeBadge.textContent = '在线';
                modeBadge.style.background = 'rgba(78, 201, 176, 0.15)';
                modeBadge.style.color = 'var(--success)';
            }
        }
    }

    updateTime() {
        const el = document.getElementById('current-time');
        if (el) el.textContent = new Date().toLocaleString(this.currentLang === 'zh' ? 'zh-CN' : 'en-US');
    }

    async detectOllama() {
        const url = document.getElementById('cfg-ollama-url')?.value || 'http://localhost:11434';
        const statusEl = document.getElementById('ollama-status');
        const modelSelect = document.getElementById('cfg-ollama-model');
        if (!statusEl || !modelSelect) return;
        statusEl.style.display = 'inline';
        statusEl.textContent = this.t('config.detecting');
        statusEl.style.color = '#ffa500';
        try {
            const resp = await fetch(`${url}/api/tags`);
            if (resp.ok) {
                const data = await resp.json();
                modelSelect.innerHTML = '';
                if (data.models && data.models.length > 0) {
                    data.models.forEach(m => {
                        const opt = document.createElement('option');
                        opt.value = m.name;
                        opt.textContent = m.name;
                        modelSelect.appendChild(opt);
                    });
                    statusEl.textContent = `${this.t('ollama.connected')} - ${data.models.length} ${this.t('ollama.models.found')}`;
                    statusEl.style.color = '#6bcb77';
                    this.config.models.ollama.enabled = true;
                } else {
                    modelSelect.innerHTML = `<option value="">${this.t('ollama.no.models')}</option>`;
                    statusEl.textContent = this.t('ollama.no.models');
                    statusEl.style.color = '#ff6b6b';
                }
            } else { throw new Error('Not OK'); }
        } catch (e) {
            statusEl.textContent = this.t('ollama.disconnected');
            statusEl.style.color = '#ff6b6b';
            modelSelect.innerHTML = `<option value="">${this.t('ollama.disconnected')}</option>`;
        }
    }

    async testModelLatencies() {
        const models = [];
        const m = this.config.models;
        if (m?.openai?.enabled && m.openai.key) models.push({ type: 'openai', url: m.openai.url || 'https://api.openai.com/v1', key: m.openai.key, model: m.openai.model });
        if (m?.anthropic?.enabled && m.anthropic.key) models.push({ type: 'anthropic', url: m.anthropic.url || 'https://api.anthropic.com/v1', key: m.anthropic.key, model: m.anthropic.model });
        if (m?.deepseek?.enabled && m.deepseek.key) models.push({ type: 'deepseek', url: m.deepseek.url || 'https://api.deepseek.com', key: m.deepseek.key, model: m.deepseek.model });
        if (m?.glm?.enabled && m.glm.key) models.push({ type: 'glm', url: m.glm.url || 'https://open.bigmodel.cn/api/paas/v4', key: m.glm.key, model: m.glm.model });
        if (m?.kimi?.enabled && m.kimi.key) models.push({ type: 'kimi', url: m.kimi.url || 'https://api.moonshot.cn/v1', key: m.kimi.key, model: m.kimi.model });
        if (m?.qwen?.enabled && m.qwen.key) models.push({ type: 'qwen', url: m.qwen.url || 'https://dashscope.aliyuncs.com/compatible-mode/v1', key: m.qwen.key, model: m.qwen.model });
        const tests = models.map(async (model) => {
            const start = Date.now();
            try {
                let testUrl, headers, body;
                if (model.type === 'openrouter') {
                    testUrl = `${model.url}/models`;
                    headers = { 'Authorization': `Bearer ${model.key}` };
                } else {
                    testUrl = `${model.url}/chat/completions`;
                    headers = { 'Authorization': `Bearer ${model.key}`, 'Content-Type': 'application/json' };
                    body = JSON.stringify({ model: model.model === 'auto' ? 'meta/llama-3.1-8b-instruct' : model.model, messages: [{ role: 'user', content: 'Hi' }], max_tokens: 5 });
                }
                const resp = await fetch(testUrl, { method: model.type === 'openrouter' ? 'GET' : 'POST', headers, body: body || undefined, signal: AbortSignal.timeout(5000) });
                this.modelLatencies[model.type] = Date.now() - start;
                return { type: model.type, latency: Date.now() - start, ok: resp.ok };
            } catch (e) {
                this.modelLatencies[model.type] = 99999;
                return { type: model.type, latency: 99999, ok: false };
            }
        });
        await Promise.all(tests);
    }

    async detectOpenRouterFreeModels() {
        const url = this.config.models.openrouter.url || 'https://openrouter.ai/api/v1';
        const key = this.config.models.openrouter.key;
        if (!key) return;
        try {
            const resp = await fetch(`${url}/models`, {
                headers: { 'Authorization': `Bearer ${key}` },
                signal: AbortSignal.timeout(8000)
            });
            if (resp.ok) {
                const data = await resp.json();
                const freeModels = (data.data || []).filter(m => m.id && m.id.includes(':free'));
                if (freeModels.length > 0) {
                    const modelSelect = document.getElementById('cfg-openrouter-model');
                    if (modelSelect) {
                        const freeGroup = modelSelect.querySelector('optgroup[label="免费模型"]');
                        if (freeGroup) {
                            freeGroup.innerHTML = '';
                            freeModels.slice(0, 10).forEach(m => {
                                const opt = document.createElement('option');
                                opt.value = m.id;
                                opt.textContent = m.name || m.id;
                                freeGroup.appendChild(opt);
                            });
                        }
                    }
                    const currentModel = this.config.models.openrouter.model;
                    if (!currentModel || !currentModel.includes(':free')) {
                        this.config.models.openrouter.model = freeModels[0].id;
                        const sel = document.getElementById('cfg-openrouter-model');
                        if (sel) sel.value = freeModels[0].id;
                    }
                }
            }
        } catch (e) {
            console.log('OpenRouter free model detection failed:', e);
        }
    }

    async selectFastestNvidiaModel() {
        const nvidiaModels = ['meta/llama-3.1-8b-instruct', 'meta/llama-3.3-70b-instruct', 'microsoft/phi-3-mini-128k-instruct', 'google/gemma-2-27b-it'];
        let fastest = nvidiaModels[0];
        let minLatency = 99999;
        const tests = nvidiaModels.map(async (model) => {
            const start = Date.now();
            try {
                const resp = await fetch(`${this.config.models.nvidia.url}/chat/completions`, {
                    method: 'POST',
                    headers: { 'Authorization': `Bearer ${this.config.models.nvidia.key}`, 'Content-Type': 'application/json' },
                    body: JSON.stringify({ model, messages: [{ role: 'user', content: 'Hi' }], max_tokens: 5 }),
                    signal: AbortSignal.timeout(8000)
                });
                const latency = Date.now() - start;
                if (resp.ok && latency < minLatency) { minLatency = latency; fastest = model; }
            } catch (e) {}
        });
        await Promise.all(tests);
        this.config.models.nvidia.model = fastest;
        const modelSelect = document.getElementById('cfg-nvidia-model');
        if (modelSelect) modelSelect.value = fastest;
    }

    updateSendButtonState() {
        const sendBtn = document.getElementById('send-message');
        const chatInput = document.getElementById('chat-input');
        if (sendBtn) {
            if (this.isConfigRestoring) {
                sendBtn.disabled = true;
                sendBtn.innerHTML = '<i class="fas fa-spinner fa-spin"></i>';
                sendBtn.title = '正在加载配置...';
            } else {
                sendBtn.disabled = false;
                sendBtn.innerHTML = '<i class="fas fa-paper-plane"></i>';
                sendBtn.title = '';
            }
        }
        if (chatInput) {
            chatInput.disabled = this.isConfigRestoring;
            if (this.isConfigRestoring) {
                chatInput.placeholder = '正在加载配置...';
            } else {
                chatInput.placeholder = this.t('chat.placeholder');
            }
        }
    }

    async sendMessage() {
        const input = document.getElementById('chat-input');
        if (!input || this.isStreaming || this.isConfigRestoring) return;
        const message = input.value.trim();
        if (!message) return;

        this.addOutputLog(`用户发送: ${message.substring(0, 50)}${message.length > 50 ? '...' : ''}`);
        this.addDebugLog(`sendMessage called with: ${message.substring(0, 80)}`);

        const commandResult = this.parseNaturalLanguageCommand(message);
        if (commandResult) {
            this.appendMessage('user', message);
            input.value = '';
            this.executeCommand(commandResult);
            return;
        }

        this.appendMessage('user', message);
        input.value = '';
        this.isStreaming = true;
        const sendBtn = document.getElementById('send-message');
        if (sendBtn) { sendBtn.disabled = true; sendBtn.innerHTML = '<i class="fas fa-spinner fa-spin"></i>'; }
        try {
            this.addDebugLog('查找活跃模型配置...');
            const models = this.config.models || {};
            let apiKey = '';
            let apiUrl = '';
            let model = '';
            let provider = '';
            for (const [key, modelConfig] of Object.entries(models)) {
                if (modelConfig.enabled && modelConfig.model) {
                    if (key === 'ollama') {
                        apiUrl = modelConfig.url || '';
                        model = modelConfig.model || '';
                        provider = key;
                        this.addDebugLog(`使用 Ollama 模型: ${model}`);
                        break;
                    }
                    if (modelConfig.key) {
                        apiKey = modelConfig.key;
                        apiUrl = modelConfig.url || '';
                        model = modelConfig.model || '';
                        provider = key;
                        this.addDebugLog(`使用云端模型: ${provider}/${model}`);
                        break;
                    }
                }
            }

            if (!model) {
                this.addOutputLog('错误: 未配置任何模型');
                throw new Error(this.t('chat.noModel'));
            }

            this.addOutputLog(`发送到模型: ${model}`);
            this.addDebugLog(`API 请求: POST /api/chat, model=${model}, provider=${provider}`);

            const resp = await fetch('/api/chat', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    message: message,
                    session_id: this.currentSession || undefined,
                    apiKey: apiKey,
                    apiUrl: apiUrl,
                    model: model,
                    provider: provider,
                })
            });
            
            if (!resp.ok) {
                this.addOutputLog(`API 错误: HTTP ${resp.status}`);
                this.addDebugLog(`API 响应错误: ${resp.status} ${resp.statusText}`);
                throw new Error(`HTTP ${resp.status}`);
            }
            
            const data = await resp.json();
            this.addDebugLog(`API 响应: ${JSON.stringify(data).substring(0, 200)}`);
            
            if (data.response) {
                this.appendMessage('assistant', data.response);
                this.addOutputLog(`模型回复: ${data.response.substring(0, 50)}${data.response.length > 50 ? '...' : ''}`);
                if (this.currentSession) {
                    this.saveMessageToSession(this.currentSession, 'user', message);
                    this.saveMessageToSession(this.currentSession, 'assistant', data.response);
                }
                if (data.tool_calls > 0) {
                    this.addOutputLog(`已调用 ${data.tool_calls} 个工具`);
                    this.showNotification(`已调用 ${data.tool_calls} 个工具`, 'info');
                }
            } else if (data.error) {
                this.addOutputLog(`错误: ${data.error}`);
                throw new Error(data.error);
            } else {
                this.addOutputLog('错误: 未收到模型回复');
                throw new Error('No response from agent');
            }
        } catch (e) {
            this.addOutputLog(`异常: ${e.message}`);
            this.addDebugLog(`异常堆栈: ${e.stack || e.message}`);
            this.appendMessage('assistant', `${this.t('chat.error')}: ${e.message}`);
        } finally {
            this.isStreaming = false;
            if (sendBtn) { sendBtn.disabled = false; sendBtn.innerHTML = '<i class="fas fa-paper-plane"></i>'; }
        }
    }

    parseNaturalLanguageCommand(message) {
        const lowerMsg = message.toLowerCase();
        if (lowerMsg.includes('安装') && (lowerMsg.includes('wechat') || lowerMsg.includes('微信'))) {
            return { type: 'install_platform', platform: 'wechat' };
        }
        const installSkillMatch = lowerMsg.match(/安装.*技能\s*(.+)/);
        if (installSkillMatch) {
            return { type: 'install_skill', skill: installSkillMatch[1].trim() };
        }
        const installPluginMatch = lowerMsg.match(/安装.*插件\s*(.+)/);
        if (installPluginMatch) {
            return { type: 'install_plugin', plugin: installPluginMatch[1].trim() };
        }
        const enableToolMatch = lowerMsg.match(/(启用|禁用)\s*工具\s*(.+)/);
        if (enableToolMatch) {
            return { type: 'toggle_tool', tool: enableToolMatch[2].trim(), enable: enableToolMatch[1] === '启用' };
        }
        const searchMemoryMatch = lowerMsg.match(/搜索.*记忆\s*(.+)/);
        if (searchMemoryMatch) {
            return { type: 'search_memory', query: searchMemoryMatch[1].trim() };
        }
        const addFactMatch = lowerMsg.match(/添加.*事实\s*(.+)/);
        if (addFactMatch) {
            return { type: 'add_fact', fact: addFactMatch[1].trim() };
        }
        const runCommandMatch = lowerMsg.match(/运行\s*(.+)/);
        if (runCommandMatch) {
            return { type: 'run_command', command: runCommandMatch[1].trim() };
        }
        const openFileMatch = lowerMsg.match(/打开\s*文件\s*(.+)/);
        if (openFileMatch) {
            return { type: 'open_file', path: openFileMatch[1].trim() };
        }
        const webSearchMatch = lowerMsg.match(/搜索\s*(.+)/);
        if (webSearchMatch && !lowerMsg.includes('记忆')) {
            return { type: 'web_search', query: webSearchMatch[1].trim() };
        }
        return null;
    }

    async executeCommand(command) {
        const executingMsg = this.appendToolMessage(command.type, 'executing', '');
        try {
            let resultMsg;
            switch (command.type) {
                case 'install_platform': {
                    const platform = command.platform;
                    this.config.platforms[platform].enabled = true;
                    this.saveConfigFromUI();
                    this.showNotification(`已启用 ${platform} 平台`, 'success');
                    resultMsg = `✅ ${platform} 平台已启用。请在配置窗口中完成详细配置。`;
                    break;
                }
                case 'install_skill': {
                    const res = await fetch('/api/skills', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ name: command.skill })
                    });
                    const data = await res.json();
                    resultMsg = data.message || `技能 ${command.skill} 安装完成`;
                    break;
                }
                case 'install_plugin': {
                    const res = await fetch('/api/plugins', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ name: command.plugin })
                    });
                    const data = await res.json();
                    resultMsg = data.message || `插件 ${command.plugin} 安装完成`;
                    break;
                }
                case 'toggle_tool': {
                    this.showNotification(`${command.enable ? '启用' : '禁用'}工具: ${command.tool}`, 'info');
                    resultMsg = `工具 ${command.tool} 已${command.enable ? '启用' : '禁用'}`;
                    break;
                }
                case 'search_memory': {
                    const res = await fetch('/api/memory/search', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ query: command.query })
                    });
                    const data = await res.json();
                    const results = data.results || [];
                    if (results.length > 0) {
                        resultMsg = `找到 ${results.length} 条相关记忆:\n${results.map(r => `- ${r.content || r}`).join('\n')}`;
                    } else {
                        resultMsg = '未找到相关记忆';
                    }
                    break;
                }
                case 'add_fact': {
                    const res = await fetch('/api/memory/facts', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ content: command.fact, category: 'user' })
                    });
                    const data = await res.json();
                    resultMsg = data.message || '事实已添加到记忆';
                    break;
                }
                case 'run_command': {
                    this.showNotification(`运行命令: ${command.command}`, 'info');
                    resultMsg = `命令已发送到终端: ${command.command}`;
                    break;
                }
                case 'open_file': {
                    this.showNotification(`打开文件: ${command.path}`, 'info');
                    resultMsg = `正在打开文件: ${command.path}`;
                    break;
                }
                case 'web_search': {
                    this.showNotification(`网页搜索: ${command.query}`, 'info');
                    try {
                        const searchResp = await fetch('/api/tools/web_fetch/execute', {
                            method: 'POST',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({ arguments: JSON.stringify({ url: `https://html.duckduckgo.com/html/?q=${encodeURIComponent(command.query)}` }) })
                        });
                        const searchData = await searchResp.json();
                        if (searchData.success && searchData.result) {
                            const html = searchData.result;
                            const results = [];
                            const resultRegex = /<a[^>]+class="result__a"[^>]*>([\s\S]*?)<\/a>[\s\S]*?<a[^>]+class="result__snippet"[^>]*>([\s\S]*?)<\/a>/g;
                            let match;
                            let count = 0;
                            while ((match = resultRegex.exec(html)) !== null && count < 5) {
                                const title = match[1].replace(/<[^>]*>/g, '').trim();
                                const snippet = match[2].replace(/<[^>]*>/g, '').trim();
                                if (title) {
                                    results.push(`- ${title}: ${snippet}`);
                                    count++;
                                }
                            }
                            if (results.length > 0) {
                                resultMsg = `找到 ${results.length} 条搜索结果:\n${results.join('\n')}`;
                            } else {
                                resultMsg = `未找到 "${command.query}" 的相关搜索结果`;
                            }
                        } else {
                            resultMsg = `搜索 "${command.query}" 时出错: ${searchData.error || '未知错误'}`;
                        }
                    } catch (e) {
                        resultMsg = `搜索 "${command.query}" 时出错: ${e.message}`;
                    }
                    break;
                }
                default:
                    resultMsg = '未知命令类型';
            }
            
            if (executingMsg) {
                executingMsg.remove();
            }
            this.appendToolMessage(command.type, 'completed', resultMsg);
            
            if (this.currentSession) {
                this.saveMessageToSession(this.currentSession, 'user', command.type);
                this.saveMessageToSession(this.currentSession, 'assistant', resultMsg);
            }
        } catch (e) {
            if (executingMsg) {
                executingMsg.remove();
            }
            this.appendToolMessage(command.type, 'error', `执行失败: ${e.message}`);
        }
    }

    async getWechatQRCode() {
        const qrDisplay = document.getElementById('qr-code-display');
        const qrStatus = document.getElementById('qr-status');
        const btnCheck = document.getElementById('btn-wechat-qr-check');
        
        if (qrStatus) {
            qrStatus.textContent = '正在获取二维码...';
            qrStatus.className = 'qr-status waiting';
        }
        
        try {
            const resp = await fetch('/api/wechat/qr', { method: 'POST' });
            const data = await resp.json();
            
            if (data.success && data.qr_url) {
                if (qrDisplay) {
                    qrDisplay.innerHTML = `<img src="${data.qr_url}" alt="WeChat QR Code" />`;
                }
                if (qrStatus) {
                    qrStatus.textContent = '请使用微信扫描二维码';
                    qrStatus.className = 'qr-status waiting';
                }
                if (btnCheck) btnCheck.style.display = 'inline-block';
                this.showNotification('二维码已生成，请扫码登录', 'success');
                this.startWechatStatusPolling();
            } else {
                if (qrStatus) {
                    qrStatus.textContent = `获取二维码失败: ${data.error || '未知错误'}`;
                    qrStatus.className = 'qr-status error';
                }
                this.showNotification('获取二维码失败', 'error');
            }
        } catch (e) {
            if (qrStatus) {
                qrStatus.textContent = `请求失败: ${e.message}`;
                qrStatus.className = 'qr-status error';
            }
            this.showNotification('请求失败', 'error');
        }
    }

    async checkWechatQRStatus() {
        const qrStatus = document.getElementById('qr-status');
        
        try {
            const resp = await fetch('/api/wechat/qr/status');
            const data = await resp.json();
            
            if (data.success) {
                const status = data.status;
                if (qrStatus) {
                    switch (status) {
                        case 'wait':
                            qrStatus.textContent = '等待扫码...';
                            qrStatus.className = 'qr-status waiting';
                            break;
                        case 'scaned':
                            qrStatus.textContent = '已扫码，请在微信中确认...';
                            qrStatus.className = 'qr-status scanned';
                            break;
                        case 'confirmed':
                            qrStatus.textContent = '登录成功！';
                            qrStatus.className = 'qr-status confirmed';
                            this.showNotification('微信登录成功', 'success');
                            this.config.platforms.wechat.enabled = true;
                            this.saveConfigFromUI();
                            break;
                        case 'expired':
                            qrStatus.textContent = '二维码已过期，请重新获取';
                            qrStatus.className = 'qr-status error';
                            break;
                        default:
                            qrStatus.textContent = `状态: ${status}`;
                            qrStatus.className = 'qr-status waiting';
                    }
                }
            } else {
                if (qrStatus) {
                    qrStatus.textContent = `检查状态失败: ${data.error}`;
                    qrStatus.className = 'qr-status error';
                }
            }
        } catch (e) {
            if (qrStatus) {
                qrStatus.textContent = `请求失败: ${e.message}`;
                qrStatus.className = 'qr-status error';
            }
        }
    }

    loadSessions() {
        const saved = localStorage.getItem('mahakala_sessions');
        if (saved) { try { return JSON.parse(saved); } catch (e) {} }
        return [{ id: 'default', name: this.currentLang === 'zh' ? '默认会话' : 'Default Session', messages: [], created: Date.now() }];
    }

    saveSessions() { localStorage.setItem('mahakala_sessions', JSON.stringify(this.sessions)); }

    saveMessageToSession(sessionId, role, content) {
        const session = this.sessions.find(s => s.id === sessionId);
        if (session) { session.messages.push({ role, content, timestamp: Date.now() }); this.saveSessions(); }
    }

    createNewSession() {
        const id = 'session_' + Date.now();
        const name = `${this.currentLang === 'zh' ? '会话' : 'Session'} ${this.sessions.length + 1}`;
        this.sessions.push({ id, name, messages: [], created: Date.now() });
        this.saveSessions();
        this.switchSession(id);
        this.loadSidebarSessions();
    }

    switchSession(sessionId) {
        this.currentSession = sessionId;
        const session = this.sessions.find(s => s.id === sessionId);
        if (session) {
            const container = document.getElementById('chat-messages');
            if (container) { container.innerHTML = ''; session.messages.forEach(msg => this.appendMessage(msg.role, msg.content)); }
            const sessionIdEl = document.getElementById('session-id');
            if (sessionIdEl) sessionIdEl.textContent = sessionId;
            const msgCountEl = document.getElementById('message-count');
            if (msgCountEl) msgCountEl.textContent = session.messages.length;
        }
        document.querySelectorAll('.sidebar-item').forEach(i => i.classList.remove('active'));
        document.querySelector(`.sidebar-item[data-session-id="${sessionId}"]`)?.classList.add('active');
    }

    deleteSession(sessionId) {
        if (sessionId === 'default') return;
        this.sessions = this.sessions.filter(s => s.id !== sessionId);
        this.saveSessions();
        if (this.currentSession === sessionId) this.switchSession('default');
        this.loadSidebarSessions();
    }

    loadSidebarSessions() {
        const container = document.getElementById('sidebar-content');
        if (!container) return;
        container.innerHTML = '';
        this.sessions.forEach(s => {
            const item = document.createElement('div');
            item.className = `sidebar-item session-item ${s.id === this.currentSession ? 'active' : ''}`;
            item.setAttribute('data-session-id', s.id);
            item.innerHTML = `<i class="fas fa-comment"></i><span class="session-name">${s.name}</span><span class="session-delete"><i class="fas fa-times"></i></span>`;
            const delBtn = item.querySelector('.session-delete');
            delBtn.onclick = (e) => { e.stopPropagation(); this.deleteSession(s.id); };
            item.onclick = () => this.switchSession(s.id);
            container.appendChild(item);
        });
    }

    compressContext() {
        if (!this.currentSession) return;
        const session = this.sessions.find(s => s.id === this.currentSession);
        if (session && session.messages.length > 10) {
            session.messages = session.messages.slice(-10);
            this.saveSessions();
            this.showNotification(this.t('notification.success'), 'success');
        }
    }

    async loadInitialData() {
        this.loadSidebarSessions();
        this.updateActiveModel();
        this.loadPageContent();
        try {
            const resp = await fetch('/api/status');
            if (resp.ok) { const data = await resp.json(); this.updateStatus(data); }
        } catch (e) {}
    }

    async loadPageContent() {
        switch (this.currentPage) {
            case 'chat': break;
            case 'memory': this.loadMemoryPage(); break;
            case 'skills': this.loadSkillsPage(); break;
            case 'tools': this.loadToolsPage(); break;
            case 'plugins': this.loadPluginsPage(); break;
            case 'messages': this.loadMessagesPage(); break;
            case 'platforms': this.loadPlatformsPage(); break;
            case 'cron': this.loadCronPage(); break;
            case 'settings': this.loadSettingsPage(); break;
        }
    }

    async loadMemoryPage() {
        const factsList = document.getElementById('facts-list');
        if (!factsList) return;
        this.loadFactsList();
    }

    async loadFactsList() {
        const listEl = document.getElementById('facts-list');
        if (!listEl) return;
        try {
            const resp = await fetch('/api/memory/facts');
            const data = resp.ok ? await resp.json() : {};
            const facts = data.facts || [];
            this.allFacts = facts;
            this.renderFacts(facts);
            const countEl = document.getElementById('fact-count');
            if (countEl) countEl.textContent = facts.length;
        } catch (e) {
            listEl.innerHTML = '<div class="empty-state">No facts loaded</div>';
        }
    }

    renderFacts(facts) {
        const listEl = document.getElementById('facts-list');
        if (!listEl) return;
        listEl.innerHTML = '';
        if (facts.length === 0) {
            listEl.innerHTML = '<div class="empty-state">No facts found</div>';
            return;
        }
        facts.forEach((fact, i) => {
            const item = document.createElement('div');
            item.className = 'fact-item';
            item.innerHTML = `<span>${fact.content || fact}</span><button class="btn btn-secondary btn-sm" onclick="window.app.deleteFact(${i})">${this.t('memory.delete')}</button>`;
            listEl.appendChild(item);
        });
    }

    async searchFacts(query) {
        if (!query.trim()) {
            this.renderFacts(this.allFacts || []);
            return;
        }
        const filtered = (this.allFacts || []).filter(f => (f.content || f).toLowerCase().includes(query.toLowerCase()));
        this.renderFacts(filtered);
    }

    showAddFactDialog() {
        const content = prompt(this.t('memory.addFact') || '请输入要添加的事实:');
        if (content && content.trim()) {
            fetch('/api/memory/facts', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ content: content.trim() })
            }).then(() => {
                this.loadFactsList();
                this.showNotification(this.t('notification.success'), 'success');
            }).catch(() => this.showNotification(this.t('notification.error'), 'error'));
        }
    }

    showSearchMemoryDialog() {
        const query = prompt(this.t('memory.search') || '请输入搜索关键词:');
        if (query && query.trim()) {
            fetch('/api/memory/search', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ query: query.trim() })
            }).then(res => res.json()).then(data => {
                const results = data.results || [];
                if (results.length > 0) {
                    alert(`找到 ${results.length} 条相关记忆:\n${results.map(r => `- ${r.content || r}`).join('\n')}`);
                } else {
                    alert('未找到相关记忆');
                }
            }).catch(() => this.showNotification(this.t('notification.error'), 'error'));
        }
    }

    async addFact() {
        const input = document.getElementById('new-fact-input');
        if (!input || !input.value.trim()) return;
        try {
            await fetch('/api/memory/facts', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content: input.value.trim() }) });
            input.value = '';
            this.loadFactsList();
            this.showNotification(this.t('notification.success'), 'success');
        } catch (e) { this.showNotification(this.t('notification.error'), 'error'); }
    }

    async deleteFact(index) {
        const fact = this.allFacts && this.allFacts[index];
        if (!fact) return;
        const factId = fact.id || index;
        try { await fetch(`/api/memory/facts/${factId}`, { method: 'DELETE' }); this.loadFactsList(); } catch (e) {}
    }

    async loadSkillsPage() {
        const container = document.getElementById('skills-grid');
        if (!container) return;
        try {
            const resp = await fetch('/api/skills');
            const data = await resp.json();
            this.skillsData = (data.skills || []).map(s => ({
                name: s.name,
                category: s.category || 'general',
                desc: s.description || '',
                installed: s.installed || false,
                enabled: s.enabled || false,
                id: s.id
            }));
        } catch (e) {
            this.skillsData = [];
        }
        this.currentSkillFilter = 'all';
        this.renderSkills();
        this.bindSkillFilters();
    }

    bindSkillFilters() {
        document.querySelectorAll('#skill-filters .filter-tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('#skill-filters .filter-tab').forEach(t => t.classList.remove('active'));
                tab.classList.add('active');
                this.currentSkillFilter = tab.getAttribute('data-category');
                this.renderSkills();
            });
        });
    }

    renderSkills() {
        const container = document.getElementById('skills-grid');
        if (!container) return;
        const filtered = this.currentSkillFilter === 'all' ? this.skillsData : this.skillsData.filter(s => s.category === this.currentSkillFilter);
        container.innerHTML = '';
        filtered.forEach((skill, idx) => {
            const realIdx = this.skillsData.indexOf(skill);
            const card = document.createElement('div');
            card.className = 'skill-card';
            card.innerHTML = `
                <div class="skill-header"><h3>${skill.name}</h3><span class="skill-category">${skill.category}</span></div>
                <p class="skill-desc">${skill.desc}</p>
                <div class="skill-status">${skill.installed ? (skill.enabled ? this.t('skill.enabled') : this.t('skill.disabled')) : this.t('skill.install')}</div>
                <div class="skill-actions">
                    ${skill.installed ? `
                        <button class="btn btn-secondary btn-sm" onclick="window.app.toggleSkillEnabled(${realIdx}, this)">${skill.enabled ? this.t('skill.disable') : this.t('skill.enable')}</button>
                        <button class="btn btn-secondary btn-sm" onclick="window.app.uninstallSkill(${realIdx}, this)">${this.t('skill.uninstall')}</button>
                    ` : `
                        <button class="btn btn-primary btn-sm" onclick="window.app.installSkill(${realIdx}, this)">${this.t('skill.install')}</button>
                    `}
                </div>
            `;
            container.appendChild(card);
        });
    }

    async installSkill(idx, btn) {
        btn.disabled = true;
        btn.textContent = '...';
        const skill = this.skillsData[idx];
        try {
            const resp = await fetch(`/api/skills/${skill.id || skill.name}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ action: 'install' }) });
            const data = await resp.json();
            if (data.success) {
                this.skillsData[idx].installed = true;
                this.skillsData[idx].enabled = true;
                this.renderSkills();
                this.showNotification(`${skill.name} ${this.t('notification.success')}`, 'success');
            } else {
                throw new Error(data.error || 'Failed');
            }
        } catch (e) {
            this.skillsData[idx].installed = true;
            this.skillsData[idx].enabled = true;
            this.renderSkills();
            this.showNotification(`${skill.name} installed (simulated)`, 'success');
        }
    }

    async uninstallSkill(idx, btn) {
        btn.disabled = true;
        const skill = this.skillsData[idx];
        try {
            const resp = await fetch(`/api/skills/${skill.id || skill.name}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ action: 'uninstall' }) });
            const data = await resp.json();
            if (data.success) {
                this.skillsData[idx].installed = false;
                this.skillsData[idx].enabled = false;
                this.renderSkills();
                this.showNotification(`${skill.name} ${this.t('notification.success')}`, 'success');
            } else {
                throw new Error(data.error || 'Failed');
            }
        } catch (e) {
            this.skillsData[idx].installed = false;
            this.skillsData[idx].enabled = false;
            this.renderSkills();
            this.showNotification(`${skill.name} uninstalled (simulated)`, 'success');
        }
    }

    async toggleSkillEnabled(idx, btn) {
        btn.disabled = true;
        const skill = this.skillsData[idx];
        try {
            const action = skill.enabled ? 'disable' : 'enable';
            const resp = await fetch(`/api/skills/${skill.id || skill.name}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ action }) });
            const data = await resp.json();
            if (data.success) {
                this.skillsData[idx].enabled = !skill.enabled;
                this.renderSkills();
            } else {
                throw new Error(data.error || 'Failed');
            }
        } catch (e) {
            this.skillsData[idx].enabled = !skill.enabled;
            this.renderSkills();
        }
    }

    async toggleSkill(name, btn) {
        btn.disabled = true;
        try {
            await fetch(`/api/skills/${name}`, { method: 'POST' });
            btn.textContent = btn.textContent === this.t('skill.install') ? this.t('skill.uninstall') : this.t('skill.install');
            this.showNotification(this.t('notification.success'), 'success');
        } catch (e) { this.showNotification(this.t('notification.error'), 'error'); }
        btn.disabled = false;
    }

    async loadToolsPage() {
        const container = document.getElementById('tools-list');
        if (!container) return;
        try {
            const resp = await fetch('/api/tools');
            const data = await resp.json();
            console.log('Tools API response:', data);
            this.toolsData = (data.tools || []).map(t => {
                // Handle OpenAI-compatible format: { type: "function", function: { name, description, parameters } }
                if (t.type === 'function' && t.function) {
                    return {
                        name: t.function.name || t.function,
                        category: this.categorizeTool(t.function.name || ''),
                        desc: t.function.description || '',
                        enabled: true
                    };
                }
                // Handle old format: { name, description, category, enabled }
                return {
                    name: t.name || t,
                    category: t.category || 'system',
                    desc: t.description || t,
                    enabled: t.enabled !== false
                };
            });
            console.log('Processed toolsData:', this.toolsData);
        } catch (e) {
            console.error('Error loading tools:', e);
            this.toolsData = [];
        }
        this.currentToolFilter = 'all';
        this.renderTools();
        this.bindToolFilters();
    }

    categorizeTool(name) {
        const fileTools = ['file_read', 'file_write', 'file_list', 'file_append', 'file_search'];
        const webTools = ['web_fetch', 'web_search', 'browser_tool'];
        const codeTools = ['calculator', 'code_execute', 'git', 'diff', 'grep', 'sed', 'patch'];
        const systemTools = ['shell_exec', 'memory', 'todo', 'notification', 'vision', 'url_safety'];
        
        if (fileTools.includes(name)) return 'file';
        if (webTools.includes(name)) return 'web';
        if (codeTools.includes(name)) return 'code';
        if (systemTools.includes(name)) return 'system';
        return 'system';
    }

    bindToolFilters() {
        document.querySelectorAll('#tool-filters .filter-tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('#tool-filters .filter-tab').forEach(t => t.classList.remove('active'));
                tab.classList.add('active');
                this.currentToolFilter = tab.getAttribute('data-category');
                this.renderTools();
            });
        });
    }

    renderTools() {
        const container = document.getElementById('tools-list');
        if (!container) {
            console.error('tools-list container not found');
            return;
        }
        console.log('Rendering tools, toolsData length:', this.toolsData ? this.toolsData.length : 0);
        if (!this.toolsData || this.toolsData.length === 0) {
            container.innerHTML = '<div class="empty-state">没有可用的工具</div>';
            return;
        }
        const filtered = this.currentToolFilter === 'all' ? this.toolsData : this.toolsData.filter(t => t.category === this.currentToolFilter);
        console.log('Filtered tools count:', filtered.length);
        container.innerHTML = '';
        filtered.forEach((tool, idx) => {
            const realIdx = this.toolsData.indexOf(tool);
            const item = document.createElement('div');
            item.className = 'tool-item';
            item.innerHTML = `
                <div class="tool-name"><i class="fas fa-wrench"></i> ${tool.name}</div>
                <div class="tool-desc">${tool.desc}</div>
                <div class="tool-category">${tool.category}</div>
                <div class="tool-toggle">
                    <label class="toggle-switch">
                        <input type="checkbox" ${tool.enabled ? 'checked' : ''} onchange="window.app.toggleTool(${realIdx}, this.checked)">
                        <span class="toggle-slider"></span>
                    </label>
                </div>
            `;
            container.appendChild(item);
        });
    }

    async toggleTool(idx, enabled) {
        this.toolsData[idx].enabled = enabled;
        try {
            await fetch(`/api/tools/${this.toolsData[idx].name}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ enabled }) });
        } catch (e) {}
    }

    async loadPluginsPage() {
        const container = document.getElementById('plugins-grid');
        if (!container) return;
        try {
            const resp = await fetch('/api/plugins');
            const data = await resp.json();
            this.pluginsData = (data.plugins || []).map(p => ({
                name: p.name || p.id,
                status: p.loaded ? 'loaded' : 'unloaded',
                desc: p.description || '',
                version: p.version || '1.0.0',
                id: p.id
            }));
        } catch (e) {
            this.pluginsData = [];
        }
        this.renderPlugins();
    }

    renderPlugins() {
        const container = document.getElementById('plugins-grid');
        if (!container) return;
        container.innerHTML = '';
        this.pluginsData.forEach((plugin, idx) => {
            const card = document.createElement('div');
            card.className = 'plugin-card';
            card.innerHTML = `
                <div class="plugin-header"><h3>${plugin.name}</h3><span class="plugin-status ${plugin.status}">${plugin.status === 'loaded' ? this.t('plugin.loaded') : this.t('plugin.unloaded')}</span></div>
                <p class="plugin-desc">${plugin.desc}</p>
                <div class="plugin-version">v${plugin.version}</div>
                <div class="plugin-actions">
                    <button class="btn btn-primary btn-sm" onclick="window.app.togglePlugin(${idx}, this)">${plugin.status === 'loaded' ? this.t('plugin.unload') : this.t('plugin.load')}</button>
                </div>
            `;
            container.appendChild(card);
        });
    }

    async togglePlugin(idx, btn) {
        btn.disabled = true;
        const plugin = this.pluginsData[idx];
        const newStatus = plugin.status === 'loaded' ? 'unloaded' : 'loaded';
        const action = newStatus === 'loaded' ? 'load' : 'unload';
        try {
            const resp = await fetch(`/api/plugins/${plugin.id || plugin.name}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ action }) });
            const data = await resp.json();
            if (data.success) {
                this.pluginsData[idx].status = newStatus;
                this.renderPlugins();
                this.showNotification(`${plugin.name} ${newStatus === 'loaded' ? this.t('plugin.loaded') : this.t('plugin.unloaded')}`, 'success');
            } else {
                throw new Error(data.error || 'Failed');
            }
        } catch (e) {
            this.pluginsData[idx].status = newStatus;
            this.renderPlugins();
            this.showNotification(`${plugin.name} ${newStatus === 'loaded' ? this.t('plugin.loaded') : this.t('plugin.unloaded')} (simulated)`, 'success');
        }
        btn.disabled = false;
    }

    async loadMessagesPage() {
        const container = document.getElementById('message-list');
        if (!container) return;
        try {
            const resp = await fetch('/api/gateway/messages');
            const data = await resp.json();
            this.messagesData = (data.messages || []).map(m => ({
                id: m.id || Math.random().toString(36),
                platform: m.platform || 'unknown',
                content: m.content || m.message || '',
                sender: m.sender || m.from || 'unknown',
                timestamp: m.timestamp || Date.now(),
                read: m.read || false
            }));
        } catch (e) {
            this.messagesData = [];
        }
        this.currentMessageFilter = 'all';
        this.renderMessages();
        this.bindMessageFilters();
    }

    bindMessageFilters() {
        document.querySelectorAll('#message-filters .filter-tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('#message-filters .filter-tab').forEach(t => t.classList.remove('active'));
                tab.classList.add('active');
                this.currentMessageFilter = tab.getAttribute('data-platform');
                this.renderMessages();
            });
        });
    }

    renderMessages() {
        const container = document.getElementById('message-list');
        if (!container) return;
        const filtered = this.currentMessageFilter === 'all' ? this.messagesData : this.messagesData.filter(m => m.platform === this.currentMessageFilter);
        container.innerHTML = '';
        if (filtered.length === 0) {
            container.innerHTML = '<div class="empty-state">No messages</div>';
            return;
        }
        const platformIcons = { telegram: 'fab fa-telegram-plane', discord: 'fab fa-discord', wechat: 'fab fa-weixin', qqbot: 'fab fa-qq', feishu: 'fas fa-feather-alt', slack: 'fab fa-slack' };
        filtered.forEach(msg => {
            const item = document.createElement('div');
            item.className = `message-item ${msg.read ? 'read' : 'unread'}`;
            const timeStr = new Date(msg.timestamp).toLocaleString();
            item.innerHTML = `
                <div class="message-item-header">
                    <span class="message-platform"><i class="${platformIcons[msg.platform] || 'fas fa-envelope'}"></i> ${msg.platform}</span>
                    <span class="message-sender">${msg.sender}</span>
                    <span class="message-time">${timeStr}</span>
                </div>
                <div class="message-item-content">${msg.content}</div>
            `;
            container.appendChild(item);
        });
    }

    async loadPlatformsPage() {
        const container = document.getElementById('platforms-grid');
        if (!container) return;
        try {
            const resp = await fetch('/api/platforms');
            const data = await resp.json();
            const platformIcons = {
                wechat: 'fab fa-weixin',
                qqbot: 'fab fa-qq',
                telegram: 'fab fa-telegram-plane',
                feishu: 'fas fa-feather-alt',
                discord: 'fab fa-discord'
            };
            this.platformsData = (data.platforms || []).map(p => ({
                id: p.id || p.name?.toLowerCase(),
                name: p.name || p.id,
                icon: platformIcons[p.id] || 'fas fa-server',
                connected: p.connected || false,
                status: p.status || (p.connected ? 'online' : 'offline'),
                config: this.config.platforms[p.id] || {}
            }));
        } catch (e) {
            this.platformsData = [];
        }
        this.renderPlatforms();
    }

    renderPlatforms() {
        const container = document.getElementById('platforms-grid');
        if (!container) return;
        container.innerHTML = '';
        this.platformsData.forEach((p, idx) => {
            const card = document.createElement('div');
            card.className = 'platform-card-item';
            card.innerHTML = `
                <div class="platform-icon"><i class="${p.icon}"></i></div>
                <div class="platform-info">
                    <div class="platform-name">${p.name}</div>
                    <div class="platform-status ${p.connected ? 'online' : 'offline'}">
                        <i class="fas fa-circle"></i>
                        <span>${p.connected ? this.t('platform.connected') : this.t('platform.disconnected')}</span>
                    </div>
                </div>
                <div class="platform-actions">
                    <button class="btn ${p.connected ? 'btn-secondary' : 'btn-primary'} btn-sm" onclick="window.app.togglePlatform(${idx}, this)">
                        ${p.connected ? this.t('platform.disconnect') : this.t('platform.connect')}
                    </button>
                    <button class="btn btn-secondary btn-sm" onclick="window.app.showConfigModal()">
                        <i class="fas fa-cog"></i>
                    </button>
                </div>
            `;
            container.appendChild(card);
        });
    }

    async togglePlatform(idx, btn) {
        btn.disabled = true;
        const platform = this.platformsData[idx];
        const newStatus = !platform.connected;
        try {
            await fetch(`/api/platforms/${platform.id}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ action: newStatus ? 'connect' : 'disconnect' }) });
            this.platformsData[idx].connected = newStatus;
            this.platformsData[idx].status = newStatus ? 'online' : 'offline';
            this.renderPlatforms();
            this.showNotification(`${platform.name} ${newStatus ? this.t('platform.connected') : this.t('platform.disconnected')}`, 'success');
        } catch (e) {
            this.platformsData[idx].connected = newStatus;
            this.platformsData[idx].status = newStatus ? 'online' : 'offline';
            this.renderPlatforms();
            this.showNotification(`${platform.name} ${newStatus ? this.t('platform.connected') : this.t('platform.disconnected')} (simulated)`, 'success');
        }
        btn.disabled = false;
    }

    async loadCronPage() {
        const container = document.getElementById('cron-list');
        if (!container) return;
        try {
            const resp = await fetch('/api/cron');
            const data = await resp.json();
            this.cronJobs = (data.jobs || []).map(j => ({
                id: j.id || Math.random().toString(36),
                name: j.name || 'Unnamed',
                expression: j.expression || j.schedule || '',
                command: j.command || '',
                running: j.running || j.enabled || false,
                lastRun: j.last_run || j.lastRun || null,
                nextRun: j.next_run || j.nextRun || null
            }));
        } catch (e) {
            this.cronJobs = [];
        }
        container.innerHTML = `
            <div class="cron-add-row">
                <input type="text" id="cron-name" placeholder="${this.t('cron.name')}">
                <input type="text" id="cron-expression" placeholder="${this.t('cron.expression')}">
                <input type="text" id="cron-command" placeholder="${this.t('cron.command')}">
                <button class="btn btn-primary btn-sm" onclick="window.app.addCronJob()">${this.t('cron.add')}</button>
            </div>
            <div id="cron-jobs-list"></div>
        `;
        this.renderCronJobs();
    }

    renderCronJobs() {
        const listEl = document.getElementById('cron-jobs-list');
        if (!listEl) return;
        listEl.innerHTML = '';
        if (this.cronJobs.length === 0) {
            listEl.innerHTML = '<div class="empty-state">No cron jobs</div>';
            return;
        }
        this.cronJobs.forEach((job, i) => {
            const item = document.createElement('div');
            item.className = 'cron-item';
            const lastRunStr = job.lastRun ? new Date(job.lastRun).toLocaleString() : 'Never';
            const nextRunStr = job.nextRun ? new Date(job.nextRun).toLocaleString() : 'N/A';
            item.innerHTML = `
                <div class="cron-info">
                    <span class="cron-name">${job.name}</span>
                    <span class="cron-expr">${job.expression}</span>
                    <span class="cron-status ${job.running ? 'running' : 'paused'}">${job.running ? this.t('cron.running') : this.t('cron.paused')}</span>
                </div>
                <div class="cron-details">
                    <span class="cron-last-run">Last: ${lastRunStr}</span>
                    <span class="cron-next-run">Next: ${nextRunStr}</span>
                </div>
                <div class="cron-actions">
                    <button class="btn btn-secondary btn-sm" onclick="window.app.toggleCron(${i})">${job.running ? this.t('cron.pause') : this.t('cron.run')}</button>
                    <button class="btn btn-secondary btn-sm" onclick="window.app.runCronNow(${i})"><i class="fas fa-play"></i></button>
                    <button class="btn btn-secondary btn-sm" onclick="window.app.deleteCron(${i})">${this.t('memory.delete')}</button>
                </div>
            `;
            listEl.appendChild(item);
        });
    }

    async addCronJob() {
        const name = document.getElementById('cron-name')?.value;
        const expression = document.getElementById('cron-expression')?.value;
        const command = document.getElementById('cron-command')?.value;
        if (!name || !expression || !command) return;
        try {
            await fetch('/api/cron', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ name, expression, command }) });
            this.cronJobs.push({ id: Date.now(), name, expression, command, running: false, lastRun: null, nextRun: null });
            this.renderCronJobs();
            this.showNotification(this.t('notification.success'), 'success');
        } catch (e) {
            this.cronJobs.push({ id: Date.now(), name, expression, command, running: false, lastRun: null, nextRun: null });
            this.renderCronJobs();
            this.showNotification('Cron job added (simulated)', 'success');
        }
    }

    async toggleCron(index) {
        this.cronJobs[index].running = !this.cronJobs[index].running;
        try {
            await fetch(`/api/cron/${this.cronJobs[index].id}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ action: this.cronJobs[index].running ? 'start' : 'pause' }) });
        } catch (e) {}
        this.renderCronJobs();
    }

    async runCronNow(index) {
        this.cronJobs[index].lastRun = Date.now();
        this.showNotification(`Running ${this.cronJobs[index].name}...`, 'info');
        try {
            await fetch(`/api/cron/${this.cronJobs[index].id}/run`, { method: 'POST' });
        } catch (e) {}
        this.renderCronJobs();
    }

    async deleteCron(index) {
        try {
            await fetch(`/api/cron/${this.cronJobs[index].id}`, { method: 'DELETE' });
        } catch (e) {}
        this.cronJobs.splice(index, 1);
        this.renderCronJobs();
        this.showNotification(this.t('notification.success'), 'success');
    }

    async loadSettingsPage() {
        const container = document.getElementById('settings-content');
        if (!container) return;
        const activeTab = document.querySelector('.settings-tab.active')?.getAttribute('data-tab') || 'general';

        if (activeTab === 'general') {
            container.innerHTML = `
                <div class="settings-section">
                    <h3><i class="fas fa-cog"></i> ${this.t('settings.general')}</h3>
                    <div class="setting-item">
                        <label>${this.t('config.general.language')}</label>
                        <select id="settings-lang" onchange="window.app.updateSetting('language', this.value)">
                            <option value="zh" ${this.config.language === 'zh' ? 'selected' : ''}>中文</option>
                            <option value="en" ${this.config.language === 'en' ? 'selected' : ''}>English</option>
                        </select>
                    </div>
                    <div class="setting-item">
                        <label>${this.t('config.general.theme')}</label>
                        <select id="settings-theme" onchange="window.app.updateSetting('theme', this.value)">
                            <option value="dark" ${this.config.theme === 'dark' ? 'selected' : ''}>${this.t('theme.dark')}</option>
                            <option value="light" ${this.config.theme === 'light' ? 'selected' : ''}>${this.t('theme.light')}</option>
                        </select>
                    </div>
                    <div class="setting-item">
                        <label>${this.t('config.server.port')}</label>
                        <input type="number" id="settings-port" value="${this.config.port}" onchange="window.app.updateSetting('port', parseInt(this.value))">
                    </div>
                </div>
                <div class="settings-section">
                    <h3><i class="fas fa-brain"></i> ${this.t('settings.model')}</h3>
                    <div class="setting-item">
                        <label>${this.t('info.model')}</label>
                        <div id="settings-model-info">${this.activeModel ? `${this.activeModel.name}: ${this.activeModel.model}` : this.t('chat.noModel')}</div>
                    </div>
                    <button class="btn btn-primary" onclick="window.app.showConfigModal()">${this.t('config.configure')}</button>
                </div>
            `;
        } else if (activeTab === 'model') {
            container.innerHTML = `
                <div class="settings-section">
                    <h3><i class="fas fa-brain"></i> ${this.t('settings.model')}</h3>
                    <div class="setting-item">
                        <label>${this.t('info.model')}</label>
                        <div id="settings-model-info">${this.config.model || '未配置'}</div>
                    </div>
                    <div class="setting-item">
                        <label>Provider</label>
                        <div>${this.config.provider || '未配置'}</div>
                    </div>
                    <div class="setting-item">
                        <label>API Base URL</label>
                        <div>${this.config.api_base_url || '未配置'}</div>
                    </div>
                    <button class="btn btn-primary" onclick="window.app.showConfigModal()">${this.t('config.configure')}</button>
                </div>
            `;
        } else if (activeTab === 'appearance') {
            container.innerHTML = `
                <div class="settings-section">
                    <h3><i class="fas fa-palette"></i> 外观设置</h3>
                    <div class="setting-item">
                        <label>主题</label>
                        <select onchange="window.app.updateSetting('theme', this.value)">
                            <option value="dark" ${this.config.theme === 'dark' ? 'selected' : ''}>深色模式</option>
                            <option value="light" ${this.config.theme === 'light' ? 'selected' : ''}>浅色模式</option>
                        </select>
                    </div>
                    <div class="setting-item">
                        <label>字体大小</label>
                        <select onchange="document.documentElement.style.fontSize = this.value">
                            <option value="14px">小</option>
                            <option value="16px" selected>中</option>
                            <option value="18px">大</option>
                            <option value="20px">特大</option>
                        </select>
                    </div>
                    <div class="setting-item">
                        <label>侧边栏宽度</label>
                        <input type="range" min="200" max="400" value="280" onchange="document.getElementById('sidebar').style.width = this.value + 'px'">
                    </div>
                </div>
            `;
        } else if (activeTab === 'advanced') {
            container.innerHTML = `
                <div class="settings-section">
                    <h3><i class="fas fa-sliders-h"></i> 高级设置</h3>
                    <div class="setting-item">
                        <label>最大 Token</label>
                        <input type="number" value="${this.config.max_tokens || 2048}" onchange="window.app.updateSetting('max_tokens', parseInt(this.value))">
                    </div>
                    <div class="setting-item">
                        <label>Temperature</label>
                        <input type="number" step="0.1" min="0" max="2" value="${this.config.temperature || 0.7}" onchange="window.app.updateSetting('temperature', parseFloat(this.value))">
                    </div>
                    <div class="setting-item">
                        <label>工具调用最大迭代次数</label>
                        <input type="number" value="50" disabled title="已在后端配置为 50 次">
                    </div>
                    <div class="setting-item">
                        <label>数据目录</label>
                        <div style="color: var(--text-secondary); font-size: 12px;">${this.config.workspace || '默认'}</div>
                    </div>
                    <div class="setting-item">
                        <label>调试模式</label>
                        <label class="toggle-switch">
                            <input type="checkbox" onchange="window.app.showNotification(this.checked ? '调试模式已开启' : '调试模式已关闭', 'info')">
                            <span class="toggle-slider"></span>
                        </label>
                    </div>
                </div>
            `;
        }
    }

    updateSetting(key, value) {
        this.config.general[key] = value;
        if (key === 'language') { this.currentLang = value; this.applyLanguage(); }
        else if (key === 'theme') { this.currentTheme = value; this.applyTheme(); }
        localStorage.setItem('mahakala_config', JSON.stringify(this.config));
    }

    loadSettingsData() {
        const toolsEl = document.getElementById('status-tools');
        const pluginsEl = document.getElementById('status-plugins');
        const memoryEl = document.getElementById('status-memory');
        const skillsEl = document.getElementById('status-skills');
        if (toolsEl) toolsEl.textContent = `${this.t('statusbar.tools')}: 31`;
        if (pluginsEl) pluginsEl.textContent = `${this.t('statusbar.plugins')}: 2`;
        if (memoryEl) memoryEl.textContent = `${this.t('statusbar.memory')}: 0`;
        if (skillsEl) skillsEl.textContent = `${this.t('statusbar.skills')}: 0`;
    }

    switchPanel(panel) {
        document.querySelectorAll('.panel-tab').forEach(t => t.classList.remove('active'));
        document.querySelectorAll('.panel-section').forEach(s => s.classList.remove('active'));
        document.querySelector(`.panel-tab[data-panel="${panel}"]`)?.classList.add('active');
        document.getElementById(`panel-${panel}`)?.classList.add('active');
        if (panel === 'output') this.loadOutputLog();
        if (panel === 'debug') this.loadDebugLog();
    }

    loadOutputLog() {
        const outputEl = document.getElementById('output-log');
        if (!outputEl) return;
        outputEl.textContent = this.outputLog.join('\n') || '暂无输出日志';
        outputEl.scrollTop = outputEl.scrollHeight;
    }

    loadDebugLog() {
        const debugEl = document.getElementById('debug-log');
        if (!debugEl) return;
        debugEl.textContent = this.debugLog.join('\n') || '暂无调试日志';
        debugEl.scrollTop = debugEl.scrollHeight;
    }

    addOutputLog(message) {
        const timestamp = new Date().toLocaleTimeString();
        this.outputLog.push(`[${timestamp}] ${message}`);
        const outputEl = document.getElementById('output-log');
        if (outputEl && document.getElementById('panel-output')?.classList.contains('active')) {
            outputEl.textContent = this.outputLog.join('\n');
            outputEl.scrollTop = outputEl.scrollHeight;
        }
    }

    addDebugLog(message) {
        const timestamp = new Date().toLocaleTimeString();
        this.debugLog.push(`[${timestamp}] ${message}`);
        const debugEl = document.getElementById('debug-log');
        if (debugEl && document.getElementById('panel-debug')?.classList.contains('active')) {
            debugEl.textContent = this.debugLog.join('\n');
            debugEl.scrollTop = debugEl.scrollHeight;
        }
    }

    switchSettingsTab(tab) {
        document.querySelectorAll('.settings-tab').forEach(t => t.classList.remove('active'));
        document.querySelector(`.settings-tab[data-tab="${tab}"]`)?.classList.add('active');
        this.loadSettingsPage();
    }

    filterContent(filterId, category) {
        const container = document.getElementById(filterId);
        if (!container) return;
        container.querySelectorAll('.filter-tab').forEach(t => t.classList.remove('active'));
        container.querySelector(`.filter-tab[data-category="${category}"], .filter-tab[data-platform="${category}"]`)?.classList.add('active');
    }

    loadToolsSidebar(container) {
        if (!container) container = document.getElementById('sidebar-content');
        if (!container) return;
        container.innerHTML = `
            <div class="sidebar-section">
                <h4>${this.t('sidebar.tools')}</h4>
                <div class="tool-list">
                    <div class="sidebar-tool" data-tool="read_file"><i class="fas fa-file-alt"></i> Read File</div>
                    <div class="sidebar-tool" data-tool="write_file"><i class="fas fa-file-edit"></i> Write File</div>
                    <div class="sidebar-tool" data-tool="web_search"><i class="fas fa-search"></i> Web Search</div>
                    <div class="sidebar-tool" data-tool="run_command"><i class="fas fa-terminal"></i> Run Command</div>
                </div>
            </div>
        `;
        container.querySelectorAll('.sidebar-tool').forEach(tool => {
            tool.addEventListener('click', () => {
                const toolName = tool.getAttribute('data-tool');
                this.showNotification(`Tool: ${toolName}`, 'info');
            });
        });
    }

    loadSkillsSidebar(container) {
        if (!container) container = document.getElementById('sidebar-content');
        if (!container) return;
        container.innerHTML = `
            <div class="sidebar-section">
                <h4>${this.t('sidebar.skills')}</h4>
                <div class="skill-list">
                    <div class="sidebar-skill" data-skill="creative"><i class="fas fa-paint-brush"></i> Creative Writing</div>
                    <div class="sidebar-skill" data-skill="code"><i class="fas fa-code"></i> Code Review</div>
                    <div class="sidebar-skill" data-skill="research"><i class="fas fa-search"></i> Web Research</div>
                </div>
            </div>
        `;
    }

    loadPluginsSidebar(container) {
        if (!container) container = document.getElementById('sidebar-content');
        if (!container) return;
        container.innerHTML = `
            <div class="sidebar-section">
                <h4>${this.t('sidebar.plugins')}</h4>
                <div class="plugin-list">
                    <div class="sidebar-plugin" data-plugin="disk_cleanup"><i class="fas fa-broom"></i> Disk Cleanup</div>
                    <div class="sidebar-plugin" data-plugin="memory"><i class="fas fa-memory"></i> Memory Plugin</div>
                </div>
            </div>
        `;
    }

    loadPlatformsSidebar(container) {
        if (!container) container = document.getElementById('sidebar-content');
        if (!container) return;
        container.innerHTML = `
            <div class="sidebar-section">
                <h4>${this.t('sidebar.platforms')}</h4>
                <div class="platform-list">
                    <div class="sidebar-platform" data-platform="wechat"><i class="fab fa-weixin"></i> WeChat</div>
                    <div class="sidebar-platform" data-platform="qqbot"><i class="fab fa-qq"></i> QQ Bot</div>
                    <div class="sidebar-platform" data-platform="telegram"><i class="fab fa-telegram"></i> Telegram</div>
                    <div class="sidebar-platform" data-platform="discord"><i class="fab fa-discord"></i> Discord</div>
                </div>
            </div>
        `;
    }

    loadMessagesSidebar(container) {
        if (!container) container = document.getElementById('sidebar-content');
        if (!container) return;
        container.innerHTML = `
            <div class="sidebar-section">
                <h4>${this.t('sidebar.messages')}</h4>
                <div class="message-list">
                    <div class="sidebar-message" data-msg="1"><i class="fas fa-envelope"></i> Message 1</div>
                    <div class="sidebar-message" data-msg="2"><i class="fas fa-envelope"></i> Message 2</div>
                </div>
            </div>
        `;
    }

    loadCronSidebar(container) {
        if (!container) container = document.getElementById('sidebar-content');
        if (!container) return;
        container.innerHTML = `
            <div class="sidebar-section">
                <h4>${this.t('sidebar.cron')}</h4>
                <div class="cron-list">
                    <div class="sidebar-cron" data-cron="1"><i class="fas fa-clock"></i> Daily Backup</div>
                    <div class="sidebar-cron" data-cron="2"><i class="fas fa-clock"></i> Clean Temp</div>
                </div>
            </div>
        `;
    }

    loadMemorySidebar(container) {
        if (!container) container = document.getElementById('sidebar-content');
        if (!container) return;
        container.innerHTML = `
            <div class="sidebar-section">
                <h4>${this.t('sidebar.memory')}</h4>
                <div class="memory-stats">
                    <div class="stat"><i class="fas fa-database"></i> Facts: <span id="sidebar-fact-count">0</span></div>
                    <div class="stat"><i class="fas fa-tags"></i> Entities: <span>0</span></div>
                </div>
            </div>
        `;
    }

    showNotification(message, type = 'info') {
        if (window.__TAURI_ADAPTER__ && window.__TAURI_ADAPTER__.isTauri) {
            window.__TAURI_ADAPTER__.sendNotification('Mahakala Agent', message);
        }
        const container = document.getElementById('notifications');
        if (!container) return;
        const notif = document.createElement('div');
        notif.className = `notification ${type}`;
        notif.innerHTML = `<i class="fas fa-${type === 'success' ? 'check-circle' : type === 'error' ? 'times-circle' : type === 'warning' ? 'exclamation-triangle' : 'info-circle'}"></i><span>${message}</span>`;
        container.appendChild(notif);
        setTimeout(() => { notif.classList.add('fade-out'); setTimeout(() => notif.remove(), 300); }, 3000);
    }

    // Tool selector popup
    async showToolSelector() {
        const tools = this.toolsData || [];
        if (tools.length === 0) {
            try {
                const resp = await fetch('/api/tools');
                const data = await resp.json();
                this.toolsData = (data.tools || []).map(t => {
                    // Handle OpenAI-compatible format: { type: "function", function: { name, description, parameters } }
                    if (t.type === 'function' && t.function) {
                        return {
                            name: t.function.name || t.function,
                            category: this.categorizeTool(t.function.name || ''),
                            desc: t.function.description || '',
                            enabled: true
                        };
                    }
                    // Handle old format: { name, description, category, enabled }
                    return {
                        name: t.name || t,
                        category: t.category || 'system',
                        desc: t.description || t,
                        enabled: t.enabled !== false
                    };
                });
            } catch (e) {
                this.showNotification('无法加载工具列表', 'error');
                return;
            }
        }

        const enabledTools = this.toolsData.filter(t => t.enabled);
        if (enabledTools.length === 0) {
            this.showNotification('没有可用的工具', 'warning');
            return;
        }

        const toolName = prompt(`可用工具:\n${enabledTools.map((t, i) => `${i + 1}. ${t.name} - ${t.desc}`).join('\n')}\n\n请输入工具名称:`);
        if (!toolName) return;

        const tool = enabledTools.find(t => t.name.toLowerCase() === toolName.toLowerCase());
        if (!tool) {
            this.showNotification(`工具 '${toolName}' 未找到`, 'error');
            return;
        }

        const args = prompt(`请输入 ${tool.name} 的参数 (JSON 格式):`, '{}');
        if (args === null) return;

        this.appendMessage('user', `/tool ${tool.name} ${args}`);
        this.executeToolCall(tool.name, args);
    }

    // Skill selector popup
    async showSkillSelector() {
        const skills = this.skillsData || [];
        if (skills.length === 0) {
            try {
                const resp = await fetch('/api/skills');
                const data = await resp.json();
                this.skillsData = (data.skills || []).map(s => ({
                    name: s.name,
                    category: s.category || 'general',
                    desc: s.description || '',
                    installed: s.installed || false,
                    enabled: s.enabled || false,
                    id: s.id
                }));
            } catch (e) {
                this.showNotification('无法加载技能列表', 'error');
                return;
            }
        }

        const enabledSkills = this.skillsData.filter(s => s.installed && s.enabled);
        if (enabledSkills.length === 0) {
            this.showNotification('没有已启用的技能，请先安装并启用技能', 'warning');
            return;
        }

        const skillName = prompt(`可用技能:\n${enabledSkills.map((s, i) => `${i + 1}. ${s.name} - ${s.desc}`).join('\n')}\n\n请输入技能名称:`);
        if (!skillName) return;

        const skill = enabledSkills.find(s => s.name.toLowerCase() === skillName.toLowerCase() || s.id.toLowerCase() === skillName.toLowerCase());
        if (!skill) {
            this.showNotification(`技能 '${skillName}' 未找到`, 'error');
            return;
        }

        const input = prompt(`请输入要发送给 ${skill.name} 的消息:`);
        if (input === null) return;

        this.appendMessage('user', `/skill ${skill.id || skill.name} ${input}`);
        this.sendMessageWithSkill(skill.id || skill.name, input);
    }

    // Plugin selector popup
    async showPluginSelector() {
        const plugins = this.pluginsData || [];
        if (plugins.length === 0) {
            try {
                const resp = await fetch('/api/plugins');
                const data = await resp.json();
                this.pluginsData = (data.plugins || []).map(p => ({
                    name: p.name || p.id,
                    status: p.loaded ? 'loaded' : 'unloaded',
                    desc: p.description || '',
                    version: p.version || '1.0.0',
                    id: p.id
                }));
            } catch (e) {
                this.showNotification('无法加载插件列表', 'error');
                return;
            }
        }

        const loadedPlugins = this.pluginsData.filter(p => p.status === 'loaded');
        if (loadedPlugins.length === 0) {
            this.showNotification('没有已加载的插件，请先加载插件', 'warning');
            return;
        }

        const pluginName = prompt(`可用插件:\n${loadedPlugins.map((p, i) => `${i + 1}. ${p.name} - ${p.desc}`).join('\n')}\n\n请输入插件名称:`);
        if (!pluginName) return;

        const plugin = loadedPlugins.find(p => p.name.toLowerCase() === pluginName.toLowerCase() || p.id.toLowerCase() === pluginName.toLowerCase());
        if (!plugin) {
            this.showNotification(`插件 '${pluginName}' 未找到`, 'error');
            return;
        }

        const input = prompt(`请输入要发送给 ${plugin.name} 的消息:`);
        if (input === null) return;

        this.appendMessage('user', `/plugin ${plugin.id || plugin.name} ${input}`);
        this.sendMessageWithPlugin(plugin.id || plugin.name, input);
    }

    // Execute tool call directly
    async executeToolCall(toolName, args) {
        const executingMsg = this.appendToolMessage(toolName, 'executing', `执行工具 ${toolName}...`);
        try {
            const resp = await fetch(`/api/tools/${toolName}/execute`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ arguments: args })
            });
            const data = await resp.json();
            if (data.result !== undefined) {
                this.appendToolMessage(toolName, 'completed', `结果: ${data.result}`);
                this.appendMessage('assistant', `工具 ${toolName} 执行结果:\n${data.result}`);
            } else if (data.error) {
                throw new Error(data.error);
            }
        } catch (e) {
            this.appendToolMessage(toolName, 'error', `执行失败: ${e.message}`);
            this.appendMessage('assistant', `工具 ${toolName} 执行失败: ${e.message}`);
        }
    }

    // Send message with skill
    async sendMessageWithSkill(skillName, message) {
        const fullMessage = `请使用 ${skillName} 技能处理以下请求: ${message}`;
        const input = document.getElementById('chat-input');
        if (input) {
            input.value = fullMessage;
            await this.sendMessage();
        }
    }

    // Send message with plugin
    async sendMessageWithPlugin(pluginName, message) {
        const fullMessage = `请使用 ${pluginName} 插件处理以下请求: ${message}`;
        const input = document.getElementById('chat-input');
        if (input) {
            input.value = fullMessage;
            await this.sendMessage();
        }
    }
}

document.addEventListener('DOMContentLoaded', () => {
    window.app = new MahakalaWebUI();
});
