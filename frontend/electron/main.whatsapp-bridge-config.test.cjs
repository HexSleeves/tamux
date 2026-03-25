const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

test("main process starts WhatsApp bridge in node mode", () => {
  const mainPath = path.join(__dirname, "main.cjs");
  const src = fs.readFileSync(mainPath, "utf8");
  const hasRunAsNodeFlag = /ELECTRON_RUN_AS_NODE\s*:\s*['"]1['"]/.test(src);
  assert.equal(
    hasRunAsNodeFlag,
    true,
    "startWhatsAppBridge must set ELECTRON_RUN_AS_NODE=1 so bridge does not boot Electron UI/GPU process",
  );
});
