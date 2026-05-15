import { useContext } from "react";
import { TabContext } from "./TabManager";

export function useTabs() {
  const ctx = useContext(TabContext);
  if (!ctx) throw new Error("useTabs must be used within <TabManager>");
  return ctx;
}
