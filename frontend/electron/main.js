module.exports = require('./main.cjs');

const DAEMON_NAME = 'amux-daemon';
const CLI_NAME = 'amux';
const DAEMON_TCP_HOST = '127.0.0.1';
const DAEMON_TCP_PORT = 17563;
const MAX_TERMINAL_HISTORY_BYTES = 1024 * 1024;
let mainWindow = null;
const terminalBridges = new Map();

// ---------------------------------------------------------------------------
// Daemon path resolution — portable mode runs from same directory
// ---------------------------------------------------------------------------
function getDaemonPath() {
    return getCompanionBinaryPath(DAEMON_NAME);
}

function getCliPath() {
    return getCompanionBinaryPath(CLI_NAME);
}

function getCompanionBinaryPath(binaryName) {
    const isDev = !app.isPackaged;
    const exeName = binaryName + (process.platform === 'win32' ? '.exe' : '');

    if (isDev) {
        const repoRoot = path.join(__dirname, '..', '..');
        const candidates = [
            path.join(repoRoot, 'dist', exeName),
            path.join(repoRoot, 'target', 'release', exeName),
            path.join(repoRoot, 'target', 'debug', exeName),
            path.join(repoRoot, 'target', 'x86_64-pc-windows-gnu', 'release', exeName),
        ];

        const existing = candidates.find((candidate) => fs.existsSync(candidate));
        return existing || candidates[0];
    }

    const exeDir = path.dirname(app.getPath('exe'));
    const portablePath = path.join(exeDir, exeName);
    if (fs.existsSync(portablePath)) return portablePath;

    const resourcePath = path.join(process.resourcesPath, exeName);
    if (fs.existsSync(resourcePath)) return resourcePath;

    return portablePath;
}

function emitTerminalEvent(paneId, event) {
    mainWindow?.webContents.send('terminal-event', { paneId, ...event });
}

function rememberTerminalOutput(bridge, base64Chunk) {
    const size = Buffer.byteLength(base64Chunk, 'base64');
    bridge.outputHistory.push(base64Chunk);
    bridge.outputHistoryBytes += size;

    while (bridge.outputHistoryBytes > MAX_TERMINAL_HISTORY_BYTES && bridge.outputHistory.length > 1) {
        const removed = bridge.outputHistory.shift();
        if (!removed) break;
        bridge.outputHistoryBytes -= Buffer.byteLength(removed, 'base64');
    }
}

function sendBridgeCommand(bridge, command) {
    if (!bridge || bridge.process.killed || !bridge.process.stdin.writable) return;
    bridge.process.stdin.write(`${JSON.stringify(command)}\n`);
}

function stopTerminalBridge(paneId, killSession = false) {
    const bridge = terminalBridges.get(paneId);
    if (!bridge) return false;

    bridge.closing = true;
    if (!bridge.process.killed) {
        sendBridgeCommand(bridge, { type: killSession ? 'kill-session' : 'shutdown' });
        setTimeout(() => {
            if (!bridge.process.killed) {
                bridge.process.kill();
            }
        }, 500).unref?.();
    }

    terminalBridges.delete(paneId);
    return true;
}

function stopAllTerminalBridges(killSessions = false) {
    for (const paneId of [...terminalBridges.keys()]) {
        stopTerminalBridge(paneId, killSessions);
    }
}

async function startTerminalBridge(_event, options = {}) {
    const paneId = typeof options.paneId === 'string' ? options.paneId : '';
    if (!paneId) {
        throw new Error('paneId is required');
    }

    const existing = terminalBridges.get(paneId);
    if (existing) {
        return {
            sessionId: existing.sessionId,
            initialOutput: existing.outputHistory,
            state: existing.ready ? 'reachable' : 'checking',
        };
    }

    const daemonReady = await spawnDaemon();
    if (!daemonReady) {
        throw new Error('daemon is not reachable');
    }

    const cliPath = getCliPath();
    if (!fs.existsSync(cliPath)) {
        throw new Error(`amux CLI not found at ${cliPath}`);
    }

    const cols = Number.isFinite(options.cols) ? Math.max(2, Math.trunc(options.cols)) : 80;
    const rows = Number.isFinite(options.rows) ? Math.max(2, Math.trunc(options.rows)) : 24;
    const args = ['bridge', '--cols', String(cols), '--rows', String(rows)];

    if (typeof options.sessionId === 'string' && options.sessionId) {
        args.push('--session', options.sessionId);
    }
    if (typeof options.shell === 'string' && options.shell) {
        args.push('--shell', options.shell);
    }
    if (typeof options.cwd === 'string' && options.cwd) {
        args.push('--cwd', options.cwd);
    }
    if (typeof options.workspaceId === 'string' && options.workspaceId) {
        args.push('--workspace', options.workspaceId);
    }

    const bridgeProcess = spawn(cliPath, args, {
        cwd: path.dirname(cliPath),
        windowsHide: true,
        stdio: ['pipe', 'pipe', 'pipe'],
    });

    const bridge = {
        process: bridgeProcess,
        paneId,
        sessionId: options.sessionId ?? null,
        ready: false,
        closing: false,
        outputHistory: [],
        outputHistoryBytes: 0,
        stdoutBuffer: '',
        stderrBuffer: '',
    };

    terminalBridges.set(paneId, bridge);

    bridgeProcess.stdout.on('data', (chunk) => {
        bridge.stdoutBuffer += chunk.toString('utf8');
        const lines = bridge.stdoutBuffer.split(/\r?\n/);
        bridge.stdoutBuffer = lines.pop() ?? '';

        for (const line of lines) {
            if (!line.trim()) continue;

            let event;
            try {
                event = JSON.parse(line);
            } catch (error) {
                emitTerminalEvent(paneId, {
                    type: 'error',
                    message: `invalid bridge output: ${error.message}`,
                });
                continue;
            }

            if (event.type === 'ready') {
                bridge.ready = true;
                bridge.sessionId = event.session_id;
                emitTerminalEvent(paneId, {
                    type: 'ready',
                    sessionId: event.session_id,
                });
                continue;
            }

            if (event.type === 'output') {
                rememberTerminalOutput(bridge, event.data);
                emitTerminalEvent(paneId, {
                    type: 'output',
                    sessionId: event.session_id,
                    data: event.data,
                });
                continue;
            }

            if (event.type === 'session-exited') {
                emitTerminalEvent(paneId, {
                    type: 'session-exited',
                    sessionId: event.session_id,
                    exitCode: event.exit_code,
                });
                terminalBridges.delete(paneId);
                continue;
            }

            if (event.type === 'error') {
                emitTerminalEvent(paneId, {
                    type: 'error',
                    message: event.message,
                });
            }
        }
    });

    bridgeProcess.stderr.on('data', (chunk) => {
        bridge.stderrBuffer += chunk.toString('utf8');
        const message = bridge.stderrBuffer.trim();
        if (message) {
            emitTerminalEvent(paneId, { type: 'error', message });
            bridge.stderrBuffer = '';
        }
    });

    bridgeProcess.on('error', (error) => {
        emitTerminalEvent(paneId, {
            type: 'error',
            message: error.message,
        });
        terminalBridges.delete(paneId);
    });

    bridgeProcess.on('exit', (code, signal) => {
        if (!bridge.closing && code !== 0) {
            emitTerminalEvent(paneId, {
                type: 'error',
                message: `terminal bridge exited with ${signal ?? code}`,
            });
        }
        terminalBridges.delete(paneId);
    });

    return {
        sessionId: bridge.sessionId,
        initialOutput: [],
        state: 'checking',
    };
}

function sendTerminalInput(_event, paneId, data) {
    const bridge = terminalBridges.get(paneId);
    if (!bridge || typeof data !== 'string' || !data) return false;
    sendBridgeCommand(bridge, { type: 'input', data });
    return true;
}

function resizeTerminalSession(_event, paneId, cols, rows) {
    const bridge = terminalBridges.get(paneId);
    if (!bridge) return false;
    sendBridgeCommand(bridge, {
        type: 'resize',
        cols: Math.max(2, Math.trunc(cols)),
        rows: Math.max(2, Math.trunc(rows)),
    });
    return true;
}

function getDaemonEndpoint() {
    if (process.platform === 'win32') {
        return { host: DAEMON_TCP_HOST, port: DAEMON_TCP_PORT };
    }
    const runtimeDir = process.env.XDG_RUNTIME_DIR || '/tmp';
    return { path: path.join(runtimeDir, 'amux-daemon.sock') };
}

// ---------------------------------------------------------------------------
// Daemon lifecycle
// ---------------------------------------------------------------------------
async function checkDaemonRunning() {
    const endpoint = getDaemonEndpoint();
    return new Promise((resolve) => {
        const socket = new net.Socket();
        socket.setTimeout(1000);
        socket.once('connect', () => { socket.destroy(); resolve(true); });
        socket.once('error', () => { socket.destroy(); resolve(false); });
        socket.once('timeout', () => { socket.destroy(); resolve(false); });
        socket.connect(endpoint);
    });
}

async function spawnDaemon() {
    const isRunning = await checkDaemonRunning();
    if (isRunning) { console.log('[amux] Daemon already running'); return true; }

    const daemonPath = getDaemonPath();
    console.log('[amux] Spawning daemon:', daemonPath);

    if (!fs.existsSync(daemonPath)) {
        console.error('[amux] Daemon binary not found at:', daemonPath);
        return false;
    }

    const daemon = spawn(daemonPath, [], {
        detached: true, stdio: 'ignore', windowsHide: true,
        cwd: path.dirname(daemonPath),
    });
    daemon.on('error', (err) => console.error('[amux] Daemon error:', err));
    daemon.unref();

    for (let i = 0; i < 20; i++) {
        await new Promise(r => setTimeout(r, 250));
        if (await checkDaemonRunning()) { console.log('[amux] Daemon ready'); return true; }
    }
    console.warn('[amux] Daemon did not become ready');
    return false;
}

// ---------------------------------------------------------------------------
// System font enumeration
// ---------------------------------------------------------------------------
function getSystemFonts() {
    try {
        if (process.platform === 'win32') {
            const out = execSync(
                'powershell -NoProfile -Command "[System.Reflection.Assembly]::LoadWithPartialName(\'System.Drawing\') | Out-Null; (New-Object System.Drawing.Text.InstalledFontCollection).Families | ForEach-Object { $_.Name }"',
                { encoding: 'utf-8', timeout: 10000, windowsHide: true }
            );
            return out.split('\n').map(s => s.trim()).filter(Boolean).sort();
        } else {
            const out = execSync('fc-list --format="%{family[0]}\\n" | sort -u', {
                encoding: 'utf-8', timeout: 10000,
            });
            return out.split('\n').map(s => s.trim()).filter(Boolean);
        }
    } catch {
        return ['Cascadia Code','Cascadia Mono','Consolas','JetBrains Mono','Fira Code',
            'Source Code Pro','Hack','DejaVu Sans Mono','Ubuntu Mono','Courier New','monospace'];
    }
}

// ---------------------------------------------------------------------------
// Window
// ---------------------------------------------------------------------------
function createWindow() {
    const { width: screenW, height: screenH } = screen.getPrimaryDisplay().workAreaSize;

    mainWindow = new BrowserWindow({
        width: Math.min(1400, screenW), height: Math.min(900, screenH),
        minWidth: 600, minHeight: 400,
        frame: false, titleBarStyle: 'hidden',
        titleBarOverlay: process.platform === 'win32' ? {
            color: '#181825', symbolColor: '#cdd6f4', height: 36,
        } : undefined,
        webPreferences: {
            preload: path.join(__dirname, 'preload.js'),
            nodeIntegration: false, contextIsolation: true,
        },
        title: 'amux',
        icon: path.join(__dirname, '..', 'assets', 'icon.ico'),
        backgroundColor: '#1e1e2e', show: false,
    });

    const isDev = !app.isPackaged;
    if (isDev) mainWindow.loadURL('http://localhost:5173');
    else mainWindow.loadFile(path.join(__dirname, '..', 'dist', 'index.html'));

    mainWindow.once('ready-to-show', () => mainWindow.show());
    if (isDev) mainWindow.webContents.openDevTools();

    mainWindow.on('maximize', () => mainWindow.webContents.send('window-state', 'maximized'));
    mainWindow.on('unmaximize', () => mainWindow.webContents.send('window-state', 'normal'));
}

// ---------------------------------------------------------------------------
// IPC handlers
// ---------------------------------------------------------------------------
function registerIpcHandlers() {
    ipcMain.handle('getSocketPath', () => {
        const endpoint = getDaemonEndpoint();
        return endpoint.path ?? `${endpoint.host}:${endpoint.port}`;
    });
    ipcMain.handle('checkDaemon', () => checkDaemonRunning());
    ipcMain.handle('spawnDaemon', () => spawnDaemon());
    ipcMain.handle('getSystemFonts', () => getSystemFonts());
    ipcMain.handle('getDaemonPath', () => getDaemonPath());
    ipcMain.handle('getPlatform', () => process.platform);
    ipcMain.handle('clipboard-read-text', () => clipboard.readText());
    ipcMain.handle('clipboard-write-text', (_event, text) => {
        clipboard.writeText(typeof text === 'string' ? text : '');
        return true;
    });
    ipcMain.handle('terminal-start', startTerminalBridge);
    ipcMain.handle('terminal-input', sendTerminalInput);
    ipcMain.handle('terminal-resize', resizeTerminalSession);
    ipcMain.handle('terminal-stop', (_event, paneId, killSession) => stopTerminalBridge(paneId, Boolean(killSession)));
    ipcMain.handle('window-minimize', () => mainWindow?.minimize());
    ipcMain.handle('window-maximize', () => {
        if (mainWindow?.isMaximized()) mainWindow.unmaximize();
        else mainWindow?.maximize();
    });
    ipcMain.handle('window-close', () => mainWindow?.close());
    ipcMain.handle('window-isMaximized', () => mainWindow?.isMaximized() ?? false);
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------
app.whenReady().then(async () => {
    registerIpcHandlers();
    await spawnDaemon();
    createWindow();
    app.on('activate', () => {
        if (BrowserWindow.getAllWindows().length === 0) createWindow();
    });
});

app.on('before-quit', () => {
    stopAllTerminalBridges(false);
});

app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') app.quit();
});
