import { createContext, useEffect, useMemo, useState, ReactNode } from "react";
import type { ThemeChoice, ResolvedTheme } from "./index";
import { DEFAULT_THEME, THEME_STORAGE_KEY } from "./index";

type ThemeContextValue = {
  choice: ThemeChoice;
  resolved: ResolvedTheme;
  setChoice: (c: ThemeChoice) => void;
};

export const ThemeContext = createContext<ThemeContextValue | null>(null);

function resolveChoice(choice: ThemeChoice): ResolvedTheme {
  if (choice === "auto") {
    const prefersDark = typeof window !== "undefined"
      && window.matchMedia("(prefers-color-scheme: dark)").matches;
    return prefersDark ? "dark" : "light";
  }
  return choice;
}

function loadChoice(): ThemeChoice {
  if (typeof window === "undefined") return DEFAULT_THEME;
  const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
  if (stored === "light" || stored === "dark" || stored === "auto") return stored;
  return DEFAULT_THEME;
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [choice, setChoiceState] = useState<ThemeChoice>(() => loadChoice());
  const [resolved, setResolved] = useState<ResolvedTheme>(() => resolveChoice(choice));

  useEffect(() => {
    setResolved(resolveChoice(choice));
  }, [choice]);

  // Respond to system theme changes while in "auto"
  useEffect(() => {
    if (choice !== "auto") return;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => setResolved(mq.matches ? "dark" : "light");
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, [choice]);

  // Sync to <html data-theme=...>
  useEffect(() => {
    document.documentElement.setAttribute("data-theme", resolved);
  }, [resolved]);

  const setChoice = (c: ThemeChoice) => {
    setChoiceState(c);
    window.localStorage.setItem(THEME_STORAGE_KEY, c);
  };

  const value = useMemo(() => ({ choice, resolved, setChoice }), [choice, resolved]);
  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}
