import { useTranslation } from "react-i18next";
import EmptySection from "./EmptySection";

export default function FavoritesScreen() {
  const { t } = useTranslation();
  return <EmptySection title={t("favorites.empty.title")} hint={t("favorites.empty.hint")} />;
}
