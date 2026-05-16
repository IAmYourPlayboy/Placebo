import { prefsGet, prefsSet } from "../../services/preferences";

const KEY = "tabs.snapshot.v1";

export type Snapshot = {
  tabs: Array<{
    id: string;
    initialPath: string;
    title: string;
    currentPath: string;
  }>;
  activeTabId: string;
};

/**
 * Read tabs snapshot from Tauri user_preferences. In a non-Tauri dev run
 * (vite alone) prefsGet throws; we fall back to localStorage so tab state
 * still survives a refresh during local frontend development.
 */
export async function loadSnapshot(): Promise<Snapshot | null> {
  let raw: string | null = null;
  try {
    raw = await prefsGet(KEY);
  } catch {
    try {
      raw = localStorage.getItem(KEY);
    } catch {
      return null;
    }
  }
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw) as Snapshot;
    if (!parsed.tabs?.length) return null;
    return parsed;
  } catch {
    return null;
  }
}

export async function saveSnapshot(s: Snapshot): Promise<void> {
  const raw = JSON.stringify(s);
  try {
    await prefsSet(KEY, raw);
    return;
  } catch {
    /* fall through to localStorage */
  }
  try {
    localStorage.setItem(KEY, raw);
  } catch { /* ignore */ }
}
