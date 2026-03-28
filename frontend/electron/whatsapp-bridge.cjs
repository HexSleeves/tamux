/**
 * WhatsApp Web bridge sidecar for tamux-gateway.
 *
 * Uses @whiskeysockets/baileys to connect to WhatsApp via the multi-device
 * protocol. Communicates with the Electron main process via JSON-RPC over
 * stdin/stdout.
 *
 * Protocol:
 *   Main -> Bridge:  { "id": 1, "method": "connect" }
 *   Bridge -> Main:  { "id": 1, "result": "ok" }
 *   Bridge -> Main:  {
 *     "event": "qr",
 *     "data": { "ascii_qr": "...", "data_url": "data:image/png;base64,..." }
 *   }
 *   Bridge -> Main:  { "event": "connected", "data": { "phone": "+1234..." } }
 *   Bridge -> Main:  { "event": "message", "data": { "from": "...", ... } }
 *   Bridge -> Main:  { "event": "disconnected" }
 *   Bridge -> Main:  { "event": "error", "data": "..." }
 */

const path = require('path');
const fs = require('fs');
const os = require('os');
const QRCode = require('qrcode');
const pino = require('pino');

// Auth state directory
const AUTH_DIR = path.join(
    process.platform === 'win32' && process.env.LOCALAPPDATA
        ? path.join(process.env.LOCALAPPDATA, 'tamux')
        : path.join(os.homedir(), '.tamux'),
    'whatsapp-auth'
);
fs.mkdirSync(AUTH_DIR, { recursive: true });

const logger = pino({ level: 'silent' });

let sock = null;
let isConnected = false;
let baileysApi = null;
let reconnectTimer = null;
let reconnectAttempt = 0;
let connectAttempt = 0;
const TERMINAL_RELINK_MAX_RETRIES = 1;

function clearReconnectTimer() {
    if (reconnectTimer) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
    }
}

function scheduleReconnect() {
    if (reconnectTimer) return;
    reconnectTimer = setTimeout(() => {
        reconnectTimer = null;
        connectWhatsApp().catch((err) => {
            sendEvent('error', `Reconnect failed: ${err.message || String(err)}`);
        });
    }, 3000);
}

function shouldTreatAsTerminalDisconnect(statusCode, reason, DisconnectReason) {
    if (statusCode === DisconnectReason.loggedOut) return true;
    if ([401, 403, 405].includes(statusCode)) return true;
    if (typeof reason === 'string' && /connection failure/i.test(reason)) return true;
    return false;
}

function resetAuthState() {
    try {
        fs.rmSync(AUTH_DIR, { recursive: true, force: true });
        fs.mkdirSync(AUTH_DIR, { recursive: true });
        return null;
    } catch (error) {
        return error;
    }
}

function shouldRetryTerminalRelink() {
    if (reconnectAttempt >= TERMINAL_RELINK_MAX_RETRIES) {
        return false;
    }
    reconnectAttempt += 1;
    return true;
}

async function getBaileysApi() {
    if (baileysApi) return baileysApi;
    const mod = await import('@whiskeysockets/baileys');
    baileysApi = {
        makeWASocket: mod.default,
        fetchLatestBaileysVersion: mod.fetchLatestBaileysVersion,
        DisconnectReason: mod.DisconnectReason,
        useMultiFileAuthState: mod.useMultiFileAuthState,
        makeCacheableSignalKeyStore: mod.makeCacheableSignalKeyStore,
        Browsers: mod.Browsers,
    };
    return baileysApi;
}

// ---------------------------------------------------------------------------
// JSON-RPC communication
// ---------------------------------------------------------------------------

function sendEvent(event, data) {
    const msg = JSON.stringify({ event, data });
    process.stdout.write(msg + '\n');
}

function emitTrace(phase, extra = {}) {
    sendEvent('trace', { phase, ...extra });
}

function summarizeReason(value) {
    if (typeof value !== 'string') return null;
    const trimmed = value.trim();
    return trimmed.length > 0 ? trimmed : null;
}

function sendResult(id, result) {
    const msg = JSON.stringify({ id, result });
    process.stdout.write(msg + '\n');
}

function sendError(id, error) {
    const msg = JSON.stringify({ id, error });
    process.stdout.write(msg + '\n');
}

// ---------------------------------------------------------------------------
// WhatsApp connection
// ---------------------------------------------------------------------------

async function connectWhatsApp() {
    connectAttempt += 1;
    emitTrace('connect_attempt', {
        connect_attempt: connectAttempt,
        relink_retry_attempt: reconnectAttempt,
    });
    const {
        makeWASocket,
        Browsers,
        fetchLatestBaileysVersion,
        DisconnectReason,
        useMultiFileAuthState,
        makeCacheableSignalKeyStore,
    } = await getBaileysApi();
    const { state, saveCreds } = await useMultiFileAuthState(AUTH_DIR);
    const { version } = await fetchLatestBaileysVersion();
    emitTrace('baileys_version', { version, connect_attempt: connectAttempt });

    sock = makeWASocket({
        version,
        auth: {
            creds: state.creds,
            keys: makeCacheableSignalKeyStore(state.keys, logger),
        },
        printQRInTerminal: false,
        logger,
        browser: Browsers.ubuntu('Chrome'),
        generateHighQualityLinkPreview: false,
    });

    sock.ev.on('creds.update', saveCreds);

    sock.ev.on('connection.update', async (update) => {
        const { connection, lastDisconnect, qr } = update;
        emitTrace('connection_update', {
            connection: connection || null,
            connect_attempt: connectAttempt,
            has_qr: Boolean(qr),
        });

        if (qr) {
            try {
                const asciiQr = await QRCode.toString(qr, {
                    type: 'utf8',
                    margin: 2,
                });
                const dataUrl = await QRCode.toDataURL(qr, {
                    width: 256,
                    margin: 2,
                    color: { dark: '#000000', light: '#ffffff' },
                });
                sendEvent('qr', {
                    ascii_qr: asciiQr,
                    data_url: dataUrl,
                    connect_attempt: connectAttempt,
                });
                emitTrace('qr_generated', {
                    connect_attempt: connectAttempt,
                    ascii_len: asciiQr.length,
                    has_data_url: true,
                });
            } catch (err) {
                sendEvent('error', `QR generation failed: ${err.message}`);
                emitTrace('qr_generation_failed', {
                    connect_attempt: connectAttempt,
                    error: err.message || String(err),
                });
            }
        }

        if (connection === 'close') {
            isConnected = false;
            sock = null;
            const statusCode = (lastDisconnect?.error)?.output?.statusCode;
            const numericStatusCode = Number.isFinite(statusCode) ? statusCode : null;
            const reconnectReason = summarizeReason(
                lastDisconnect?.error?.message ||
                lastDisconnect?.error?.toString?.() ||
                null
            );
            const reconnectData = lastDisconnect?.error?.data ?? null;
            const terminalDisconnect = shouldTreatAsTerminalDisconnect(
                numericStatusCode,
                reconnectReason,
                DisconnectReason
            );
            emitTrace('connection_closed', {
                status_code: numericStatusCode,
                reason: reconnectReason,
                reconnect_data: reconnectData,
                terminal_disconnect: terminalDisconnect,
                connect_attempt: connectAttempt,
            });

            if (!terminalDisconnect) {
                sendEvent('reconnecting', {
                    reason: reconnectReason,
                    status_code: numericStatusCode,
                    relink_retry_attempt: reconnectAttempt + 1,
                    connect_attempt: connectAttempt,
                });
                scheduleReconnect();
            } else {
                const resetError = resetAuthState();
                if (resetError) {
                    clearReconnectTimer();
                    sendEvent(
                        'error',
                        `Failed to reset WhatsApp auth state: ${resetError.message || String(resetError)}`
                    );
                    sendEvent('disconnected', {
                        reason: reconnectReason || 'auth_reset_failed',
                        status_code: numericStatusCode,
                        connect_attempt: connectAttempt,
                    });
                    return;
                }
                if (shouldRetryTerminalRelink()) {
                    sendEvent('reconnecting', {
                        reason: reconnectReason || 'terminal_relink_retry',
                        status_code: numericStatusCode,
                        relink_retry_attempt: reconnectAttempt,
                        connect_attempt: connectAttempt,
                    });
                    emitTrace('terminal_relink_retry', {
                        status_code: numericStatusCode,
                        reason: reconnectReason,
                        relink_retry_attempt: reconnectAttempt,
                        connect_attempt: connectAttempt,
                    });
                    scheduleReconnect();
                    return;
                }
                clearReconnectTimer();
                const reasonParts = [];
                if (numericStatusCode !== null) {
                    reasonParts.push(`status_code=${numericStatusCode}`);
                }
                if (reconnectReason) {
                    reasonParts.push(reconnectReason);
                }
                sendEvent(
                    'error',
                    `WhatsApp session requires relink${reasonParts.length ? ` (${reasonParts.join('; ')})` : ''}`
                );
                sendEvent('disconnected', {
                    reason: reconnectReason || null,
                    status_code: numericStatusCode,
                    connect_attempt: connectAttempt,
                });
            }
        } else if (connection === 'open') {
                clearReconnectTimer();
                reconnectAttempt = 0;
                isConnected = true;
                emitTrace('connected', { connect_attempt: connectAttempt });
                const phoneNumber = sock.user?.id?.split(':')[0] || 'Unknown';
            sendEvent('connected', { phone: `+${phoneNumber}` });
        }
    });

    sock.ev.on('messages.upsert', (m) => {
        if (m.type !== 'notify') return;
        for (const msg of m.messages) {
            if (msg.key.fromMe) continue; // skip own messages
            const text =
                msg.message?.conversation ||
                msg.message?.extendedTextMessage?.text ||
                msg.message?.imageMessage?.caption ||
                '';
            if (!text) continue;

            const from = msg.key.remoteJid || 'unknown';
            const pushName = msg.pushName || '';

            sendEvent('message', {
                from,
                pushName,
                text,
                timestamp: msg.messageTimestamp,
                messageId: msg.key.id,
            });
        }
    });
}

async function disconnectWhatsApp() {
    clearReconnectTimer();
    reconnectAttempt = 0;
    connectAttempt = 0;
    if (sock) {
        await sock.logout().catch(() => {});
        sock = null;
        isConnected = false;
    }
    emitTrace('manual_disconnect', {});
}

function getStatus() {
    if (!sock) return { status: 'disconnected', phone: null };
    if (isConnected) {
        const phoneNumber = sock.user?.id?.split(':')[0] || null;
        return { status: 'connected', phone: phoneNumber ? `+${phoneNumber}` : null };
    }
    return { status: 'connecting', phone: null };
}

async function sendWhatsAppMessage(jid, text) {
    if (!sock || !isConnected) {
        throw new Error('WhatsApp not connected');
    }
    await sock.sendMessage(jid, { text });
}

// ---------------------------------------------------------------------------
// stdin command handler
// ---------------------------------------------------------------------------

let inputBuffer = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', (chunk) => {
    inputBuffer += chunk;
    const lines = inputBuffer.split('\n');
    inputBuffer = lines.pop() || '';

    for (const line of lines) {
        if (!line.trim()) continue;
        try {
            const msg = JSON.parse(line);
            handleCommand(msg);
        } catch (err) {
            sendEvent('error', `Invalid JSON: ${err.message}`);
        }
    }
});

async function handleCommand(msg) {
    const { id, method, params } = msg;

    try {
        switch (method) {
            case 'connect':
                // Reply immediately so the RPC never times out.
                // QR / connected / error events arrive asynchronously.
                sendResult(id, 'ok');
                connectWhatsApp().catch((err) => {
                    sendEvent('error', `Connection failed: ${err.message || String(err)}`);
                });
                break;
            case 'disconnect':
                await disconnectWhatsApp();
                sendResult(id, 'ok');
                break;
            case 'status':
                sendResult(id, getStatus());
                break;
            case 'send':
                await sendWhatsAppMessage(params.jid, params.text);
                sendResult(id, 'ok');
                break;
            case 'ping':
                sendResult(id, 'pong');
                break;
            default:
                sendError(id, `Unknown method: ${method}`);
        }
    } catch (err) {
        sendError(id, err.message || String(err));
    }
}

// Graceful shutdown
process.on('SIGTERM', async () => {
    clearReconnectTimer();
    if (sock) await sock.end(undefined).catch(() => {});
    process.exit(0);
});

process.on('SIGINT', async () => {
    clearReconnectTimer();
    if (sock) await sock.end(undefined).catch(() => {});
    process.exit(0);
});

sendEvent('ready', null);
