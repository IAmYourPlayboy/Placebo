import "@testing-library/jest-dom/vitest";

// jsdom does not implement matchMedia. ThemeProvider uses it to resolve
// the "auto" choice; supply a minimal stub so render() doesn't throw.
if (typeof window !== "undefined" && typeof window.matchMedia !== "function") {
  window.matchMedia = (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  }) as MediaQueryList;
}
