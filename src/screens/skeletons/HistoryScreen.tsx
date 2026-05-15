import { useTranslation } from "react-i18next";
import EmptySection from "./EmptySection";

export default function HistoryScreen() {
  const { t } = useTranslation();
  return <EmptySection title={t("history.empty.title")} hint={t("history.empty.hint")} />;
}
