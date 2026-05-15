export type ThemeChoice = "light" | "dark" | "auto";
export type ResolvedTheme = "light" | "dark";

export const THEME_STORAGE_KEY = "placebo.theme";
export const DEFAULT_THEME: ThemeChoice = "auto";

export { ThemeProvider } from "./ThemeProvider";
export { useTheme } from "./useTheme";
