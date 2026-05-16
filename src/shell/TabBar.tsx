import { useTranslation } from "react-i18next";
import { useTabs } from "./tabs/useTabs";
import Tab from "./Tab";
import { Icon } from "../components/ui/Icon";

export default function TabBar() {
  const { tabs, openTab } = useTabs();
  const { t } = useTranslation();
  return (
    <div className="shell-tabbar" role="tablist">
      {tabs.map((tab) => <Tab key={tab.id} tab={tab} />)}
      <button
        className="shell-tabbar__new"
        onClick={() => openTab("/home")}
        aria-label={t("tabbar.new")}
      >
        <Icon name="PlusSmallIcon" size={16} />
      </button>
    </div>
  );
}
