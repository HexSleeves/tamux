const test = require("node:test");
const assert = require("node:assert/strict");
const path = require("node:path");
const { spawn } = require("node:child_process");

function runBridgeUntilReady(timeoutMs = 8000) {
  return new Promise((resolve) => {
    const electronBin = path.join(__dirname, "..", "node_modules", ".bin", "electron");
    const bridgePath = path.join(__dirname, "whatsapp-bridge.cjs");
    const child = spawn(electronBin, [bridgePath], {
      env: { ...process.env, ELECTRON_RUN_AS_NODE: "1" },
      stdio: ["ignore", "pipe", "pipe"],
    });

    let stdout = "";
    let stderr = "";
    let done = false;

    const finish = (result) => {
      if (done) return;
      done = true;
      try {
        child.kill("SIGTERM");
      } catch {}
      resolve(result);
    };

    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString("utf8");
      if (stdout.includes('"event":"ready"')) {
        finish({ stdout, stderr, code: null, ready: true });
      }
    });

    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString("utf8");
    });

    child.on("close", (code) => {
      finish({ stdout, stderr, code, ready: false });
    });

    setTimeout(() => {
      finish({ stdout, stderr, code: null, ready: stdout.includes('"event":"ready"') });
    }, timeoutMs);
  });
}

test("whatsapp bridge starts under Electron node mode", async () => {
  const result = await runBridgeUntilReady();
  assert.equal(result.ready, true, `bridge did not emit ready event\nstderr:\n${result.stderr}`);
  assert.ok(
    !result.stderr.includes("ERR_REQUIRE_ESM"),
    `bridge failed ESM import compatibility\nstderr:\n${result.stderr}`,
  );
});
