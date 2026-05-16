import { useTranslation } from "react-i18next";
import NavButtons from "./NavButtons";
import ThemeToggle from "./ThemeToggle";
import { Icon } from "../components/ui/Icon";

export default function TopBar() {
  const { t } = useTranslation();
  return (
    <div className="shell-topbar">
      <NavButtons />
      <label className="shell-topbar__search">
        <Icon name="SearchIcon" size={16} />
        <input
          type="text"
          placeholder={t("topbar.search.placeholder")}
          disabled
          aria-disabled="true"
        />
      </label>
      <ThemeToggle />
    </div>
  );
}
