/**
 * Welcome screen — the unauthenticated landing page.
 *
 * All flows but "Зарегистрироваться" / "Войти" are intentionally disabled in alpha:
 *   - "Попробовать без аккаунта" (no anonymous mode in alpha — every interaction needs auth)
 *   - 8 social-login buttons (M2 ships email/password only; OAuth follows in M7+)
 */

import { useTranslation } from "react-i18next";
import { Link } from "react-router-dom";
import "./auth.css";

// Order matches the Figma layout. Labels are short identifiers used as both the visible
// initial inside the round button and an aria-label for screen readers.
const SOCIAL_PROVIDERS = [
  { id: "google", label: "G", name: "Google" },
  { id: "facebook", label: "f", name: "Facebook" },
  { id: "apple", label: "⌘", name: "Apple" },
  { id: "vk", label: "VK", name: "VK" },
  { id: "telegram", label: "✈", name: "Telegram" },
  { id: "discord", label: "D", name: "Discord" },
  { id: "x", label: "X", name: "X" },
  { id: "phone", label: "☎", name: "Phone" },
] as const;

export default function WelcomeScreen() {
  const { t, i18n } = useTranslation();

  const toggleLang = () => {
    const next = i18n.resolvedLanguage === "ru" ? "en" : "ru";
    void i18n.changeLanguage(next);
  };

  return (
    <div className="auth-welcome">
      <div className="auth-welcome__center">
        <h1 className="auth-welcome__title">Placebo</h1>
        <p className="auth-welcome__subtitle">{t("auth.welcome.subtitle")}</p>

        <div className="auth-welcome__actions">
          <Link to="/register" className="auth-btn auth-btn--primary">
            {t("auth.welcome.register")}
          </Link>
          <Link to="/login" className="auth-btn">
            {t("auth.welcome.login")}
          </Link>
          <button className="auth-btn" disabled title={t("auth.welcome.social.soon")}>
            {t("auth.welcome.try_as_guest")}
          </button>
        </div>

        <p className="auth-welcome__or">{t("auth.welcome.or_via")}</p>
        <div className="auth-welcome__social">
          {SOCIAL_PROVIDERS.map((p) => (
            <button
              key={p.id}
              className="auth-social"
              disabled
              title={`${p.name} — ${t("auth.welcome.social.soon")}`}
              aria-label={p.name}
            >
              {p.label}
            </button>
          ))}
        </div>

        <div className="auth-welcome__footer">
          <button className="auth-lang" onClick={toggleLang}>
            Your language is{" "}
            <b>{i18n.resolvedLanguage === "ru" ? "Russian" : "English"}</b>
          </button>
        </div>
      </div>
    </div>
  );
}
