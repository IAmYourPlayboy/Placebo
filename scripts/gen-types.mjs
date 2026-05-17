#!/usr/bin/env node
/**
 * Cross-platform replacement for `scripts/gen-types.sh`.
 *
 * Why a Node script: the bash version was the source of truth for everyone on macOS / Linux
 * and for the original Windows author who runs Git Bash. But `npm run tauri dev` invokes the
 * `predev` hook with whatever shell `npm` chooses, and on Windows from PowerShell that ends
 * up calling /bin/bash via WSL — which fails on a machine that doesn't have a WSL bash on
 * the PATH. A Node script doesn't care about the parent shell.
 *
 * Behaviour mirrors gen-types.sh:
 *   1. Run `cargo test -p placebo-shared --features export-types export_bindings` to write
 *      ts-rs bindings into crates/placebo-shared/bindings/.
 *   2. Mirror the bindings into src/types/api/, removing stale .ts files first but keeping
 *      README.md (which is checked into git).
 *
 * cargo location: prefer ~/.cargo/bin/cargo(.exe) (rustup install path), fall back to PATH.
 */

import { spawnSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  rmSync,
  statSync,
} from "node:fs";
import { homedir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "..");
const SRC = join(ROOT, "crates", "placebo-shared", "bindings");
const DST = join(ROOT, "src", "types", "api");

function findCargo() {
  const home = homedir();
  const candidates = [
    join(home, ".cargo", "bin", "cargo.exe"),
    join(home, ".cargo", "bin", "cargo"),
  ];
  for (const candidate of candidates) {
    if (existsSync(candidate)) return candidate;
  }
  // Fall back to whatever the parent process's PATH resolves.
  return process.platform === "win32" ? "cargo.exe" : "cargo";
}

function run(cmd, args, opts = {}) {
  const result = spawnSync(cmd, args, {
    cwd: ROOT,
    stdio: "inherit",
    shell: false,
    ...opts,
  });
  if (result.error) throw result.error;
  return result.status ?? 1;
}

function wipeStaleTs(dir) {
  if (!existsSync(dir)) return;
  for (const entry of readdirSync(dir)) {
    if (entry === "README.md") continue;
    const full = join(dir, entry);
    if (statSync(full).isFile() && entry.endsWith(".ts")) {
      rmSync(full);
    }
  }
}

function main() {
  const cargo = findCargo();

  console.log("[gen-types] running cargo test export_bindings ...");
  // Mirror the bash script's intentional `|| true`: we proceed to the copy step even if the
  // export tests bail (e.g. the user has a partially-broken Rust workspace) so the JS side
  // at least keeps the previous bindings rather than ending up with an empty src/types/api.
  run(cargo, [
    "test",
    "-p",
    "placebo-shared",
    "--features",
    "export-types",
    "export_bindings",
    "--",
    "--nocapture",
  ]);

  if (existsSync(SRC)) {
    console.log(`[gen-types] copying bindings -> ${DST}`);
    mkdirSync(DST, { recursive: true });
    wipeStaleTs(DST);
    // Copy each .ts file from SRC into DST. recursive: true handles nested directories
    // if ts-rs ever starts emitting them.
    cpSync(SRC, DST, { recursive: true });
  } else {
    console.log("[gen-types] no bindings/ yet, nothing to copy (OK on first run).");
  }

  console.log("[gen-types] done");
}

main();
