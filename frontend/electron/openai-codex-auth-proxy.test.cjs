const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const mainPath = path.join(__dirname, "main.cjs");
const preloadPath = path.join(__dirname, "preload.cjs");
const mainSrc = fs.readFileSync(mainPath, "utf8");
const preloadSrc = fs.readFileSync(preloadPath, "utf8");

test("main removes local sqlite codex auth ownership", () => {
  assert.doesNotMatch(mainSrc, /require\(['"]node:sqlite['"]\)/);
  assert.doesNotMatch(mainSrc, /DatabaseSync/);
  assert.doesNotMatch(mainSrc, /function getProviderAuthDbPath\(/);
  assert.doesNotMatch(mainSrc, /function withProviderAuthDb\(/);
  assert.doesNotMatch(mainSrc, /function readStoredOpenAICodexAuth\(/);
  assert.doesNotMatch(mainSrc, /function writeStoredOpenAICodexAuth\(/);
  assert.doesNotMatch(mainSrc, /function deleteStoredOpenAICodexAuth\(/);
  assert.doesNotMatch(mainSrc, /function importCodexCliAuthIfPresent\(/);
  assert.doesNotMatch(mainSrc, /async function refreshOpenAICodexAuth\(/);
  assert.doesNotMatch(mainSrc, /async function getOpenAICodexAuthStatus\(/);
  assert.doesNotMatch(mainSrc, /async function exchangeOpenAICodexAuthorizationCode\(/);
  assert.doesNotMatch(mainSrc, /async function loginOpenAICodexInteractive\(/);
});

test("preload keeps desktop codex auth API names stable", () => {
  assert.match(preloadSrc, /openAICodexAuthStatus:\s*\(options\)\s*=>\s*ipcRenderer\.invoke\('openai-codex-auth-status', options\)/);
  assert.match(preloadSrc, /openAICodexAuthLogin:\s*\(\)\s*=>\s*ipcRenderer\.invoke\('openai-codex-auth-login'\)/);
  assert.match(preloadSrc, /openAICodexAuthLogout:\s*\(\)\s*=>\s*ipcRenderer\.invoke\('openai-codex-auth-logout'\)/);
});

test("main proxies desktop codex auth IPC handlers through daemon bridge queries", () => {
  assert.match(
    mainSrc,
    /ipcMain\.handle\('openai-codex-auth-status', async \(_event, options\) => \{[\s\S]*?sendAgentQuery\([\s\S]*?type:\s*'openai-codex-auth-status',[\s\S]*?'openai-codex-auth-status',[\s\S]*?30000/
  );
  assert.match(
    mainSrc,
    /ipcMain\.handle\('openai-codex-auth-login', async \(\) => \{[\s\S]*?sendAgentQuery\([\s\S]*?type:\s*'openai-codex-auth-login'[\s\S]*?'openai-codex-auth-login-result',[\s\S]*?30000/
  );
  assert.match(
    mainSrc,
    /ipcMain\.handle\('openai-codex-auth-logout', async \(\) => \{[\s\S]*?sendAgentQuery\([\s\S]*?type:\s*'openai-codex-auth-logout'[\s\S]*?'openai-codex-auth-logout-result',[\s\S]*?30000/
  );
});

test("main resolves daemon codex auth bridge response types", () => {
  assert.match(mainSrc, /'openai-codex-auth-status'/);
  assert.match(mainSrc, /'openai-codex-auth-login-result'/);
  assert.match(mainSrc, /'openai-codex-auth-logout-result'/);
  assert.match(
    mainSrc,
    /\[([\s\S]*?)'openai-codex-auth-status'([\s\S]*?)'openai-codex-auth-login-result'([\s\S]*?)'openai-codex-auth-logout-result'([\s\S]*?)\]\.includes\(event\.type\)/
  );
});
