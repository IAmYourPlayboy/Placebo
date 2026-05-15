import { renderHook, act } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { ReactNode } from "react";
import { TabManager } from "./TabManager";
import { useTabs } from "./useTabs";
import "../../i18n"; // ensures i18n is initialized for titleForPath

function wrapper({ children }: { children: ReactNode }) {
  return <TabManager initialPath="/home">{children}</TabManager>;
}

describe("TabManager", () => {
  it("initial state: 1 tab, active is the first, initialPath is /home", () => {
    const { result } = renderHook(() => useTabs(), { wrapper });
    expect(result.current.tabs).toHaveLength(1);
    expect(result.current.activeTabId).toBe(result.current.tabs[0].id);
    expect(result.current.tabs[0].initialPath).toBe("/home");
  });

  it("openTab adds a tab and makes it active", () => {
    const { result } = renderHook(() => useTabs(), { wrapper });
    let newId: string | undefined;
    act(() => {
      newId = result.current.openTab("/categories");
    });
    expect(result.current.tabs).toHaveLength(2);
    expect(result.current.activeTabId).toBe(newId);
    expect(result.current.tabs[1].id).toBe(newId);
  });

  it("activateTab changes activeTabId without changing tab count", () => {
    const { result } = renderHook(() => useTabs(), { wrapper });
    const firstId = result.current.tabs[0].id;
    let secondId: string | undefined;
    act(() => {
      secondId = result.current.openTab("/categories");
    });
    act(() => {
      result.current.activateTab(firstId);
    });
    expect(result.current.tabs).toHaveLength(2);
    expect(result.current.activeTabId).toBe(firstId);
    expect(secondId).not.toBe(firstId);
  });

  it("closeTab on non-active tab: tab removed, activeTabId unchanged", async () => {
    const { result } = renderHook(() => useTabs(), { wrapper });
    const firstId = result.current.tabs[0].id;
    let secondId: string | undefined;
    act(() => {
      secondId = result.current.openTab("/categories");
    });
    // Switch active back to first; close the second (non-active).
    act(() => {
      result.current.activateTab(firstId);
    });
    expect(result.current.activeTabId).toBe(firstId);
    await act(async () => {
      result.current.closeTab(secondId!);
    });
    expect(result.current.tabs).toHaveLength(1);
    expect(result.current.activeTabId).toBe(firstId);
  });

  it("closeTab on active tab with 2+ tabs: neighbor activated", async () => {
    const { result } = renderHook(() => useTabs(), { wrapper });
    const firstId = result.current.tabs[0].id;
    let secondId: string | undefined;
    act(() => {
      secondId = result.current.openTab("/categories");
    });
    // Second is now active; closing it should activate the first (neighbor).
    expect(result.current.activeTabId).toBe(secondId);
    await act(async () => {
      result.current.closeTab(secondId!);
    });
    expect(result.current.tabs).toHaveLength(1);
    expect(result.current.activeTabId).toBe(firstId);
  });

  it("closeTab on last tab: count stays 1, new tab is fresh /home", async () => {
    const { result } = renderHook(() => useTabs(), { wrapper });
    const firstId = result.current.tabs[0].id;
    await act(async () => {
      result.current.closeTab(firstId);
    });
    expect(result.current.tabs).toHaveLength(1);
    expect(result.current.tabs[0].id).not.toBe(firstId);
    expect(result.current.tabs[0].initialPath).toBe("/home");
    expect(result.current.activeTabId).toBe(result.current.tabs[0].id);
  });

  it("renameTab updates title without changing list shape", () => {
    const { result } = renderHook(() => useTabs(), { wrapper });
    const firstId = result.current.tabs[0].id;
    act(() => {
      result.current.renameTab(firstId, "My Custom Title");
    });
    expect(result.current.tabs).toHaveLength(1);
    expect(result.current.tabs[0].id).toBe(firstId);
    expect(result.current.tabs[0].title).toBe("My Custom Title");
  });
});
