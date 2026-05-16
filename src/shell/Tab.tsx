import clsx from "clsx";
import { useTabs } from "./tabs/useTabs";
import { Icon } from "../components/ui/Icon";
import type { Tab as TabData } from "./tabs/types";

export default function Tab({ tab }: { tab: TabData }) {
  const { activeTabId, activateTab, closeTab } = useTabs();
  const isActive = tab.id === activeTabId;

  const onMouseDown = (e: React.MouseEvent) => {
    if (e.button === 1) {
      e.preventDefault();
      closeTab(tab.id);
    }
  };

  return (
    <div
      className={clsx("shell-tab", isActive && "shell-tab--active")}
      onClick={() => activateTab(tab.id)}
      onMouseDown={onMouseDown}
      role="tab"
      aria-selected={isActive}
    >
      <span className="shell-tab__title" title={tab.title}>{tab.title}</span>
      <button
        className="shell-tab__close"
        onClick={(e) => { e.stopPropagation(); closeTab(tab.id); }}
        aria-label="close tab"
      >
        <Icon name="CloseIcon" size={14} />
      </button>
    </div>
  );
}
