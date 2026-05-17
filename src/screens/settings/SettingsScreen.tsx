import { useTranslation } from "react-i18next";
import { useAuth } from "../../auth/useAuth";
import { useTheme } from "../../theme/useTheme";
import type { ThemeChoice } from "../../theme";

const CHOICES: ThemeChoice[] = ["light", "auto", "dark"];
const LANGS = ["ru", "en"] as const;

export default function SettingsScreen() {
  const { t, i18n } = useTranslation();
  const { choice, setChoice } = useTheme();
  const { status, logout } = useAuth();
  const currentLang = i18n.resolvedLanguage ?? i18n.language ?? "";
  const isAuthed = status === "authenticated";

  return (
    <div className="settings">
      <h1>{t("settings.title")}</h1>

      <section className="settings__group">
        <h2>{t("settings.theme.title")}</h2>
        <div className="settings__row">
          {CHOICES.map((c) => {
            const active = choice === c;
            return (
              <button
                key={c}
                className={"settings__chip" + (active ? " settings__chip--active" : "")}
                onClick={() => setChoice(c)}
                aria-pressed={active}
              >
                {t(`settings.theme.${c}`)}
              </button>
            );
          })}
        </div>
      </section>

      <section className="settings__group">
        <h2>{t("settings.language.title")}</h2>
        <div className="settings__row">
          {LANGS.map((lng) => {
            const active = currentLang === lng || currentLang.startsWith(`${lng}-`);
            return (
              <button
                key={lng}
                className={"settings__chip" + (active ? " settings__chip--active" : "")}
                onClick={() => i18n.changeLanguage(lng)}
                aria-pressed={active}
              >
                {t(`settings.language.${lng}`)}
              </button>
            );
          })}
        </div>
      </section>

      <section className="settings__group">
        <h2>{t("settings.account.title")}</h2>
        <button
          className="settings__danger"
          disabled={!isAuthed}
          onClick={isAuthed ? () => void logout() : undefined}
        >
          {t("settings.account.logout")}
        </button>
        {!isAuthed && (
          <p className="settings__hint">{t("settings.account.logout.hint")}</p>
        )}
      </section>
    </div>
  );
}
