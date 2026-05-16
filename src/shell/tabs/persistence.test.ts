import { describe, it, expect, beforeEach } from "vitest";
import { loadSnapshot, saveSnapshot, type Snapshot } from "./persistence";

beforeEach(() => {
  try { localStorage.clear(); } catch { /* ignore */ }
});

describe("persistence", () => {
  it("loadSnapshot returns null when nothing is stored", async () => {
    const out = await loadSnapshot();
    expect(out).toBeNull();
  });

  it("saveSnapshot then loadSnapshot returns the same payload", async () => {
    const snap: Snapshot = {
      tabs: [
        { id: "a", initialPath: "/home", title: "Главная", currentPath: "/home" },
        { id: "b", initialPath: "/settings", title: "Настройки", currentPath: "/settings" },
      ],
      activeTabId: "b",
    };
    await saveSnapshot(snap);
    const restored = await loadSnapshot();
    expect(restored).toEqual(snap);
  });

  it("loadSnapshot returns null for a snapshot with empty tabs", async () => {
    await saveSnapshot({ tabs: [], activeTabId: "" });
    const restored = await loadSnapshot();
    expect(restored).toBeNull();
  });

  it("loadSnapshot returns null for malformed JSON", async () => {
    localStorage.setItem("tabs.snapshot.v1", "not-json{");
    const out = await loadSnapshot();
    expect(out).toBeNull();
  });
});
