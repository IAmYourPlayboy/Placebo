/**
 * Persistent storage for the auth bearer token.
 *
 * Primary backend: the OS keychain via Tauri commands `secure_get` / `secure_set` /
 * `secure_delete` (Windows Credential Manager / macOS Keychain / Secret Service on Linux).
 *
 * Fallback: `localStorage` — used in two cases:
 *   1. The app is running outside Tauri (browser dev mode at http://localhost:1420 or web build).
 *   2. The keychain backend is unavailable (no Secret Service installed on a headless Linux box,
 *      a permission denial, etc.). Falling through to localStorage means dev never gets stuck.
 *
 * Trade-off: localStorage is plain JS storage scoped per-origin. In a Tauri webview that's still
 * scoped to the app, but on the public web build a token there is XSS-stealable. We accept this
 * for the Tauri-only desktop alpha.
 */

import { invoke } from "@tauri-apps/api/core";

/** Single key used in both backends — namespaced so other apps' keychains don't collide. */
const KEY = "placebo.auth.token";

/** True only when running inside the Tauri webview. */
function inTauri(): boolean {
  return typeof (globalThis as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !== "undefined";
}

export async function saveToken(token: string): Promise<void> {
  if (inTauri()) {
    try {
      await invoke<void>("secure_set", { key: KEY, value: token });
      // Mirror to localStorage so a transient keychain miss doesn't log the user out.
      // Belt-and-suspenders; the keychain is the source of truth on subsequent loads.
      localStorage.setItem(KEY, token);
      return;
    } catch (err) {
      console.warn("[tokenStorage] secure_set failed, falling back to localStorage", err);
    }
  }
  localStorage.setItem(KEY, token);
}

export async function loadToken(): Promise<string | null> {
  if (inTauri()) {
    try {
      const v = await invoke<string | null>("secure_get", { key: KEY });
      if (v) return v;
      // Keychain returned None — fall through to localStorage in case we wrote there earlier
      // (e.g. first run after a previous session that lost keychain access).
    } catch (err) {
      console.warn("[tokenStorage] secure_get failed, falling back to localStorage", err);
    }
  }
  return localStorage.getItem(KEY);
}

export async function clearToken(): Promise<void> {
  if (inTauri()) {
    try {
      await invoke<void>("secure_delete", { key: KEY });
    } catch (err) {
      console.warn("[tokenStorage] secure_delete failed", err);
    }
  }
  localStorage.removeItem(KEY);
}
