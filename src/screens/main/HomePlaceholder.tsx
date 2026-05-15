import { useTranslation } from "react-i18next";

export default function HomePlaceholder() {
  const { t } = useTranslation();
  return (
    <div style={{ padding: 32 }}>
      <h1>{t("shell.tab.home")}</h1>
      <p>{t("home.placeholder.hint")}</p>
    </div>
  );
}
