const { spawn } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const mcpArgs = process.argv.slice(2);

if (!mcpArgs.includes("--browser") && !mcpArgs.includes("--executable-path")) {
  if (process.env.PLAYWRIGHT_MCP_EXECUTABLE_PATH) {
    mcpArgs.push(
      "--executable-path",
      process.env.PLAYWRIGHT_MCP_EXECUTABLE_PATH,
    );
  } else if (process.env.PLAYWRIGHT_MCP_BROWSER) {
    mcpArgs.push("--browser", process.env.PLAYWRIGHT_MCP_BROWSER);
  } else if (process.platform === "win32") {
    mcpArgs.push("--browser", "msedge");
  } else if (process.platform === "linux") {
    const executablePath = findOnPath([
      "chromium",
      "chromium-browser",
      "google-chrome-stable",
      "google-chrome",
    ]);

    if (executablePath) {
      mcpArgs.push("--executable-path", executablePath);
    }
  }
}

const npxArgs = ["-y", "@playwright/mcp@latest", ...mcpArgs];
const command = process.platform === "win32"
  ? process.env.ComSpec || "cmd.exe"
  : "npx";
const args = process.platform === "win32"
  ? ["/d", "/s", "/c", "npx.cmd", ...npxArgs]
  : npxArgs;

const child = spawn(command, args, {
  env: process.env,
  stdio: "inherit",
  windowsHide: true,
});

child.on("error", (error) => {
  console.error(`Failed to start Playwright MCP: ${error.message}`);
  process.exitCode = 1;
});

child.on("exit", (code) => {
  process.exitCode = code ?? 1;
});

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.on(signal, () => child.kill(signal));
}

function findOnPath(names) {
  const directories = (process.env.PATH || "").split(path.delimiter);

  for (const name of names) {
    for (const directory of directories) {
      const candidate = path.join(directory, name);
      if (fs.existsSync(candidate)) {
        return candidate;
      }
    }
  }

  return undefined;
}
