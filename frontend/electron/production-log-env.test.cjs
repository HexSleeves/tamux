const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const { createChildLogEnv } = require("./main/log-env.cjs");

const repoRoot = path.join(__dirname, "..");

test("packaged runtime forces rust log env to error", () => {
  const env = createChildLogEnv(
    {
      TAMUX_LOG: "debug",
      AMUX_LOG: "trace",
      TAMUX_TUI_LOG: "info",
      AMUX_GATEWAY_LOG: "warn",
      RUST_LOG: "debug",
      KEEP_ME: "yes",
    },
    { isPackaged: true },
  );

  assert.equal(env.TAMUX_LOG, "error");
  assert.equal(env.AMUX_LOG, "error");
  assert.equal(env.TAMUX_TUI_LOG, "error");
  assert.equal(env.AMUX_GATEWAY_LOG, "error");
  assert.equal(env.RUST_LOG, "error");
  assert.equal(env.KEEP_ME, "yes");
});

test("development runtime preserves existing rust log env", () => {
  const env = createChildLogEnv(
    {
      TAMUX_LOG: "debug",
      AMUX_LOG: "trace",
      RUST_LOG: "info",
    },
    { isPackaged: false },
  );

  assert.equal(env.TAMUX_LOG, "debug");
  assert.equal(env.AMUX_LOG, "trace");
  assert.equal(env.RUST_LOG, "info");
});

test("release workflow exports error-only log env", () => {
  const releaseWorkflow = fs.readFileSync(path.join(repoRoot, "../.github/workflows/release.yml"), "utf8");

  assert.match(releaseWorkflow, /TAMUX_LOG:\s*['"]?error['"]?/);
  assert.match(releaseWorkflow, /AMUX_LOG:\s*['"]?error['"]?/);
  assert.match(releaseWorkflow, /TAMUX_TUI_LOG:\s*['"]?error['"]?/);
  assert.match(releaseWorkflow, /AMUX_GATEWAY_LOG:\s*['"]?error['"]?/);
  assert.match(releaseWorkflow, /RUST_LOG:\s*['"]?error['"]?/);
});

test("release scripts export error-only log env", () => {
  const scriptPaths = [
    path.join(repoRoot, "../scripts/build-production-releases.sh"),
    path.join(repoRoot, "../scripts/build-release.sh"),
    path.join(repoRoot, "../scripts/build-release-wsl.sh"),
    path.join(repoRoot, "../scripts/build-release.ps1"),
    path.join(repoRoot, "../scripts/build-release.bat"),
  ];

  for (const scriptPath of scriptPaths) {
    const source = fs.readFileSync(scriptPath, "utf8");
    assert.match(source, /TAMUX_LOG/);
    assert.match(source, /AMUX_LOG/);
    assert.match(source, /error/);
  }
});