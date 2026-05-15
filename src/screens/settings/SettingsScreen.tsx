import { useTranslation } from "react-i18next";
import { useTheme } from "../../theme/useTheme";
import type { ThemeChoice } from "../../theme";

const CHOICES: ThemeChoice[] = ["light", "auto", "dark"];

export default function SettingsScreen() {
  const { t, i18n } = useTranslation();
  const { choice, setChoice } = useTheme();

  return (
    <div className="settings">
      <h1>{t("settings.title")}</h1>

      <section className="settings__group">
        <h2>{t("settings.theme.title")}</h2>
        <div className="settings__row">
          {CHOICES.map((c) => (
            <button
              key={c}
              className={"settings__chip" + (choice === c ? " settings__chip--active" : "")}
              onClick={() => setChoice(c)}
            >
              {t(`settings.theme.${c}`)}
            </button>
          ))}
        </div>
      </section>

      <section className="settings__group">
        <h2>{t("settings.language.title")}</h2>
        <div className="settings__row">
          {["ru", "en"].map((lng) => (
            <button
              key={lng}
              className={"settings__chip" + (i18n.resolvedLanguage === lng ? " settings__chip--active" : "")}
              onClick={() => i18n.changeLanguage(lng)}
            >
              {t(`settings.language.${lng}`)}
            </button>
          ))}
        </div>
      </section>

      <section className="settings__group">
        <h2>{t("settings.account.title")}</h2>
        <button className="settings__danger" disabled>
          {t("settings.account.logout")}
        </button>
        <p className="settings__hint">{t("settings.account.logout.hint")}</p>
      </section>
    </div>
  );
}
