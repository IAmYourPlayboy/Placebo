import { useTabs } from "./tabs/useTabs";

export default function Logo() {
  const { openTab, tabs, activateTab } = useTabs();

  const go = () => {
    const home = tabs.find((t) => t.initialPath === "/home");
    if (home) activateTab(home.id);
    else openTab("/home");
  };

  return (
    <button className="shell-logo" onClick={go}>
      <span className="shell-logo__text">Placebo</span>
    </button>
  );
}
