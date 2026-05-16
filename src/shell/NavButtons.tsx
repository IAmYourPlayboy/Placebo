import { useTranslation } from "react-i18next";
import { useTabs } from "./tabs/useTabs";
import { Icon } from "../components/ui/Icon";

export default function NavButtons() {
  const { goBack, goForward, reload } = useTabs();
  const { t } = useTranslation();
  return (
    <div className="nav-buttons">
      <button aria-label={t("topbar.back")} onClick={goBack}>
        <Icon name="ArrowLeftIcon" size={18} />
      </button>
      <button aria-label={t("topbar.forward")} onClick={goForward}>
        <Icon name="ArrowRightIcon" size={18} />
      </button>
      <button aria-label={t("topbar.reload")} onClick={reload}>
        <Icon name="RefreshIcon" size={18} />
      </button>
    </div>
  );
}
