import { RouterProvider } from "react-router-dom";
import { useTabs } from "./tabs/useTabs";

export default function TabContent() {
  const { tabs, activeTabId } = useTabs();
  return (
    <main className="shell-content">
      {tabs.map((tab) => {
        const isActive = tab.id === activeTabId;
        return (
          <div
            key={tab.id}
            className={"shell-content__tab" + (isActive ? " shell-content__tab--active" : "")}
            hidden={!isActive}
          >
            <RouterProvider router={tab.router} />
          </div>
        );
      })}
    </main>
  );
}
