const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const preloadPath = path.join(__dirname, "preload.cjs");
const mainPath = path.join(__dirname, "main.cjs");
const bridgePath = path.join(__dirname, "whatsapp-bridge.cjs");
const preloadSrc = fs.readFileSync(preloadPath, "utf8");
const mainSrc = fs.readFileSync(mainPath, "utf8");
const bridgeSrc = fs.readFileSync(bridgePath, "utf8");

test("preload keeps WhatsApp API names stable", () => {
  assert.match(preloadSrc, /whatsappConnect:\s*\(\)\s*=>\s*ipcRenderer\.invoke\('whatsapp-connect'\)/);
  assert.match(preloadSrc, /whatsappDisconnect:\s*\(\)\s*=>\s*ipcRenderer\.invoke\('whatsapp-disconnect'\)/);
  assert.match(preloadSrc, /whatsappStatus:\s*\(\)\s*=>\s*ipcRenderer\.invoke\('whatsapp-status'\)/);
  assert.match(preloadSrc, /whatsappSend:\s*\(jid, text\)\s*=>\s*ipcRenderer\.invoke\('whatsapp-send', jid, text\)/);
});

test("preload exposes daemon-backed WhatsApp event subscriptions", () => {
  assert.match(preloadSrc, /onWhatsAppQR:\s*\(cb\)\s*=>[\s\S]*?ipcRenderer\.on\('whatsapp-qr'/);
  assert.match(preloadSrc, /onWhatsAppConnected:\s*\(cb\)\s*=>[\s\S]*?ipcRenderer\.on\('whatsapp-connected'/);
  assert.match(preloadSrc, /onWhatsAppDisconnected:\s*\(cb\)\s*=>[\s\S]*?ipcRenderer\.on\('whatsapp-disconnected'/);
  assert.match(preloadSrc, /onWhatsAppError:\s*\(cb\)\s*=>[\s\S]*?ipcRenderer\.on\('whatsapp-error'/);
});

test("main uses daemon whatsapp-link protocol command names", () => {
  assert.match(mainSrc, /type:\s*'whats-app-link-start'/);
  assert.match(mainSrc, /type:\s*'whats-app-link-stop'/);
  assert.match(mainSrc, /type:\s*'whats-app-link-status'/);
  assert.match(mainSrc, /type:\s*'whats-app-link-subscribe'/);
  assert.match(mainSrc, /type:\s*'whats-app-link-unsubscribe'/);
});

test("whatsapp bridge treats 405/connection failure as terminal relink", () => {
  assert.match(bridgeSrc, /function shouldTreatAsTerminalDisconnect\(/);
  assert.match(bridgeSrc, /TERMINAL_RELINK_MAX_RETRIES\s*=\s*1/);
  assert.match(bridgeSrc, /function shouldRetryTerminalRelink\(/);
  assert.match(bridgeSrc, /if \(shouldRetryTerminalRelink\(\)\)/);
  assert.match(bridgeSrc, /terminal_relink_retry/);
  assert.match(bridgeSrc, /\[401,\s*403,\s*405\]\.includes\(statusCode\)/);
  assert.match(bridgeSrc, /connection failure\/i/);
  assert.match(bridgeSrc, /WhatsApp session requires relink/);
  assert.match(bridgeSrc, /resetAuthState\(\)/);
});

test("whatsapp bridge emits structured diagnostics and textual QR payload", () => {
  assert.match(bridgeSrc, /sendEvent\('trace', \{ phase, \.\.\.extra \}\)/);
  assert.match(bridgeSrc, /fetchLatestBaileysVersion:\s*mod\.fetchLatestBaileysVersion/);
  assert.match(bridgeSrc, /const \{ version \} = await fetchLatestBaileysVersion\(\)/);
  assert.match(bridgeSrc, /emitTrace\('baileys_version', \{ version, connect_attempt: connectAttempt \}\)/);
  assert.match(bridgeSrc, /version,\s*auth:/);
  assert.match(bridgeSrc, /emitTrace\('connection_update'/);
  assert.match(bridgeSrc, /emitTrace\('connection_closed'/);
  assert.match(bridgeSrc, /emitTrace\('qr_generated'/);
  assert.match(bridgeSrc, /QRCode\.toString\(qr,\s*\{\s*type:\s*'utf8'/);
  assert.match(bridgeSrc, /sendEvent\('qr', \{\s*ascii_qr:\s*asciiQr,\s*data_url:\s*dataUrl,/);
  assert.match(bridgeSrc, /browser:\s*Browsers\.ubuntu\('Chrome'\)/);
  assert.match(bridgeSrc, /reconnect_data:\s*reconnectData/);
});
