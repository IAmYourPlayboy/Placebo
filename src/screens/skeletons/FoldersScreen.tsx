import { useTranslation } from "react-i18next";
import EmptySection from "./EmptySection";

export default function FoldersScreen() {
  const { t } = useTranslation();
  return <EmptySection title={t("folders.empty.title")} hint={t("folders.empty.hint")} />;
}
