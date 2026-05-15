import { useTranslation } from "react-i18next";
import { useParams } from "react-router-dom";

export default function ProfilePlaceholder() {
  const { t } = useTranslation();
  const { username } = useParams();
  return (
    <div style={{ padding: 32 }}>
      <h1>{username ? `@${username}` : t("shell.tab.profile")}</h1>
      <p>{t("profile.placeholder.hint")}</p>
    </div>
  );
}
