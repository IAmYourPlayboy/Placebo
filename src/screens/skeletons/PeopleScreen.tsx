import { useTranslation } from "react-i18next";
import EmptySection from "./EmptySection";

export default function PeopleScreen() {
  const { t } = useTranslation();
  return <EmptySection title={t("people.empty.title")} hint={t("people.empty.hint")} />;
}
