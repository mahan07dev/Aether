// main.js – Full frontend with real‑time state sync
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;

// ---------- External link handler using backend ----------
async function openExternalUrl(url) {
      try {
        await window.__TAURI__.opener.openUrl(url);
    } 
    catch {
    try {
        await invoke('open_url', { url });
        console.log('[SUCCESS] Opened URL externally:', url);
    } catch (e) {
        console.error('[FAILED] Backend open failed:', e);
        // Fallback: try window.open (just in case)
        window.open(url, '_blank');
    } }
}

document.addEventListener('DOMContentLoaded', () => {
    document.querySelectorAll('a[href^="http"]').forEach(link => {
        link.addEventListener('click', function(e) {
            e.preventDefault();
            e.stopPropagation();
            const url = this.getAttribute('href');
            console.log('[LINK CLICKED] Opening:', url);
            openExternalUrl(url);
            return false;
        });
    });
});

// ---------- Translations ----------
const translations = {
    en: {
        appTitle: 'Aether Control',
        installedLabel: 'Installed',
        notInstalled: 'Not Installed',
        connectionLabel: 'Connection',
        checking: 'Checking…',
        disconnected: 'Disconnected',
        connect: 'Connect',
        disconnect: 'Disconnect',
        connecting: 'Connecting…',
        connected: 'Connected',
        installNotice1: '⚠️ Aether not found. Please download the appropriate version for your OS from',
        installNotice2: 'and place the <strong>aether</strong> folder in the same directory as this app.',
        protocolLabel: 'Protocol',
        protocolMasque: 'Masque',
        protocolWireguard: 'WireGuard',
        protocolWiw: 'Warp‑in‑Warp (Goo)',
        httpLabel: 'HTTP Version',
        http2: 'HTTP/2',
        http3: 'HTTP/3',
        speedLabel: 'Scan Speed',
        speedTurbo: '1 – Turbo',
        speedBalanced: '2 – Balanced',
        speedThorough: '3 – Thorough',
        speedStealth: '4 – Stealth',
        useLastLabel: 'Use last successful connection',
        socksLabel: 'SOCKS5 Proxy',
        statusLabel: 'Status',
        guideTitle: '📖 V2Ray Configuration Guide',
        guideIntro: 'Configure your V2Ray client with the following SOCKS5 settings:',
        guideAddressLabel: 'Address',
        guidePortLabel: 'Port',
        guideProtocolLabel: 'Protocol',
        guideAuth: 'No authentication required.',
        guideConfigLabel: 'Example `config.json` for V2Ray:',
        guideNote: 'Set your browser or system proxy to SOCKS5 at the above address.',
        logTitle: '📋 Output Log',
        ready: 'Aether Control ready.',
        checkingAether: 'Checking for aether…',
        logsCleared: 'Logs cleared.',
        disconnectSuccess: 'Disconnected by user.',
        disconnectError: 'Disconnect error: ',
        connectError: 'Connect error: ',
        processStarted: 'Aether process started. Waiting for connection…',
        processExited: 'Aether process ended.',
        processUnexpected: 'Aether process exited unexpectedly.',
        connectionFailed: 'Connection failed. Check logs for details.',
        connectionTimeout: 'Connection timeout. Check logs for errors.',
        connectedMessage: '> Connected to SOCKS5 at ',
    },
    fa: {
        appTitle: 'کنترل اتر',
        installedLabel: 'نصب شده',
        notInstalled: 'نصب نشده',
        connectionLabel: 'اتصال',
        checking: 'در حال بررسی…',
        disconnected: 'قطع شده',
        connect: 'اتصال',
        disconnect: 'قطع',
        connecting: 'در حال اتصال…',
        connected: 'متصل',
        installNotice1: '⚠️ اتر یافت نشد. لطفاً نسخه مناسب سیستمعامل خود را از',
        installNotice2: 'دانلود کرده و پوشه <strong>aether</strong> را در همان پوشه برنامه قرار دهید.',
        protocolLabel: 'پروتکل',
        protocolMasque: 'مسک',
        protocolWireguard: 'وایرگارد',
        protocolWiw: 'وارپ-در-وارپ (گو)',
        httpLabel: 'نسخه HTTP',
        http2: 'HTTP/2',
        http3: 'HTTP/3',
        speedLabel: 'سرعت اسکن',
        speedTurbo: '۱ – توربو',
        speedBalanced: '۲ – متعادل',
        speedThorough: '۳ – کامل',
        speedStealth: '۴ – مخفی',
        useLastLabel: 'استفاده از آخرین اتصال موفق',
        socksLabel: 'پروکسی SOCKS5',
        statusLabel: 'وضعیت',
        guideTitle: '📖 راهنمای تنظیم V2Ray',
        guideIntro: 'کلاینت V2Ray خود را با تنظیمات SOCKS5 زیر پیکربندی کنید:',
        guideAddressLabel: 'آدرس',
        guidePortLabel: 'پورت',
        guideProtocolLabel: 'پروتکل',
        guideAuth: 'بدون نیاز به احراز هویت.',
        guideConfigLabel: 'مثال `config.json` برای V2Ray:',
        guideNote: 'مرورگر یا پروکسی سیستم را روی SOCKS5 با آدرس بالا تنظیم کنید.',
        logTitle: '📋 گزارش خروجی',
        ready: 'کنترل اتر آماده است.',
        checkingAether: 'در حال بررسی اتر…',
        logsCleared: 'گزارش‌ها پاک شد.',
        disconnectSuccess: 'اتصال توسط کاربر قطع شد.',
        disconnectError: 'خطا در قطع اتصال: ',
        connectError: 'خطا در اتصال: ',
        processStarted: 'فرایند اتر شروع شد. در انتظار اتصال…',
        processExited: 'فرایند اتر پایان یافت.',
        processUnexpected: 'فرایند اتر به طور غیرمنتظره پایان یافت.',
        connectionFailed: 'اتصال ناموفق بود. برای جزئیات گزارش را بررسی کنید.',
        connectionTimeout: 'زمان اتصال به پایان رسید. خطاها را بررسی کنید.',
        connectedMessage: '> متصل به SOCKS5 در ',
    }
};

let currentLang = 'en';
let currentTheme = 'dark';

// ---------- DOM ----------
const installedDot = document.getElementById('installedDot');
const installedText = document.getElementById('installedText');
const connectionDot = document.getElementById('connectionDot');
const connectionText = document.getElementById('connectionText');
const installNotice = document.getElementById('installNotice');
const connectBtn = document.getElementById('connectBtn');
const protocolSelect = document.getElementById('protocolSelect');
const httpGroup = document.getElementById('httpGroup');
const httpSelect = document.getElementById('httpSelect');
const speedSelect = document.getElementById('speedSelect');
const useLastCheck = document.getElementById('useLast');
const logBox = document.getElementById('logBox');
const logCount = document.getElementById('logCount');
const clearLogsBtn = document.getElementById('clearLogsBtn');
const connectionInfo = document.getElementById('connectionInfo');
const socksAddress = document.getElementById('socksAddress');
const connStatusText = document.getElementById('connStatusText');
const guideAddress = document.getElementById('guideAddress');
const guidePort = document.getElementById('guidePort');
const themeToggle = document.getElementById('themeToggle');
const langToggle = document.getElementById('langToggle');

let isConnected = false;
let isInstalled = false;
let isConnecting = false;

// ---------- Internationalization ----------
function applyLanguage(lang) {
    const t = translations[lang];
    document.querySelectorAll('[data-i18n]').forEach(el => {
        const key = el.getAttribute('data-i18n');
        if (t[key]) {
            if (el.innerHTML.includes('<strong>')) {
                el.innerHTML = t[key];
            } else {
                el.textContent = t[key];
            }
        }
    });
    // Dynamic texts
    if (isConnected) {
        connectBtn.querySelector('.btn-label').textContent = t.disconnect;
        connStatusText.textContent = t.connected;
    } else if (isConnecting) {
        connectBtn.querySelector('.btn-label').textContent = t.connecting;
    } else {
        connectBtn.querySelector('.btn-label').textContent = t.connect;
    }
    installedText.textContent = isInstalled ? t.installed : t.notInstalled;
    if (!isInstalled) {
        document.querySelector('#installNotice p:first-child').innerHTML = t.installNotice1;
        document.querySelector('#installNotice p:last-child').innerHTML = t.installNotice2;
    }
    document.querySelector('.guide-note').textContent = t.guideNote;
    document.querySelector('.config-example p').textContent = t.guideConfigLabel;
    document.querySelectorAll('.user-guide ul li strong[data-i18n]').forEach(el => {
        const key = el.getAttribute('data-i18n');
        if (t[key]) el.textContent = t[key];
    });
    currentLang = lang;
    langToggle.textContent = lang === 'en' ? '🇫🇦' : '🇬🇧';
    document.documentElement.dir = lang === 'fa' ? 'rtl' : 'ltr';
}

// ---------- Theme ----------
function toggleTheme() {
    const html = document.documentElement;
    const next = html.getAttribute('data-theme') === 'dark' ? 'light' : 'dark';
    html.setAttribute('data-theme', next);
    currentTheme = next;
    themeToggle.textContent = next === 'dark' ? '🌙' : '☀️';
    localStorage.setItem('aether-theme', next);
}
function loadTheme() {
    const saved = localStorage.getItem('aether-theme') || 'dark';
    document.documentElement.setAttribute('data-theme', saved);
    currentTheme = saved;
    themeToggle.textContent = saved === 'dark' ? '🌙' : '☀️';
}

// ---------- Helpers ----------
function appendLog(line, type = 'stdout') {
    const div = document.createElement('div');
    div.className = `log-line log-${type}`;
    div.textContent = line;
    logBox.appendChild(div);
    logBox.scrollTop = logBox.scrollHeight;
    logCount.textContent = `${logBox.querySelectorAll('.log-line').length} lines`;

    if (type === 'stdout' || type === 'stderr') {
        const match = line.match(/socks5 listening on ([0-9.]+):([0-9]+)/);
        if (match) {
            const ip = match[1];
            const port = match[2];
            onConnectionSuccess(ip, port);
        }
    }
}

function clearLogs() {
    logBox.innerHTML = '';
    const t = translations[currentLang];
    appendLog(`> ${t.logsCleared}`, 'system');
}

function setInstalled(installed) {
    isInstalled = installed;
    const t = translations[currentLang];
    if (installed) {
        installedDot.className = 'status-dot green';
        installedText.textContent = t.installed;
        installNotice.style.display = 'none';
        connectBtn.disabled = false;
    } else {
        installedDot.className = 'status-dot red';
        installedText.textContent = t.notInstalled;
        installNotice.style.display = 'block';
        connectBtn.disabled = true;
    }
}

function setConnected(connected) {
    isConnected = connected;
    const t = translations[currentLang];
    if (connected) {
        connectionDot.className = 'status-dot cyan';
        connectionText.textContent = t.connected;
        connectionInfo.style.display = 'block';
        connectBtn.classList.add('connected');
        connectBtn.querySelector('.btn-label').textContent = t.disconnect;
        connectBtn.disabled = false;
    } else {
        connectionDot.className = 'status-dot gray';
        connectionText.textContent = t.disconnected;
        connectionInfo.style.display = 'none';
        connectBtn.classList.remove('connected');
        connectBtn.querySelector('.btn-label').textContent = t.connect;
        connectBtn.disabled = !isInstalled;
    }
    isConnecting = false;
}

function setConnecting() {
    isConnecting = true;
    const t = translations[currentLang];
    connectionDot.className = 'status-dot gray';
    connectionText.textContent = t.connecting;
    connectionInfo.style.display = 'none';
    connectBtn.classList.remove('connected');
    connectBtn.querySelector('.btn-label').textContent = t.connecting;
    connectBtn.disabled = true;
}

function onConnectionSuccess(ip, port) {
    if (!isConnecting && isConnected) return;
    const t = translations[currentLang];
    socksAddress.textContent = `${ip}:${port}`;
    guideAddress.textContent = ip;
    guidePort.textContent = port;
    connStatusText.textContent = t.connected;
    setConnected(true);
    appendLog(`${t.connectedMessage}${ip}:${port}`, 'system');
    // Update V2Ray config
    const configExample = document.getElementById('v2rayConfig');
    configExample.textContent = JSON.stringify({
        inbounds: [{
            protocol: "socks",
            settings: { auth: "noauth", udp: true, ip, port: parseInt(port) }
        }],
        outbounds: [{ protocol: "freedom", settings: {} }]
    }, null, 2);
}

function toggleHttpGroup() {
    httpGroup.style.display = parseInt(protocolSelect.value) === 1 ? 'flex' : 'none';
}
protocolSelect.addEventListener('change', toggleHttpGroup);
toggleHttpGroup();

// ---------- Tauri Events ----------
listen('log', (event) => {
    const payload = event.payload;
    try {
        const data = JSON.parse(payload);
        appendLog(data.message, data.level || 'stdout');
    } catch {
        appendLog(String(payload), 'stdout');
    }
});

listen('process_exited', () => {
    const t = translations[currentLang];
    if (isConnected || isConnecting) {
        setConnected(false);
        appendLog(`> ${t.processUnexpected}`, 'stderr');
    }
});

// ---------- State sync ----------
async function syncConnectionState() {
    try {
        const running = await invoke('is_running');
        if (running && !isConnected) {
            setConnected(true);
            appendLog('> Aether process is already running.', 'system');
        } else if (!running && isConnected) {
            setConnected(false);
            appendLog('> Aether process is no longer running.', 'system');
        }
    } catch (e) {
        console.error('[SYNC STATE] Error:', e);
    }
}

// ---------- Connect / Disconnect ----------
connectBtn.addEventListener('click', async () => {
    const t = translations[currentLang];
    if (isConnected) {
        try {
            await invoke('disconnect');
            setConnected(false);
            appendLog(`> ${t.disconnectSuccess}`, 'system');
        } catch (e) {
            appendLog(`${t.disconnectError}${e}`, 'stderr');
        }
        return;
    }

    const protocol = parseInt(protocolSelect.value);
    const speed = parseInt(speedSelect.value);
    const use_last = useLastCheck.checked;
    let http_version = 2;
    if (protocol === 1) http_version = parseInt(httpSelect.value);

    const args = { params: { protocol, http_version, speed, use_last } };
    setConnecting();
    try {
        await invoke('connect', args);
        appendLog(`> ${t.processStarted}`, 'system');
        setTimeout(() => {
            if (isConnecting && !isConnected) {
                appendLog(`> ${t.connectionTimeout}`, 'stderr');
                setConnected(false);
            }
        }, 30000);
    } catch (e) {
        if (e?.toString()?.includes('Aether is already running')) {
            setConnected(true);
            appendLog('> Aether process is already running.', 'system');
        } else {
            appendLog(`${t.connectError}${e}`, 'stderr');
            setConnected(false);
        }
    }
});

// ---------- Theme & Language ----------
themeToggle.addEventListener('click', toggleTheme);
langToggle.addEventListener('click', () => {
    const next = currentLang === 'en' ? 'fa' : 'en';
    applyLanguage(next);
    // Update dynamic texts after language change
    const t = translations[next];
    if (!isConnected && !isConnecting) connectBtn.querySelector('.btn-label').textContent = t.connect;
    else if (isConnecting) connectBtn.querySelector('.btn-label').textContent = t.connecting;
    else if (isConnected) connectBtn.querySelector('.btn-label').textContent = t.disconnect;
    installedText.textContent = isInstalled ? t.installed : t.notInstalled;
    connectionText.textContent = isConnected ? t.connected : (isConnecting ? t.connecting : t.disconnected);
    connStatusText.textContent = isConnected ? t.connected : '';
    if (!isInstalled) {
        document.querySelector('#installNotice p:first-child').innerHTML = t.installNotice1;
        document.querySelector('#installNotice p:last-child').innerHTML = t.installNotice2;
    }
    document.querySelector('.config-example p').textContent = t.guideConfigLabel;
    document.querySelector('.guide-note').textContent = t.guideNote;
    document.querySelectorAll('.user-guide ul li strong[data-i18n]').forEach(el => {
        const key = el.getAttribute('data-i18n');
        if (t[key]) el.textContent = t[key];
    });
});

clearLogsBtn.addEventListener('click', clearLogs);

// ---------- Window close & focus ----------
const appWindow = getCurrentWindow();
appWindow.onCloseRequested(async () => {
    if (isConnected) {
        console.log('[CLOSE] Disconnecting...');
        try {
            await invoke('disconnect');
        } catch (e) { console.error(e); }
    }
});
window.addEventListener('beforeunload', async () => {
    if (isConnected) {
        try { await invoke('disconnect'); } catch (e) { /* ignore */ }
    }
});
appWindow.onFocusChanged(async (focused) => {
    if (focused) {
        console.log('[FOCUS] Syncing state...');
        await syncConnectionState();
    }
});

// ---------- Startup ----------
loadTheme();
applyLanguage('en');
const t = translations['en'];
appendLog(`> ${t.ready}`, 'system');
appendLog(`> ${t.checkingAether}`, 'system');

// Check installation, then sync state
(async () => {
    const installed = await invoke('check_installed').catch(() => false);
    setInstalled(installed);
    if (installed) await syncConnectionState();
})();