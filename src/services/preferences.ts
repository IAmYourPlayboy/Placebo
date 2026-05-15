import { invoke } from "@tauri-apps/api/core";

export async function prefsGet(key: string): Promise<string | null> {
  return await invoke<string | null>("prefs_get", { key });
}

export async function prefsSet(key: string, value: string): Promise<void> {
  await invoke<void>("prefs_set", { key, value });
}

export async function prefsAll(): Promise<Array<{ key: string; value: string }>> {
  return await invoke<Array<{ key: string; value: string }>>("prefs_all");
}
