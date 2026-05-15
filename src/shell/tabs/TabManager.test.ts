import { describe, it, expect } from "vitest";
import { titleForPath } from "./tabTitles";

const t = (k: string) => k; // identity translator for tests

describe("titleForPath", () => {
  it("returns home title for /home", () => {
    expect(titleForPath("/home", t)).toBe("shell.tab.home");
  });

  it("returns home title for empty/root", () => {
    expect(titleForPath("/", t)).toBe("shell.tab.home");
    expect(titleForPath("", t)).toBe("shell.tab.home");
  });

  it("returns profile title with @username when username present", () => {
    expect(titleForPath("/profile/zara", t)).toBe("@zara");
  });

  it("returns generic profile title when no username", () => {
    expect(titleForPath("/profile", t)).toBe("shell.tab.profile");
  });

  it("strips query and hash", () => {
    expect(titleForPath("/categories?foo=bar#x", t)).toBe("shell.tab.categories");
  });

  it("room title shows short id", () => {
    const out = titleForPath("/room/abc12345-xxxx", t);
    expect(out).toContain("abc123");
  });
});
