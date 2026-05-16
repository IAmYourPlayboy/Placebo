import { useTranslation } from "react-i18next";
import EmptySection from "./EmptySection";

export default function NotificationsScreen() {
  const { t } = useTranslation();
  return <EmptySection title={t("notifications.empty.title")} hint={t("notifications.empty.hint")} />;
}
