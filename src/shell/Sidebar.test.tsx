import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, beforeAll, beforeEach, vi } from "vitest";
import { ThemeProvider } from "../theme";
import { AuthProvider } from "../auth/AuthProvider";
import { TabManager } from "./tabs/TabManager";
import { Scene3DRegistry } from "./scene3d/Scene3DRegistry";
import ShellRoot from "./ShellRoot";
import i18n from "../i18n";

// Stub the token store so AuthProvider doesn't try to invoke Tauri commands in jsdom,
// and stub /me so the bootstrap settles into "anonymous" deterministically. The tab
// titles we assert on come from the title registry, not the rendered screens, so the
// AuthGuard redirect inside each tab is fine — Sidebar still drives tab open/activate
// the same way.
vi.mock("../auth/tokenStorage", () => ({
  loadToken: vi.fn(async () => null),
  saveToken: vi.fn(async () => {}),
  clearToken: vi.fn(async () => {}),
}));
vi.mock("../api/auth", () => ({
  me: vi.fn(async () => {
    throw new Error("unreachable in this test");
  }),
  register: vi.fn(),
  login: vi.fn(),
  logout: vi.fn(),
  refresh: vi.fn(),
}));

beforeAll(async () => {
  // jsdom navigator language is en-US, but the assertions expect ru labels.
  await i18n.changeLanguage("ru");
});

beforeEach(() => {
  try { localStorage.clear(); } catch { /* ignore */ }
});

function mount() {
  return render(
    <ThemeProvider>
      <AuthProvider>
        <Scene3DRegistry>
          <TabManager initialPath="/home">
            <ShellRoot />
          </TabManager>
        </Scene3DRegistry>
      </AuthProvider>
    </ThemeProvider>,
  );
}

/** Find a sidebar item button by visible label (scopes to <aside>). */
function sidebarItem(label: string) {
  const aside = screen.getByRole("complementary", { hidden: true });
  // <aside> is implicitly role="complementary".
  return within(aside).getByRole("button", { name: label });
}

/** Count tabs in the tablist by their visible title. */
function tabsByTitle(title: string) {
  const tablist = screen.getByRole("tablist");
  return within(tablist).queryAllByRole("tab", { name: new RegExp(title) });
}

describe("Sidebar -> tab opening", () => {
  it("starts with a single Home tab", () => {
    mount();
    expect(tabsByTitle("Главная")).toHaveLength(1);
  });

  it("opens a Settings tab when the sidebar item is clicked", async () => {
    const user = userEvent.setup();
    mount();
    await user.click(sidebarItem("Настройки"));
    expect(tabsByTitle("Настройки")).toHaveLength(1);
    expect(tabsByTitle("Главная")).toHaveLength(1);
  });

  it("activates an existing tab instead of creating a duplicate", async () => {
    const user = userEvent.setup();
    mount();
    await user.click(sidebarItem("Настройки"));
    await user.click(sidebarItem("Главная"));
    await user.click(sidebarItem("Настройки"));
    // Still exactly one Settings tab and one Home tab.
    expect(tabsByTitle("Настройки")).toHaveLength(1);
    expect(tabsByTitle("Главная")).toHaveLength(1);
  });
});
