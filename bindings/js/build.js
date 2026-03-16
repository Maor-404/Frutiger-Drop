import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const wasmDir = path.join(__dirname, "wasm");

function run(cmd, args, opts = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, {
      cwd: opts.cwd ?? process.cwd(),
      stdio: "inherit",
      env: { ...process.env, ...(opts.env ?? {}) },
      shell: process.platform === "win32"
    });
    child.on("error", reject);
    child.on("exit", (code) => {
      if (code === 0) resolve();
      else reject(new Error(`${cmd} ${args.join(" ")} failed with code ${code}`));
    });
  });
}

await run("wasm-pack", ["build", "--release", "--mode", "no-install"], { cwd: wasmDir });

