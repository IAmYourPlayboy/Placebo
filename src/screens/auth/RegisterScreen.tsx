/**
 * Registration form. Two cards: required (name, email, password) and optional (DOB, username,
 * DOB-hidden flag). On submit we POST to /auth/register; the server's 409 with suggestions
 * is rendered as a row of clickable chips that overwrite the username field.
 */

import { FormEvent, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { ApiError, isApiError } from "../../api/errors";
import { useAuth } from "../../auth/useAuth";
import type { RegisterRequest } from "../../types/api/RegisterRequest";
import "./auth.css";

/** Pad a 1-2 digit number to 2 digits ("5" → "05"). */
function pad2(s: string): string {
  return s.length === 1 ? `0${s}` : s;
}

/** Build the YYYY-MM-DD string the server expects, or null if any field is empty. */
function composeDob(day: string, month: string, year: string): string | null {
  if (!day || !month || !year) return null;
  return `${year.padStart(4, "0")}-${pad2(month)}-${pad2(day)}`;
}

export default function RegisterScreen() {
  const { t, i18n } = useTranslation();
  const navigate = useNavigate();
  const { register } = useAuth();

  const [firstName, setFirstName] = useState("");
  const [lastName, setLastName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [username, setUsername] = useState("");
  const [day, setDay] = useState("");
  const [month, setMonth] = useState("");
  const [year, setYear] = useState("");
  const [dobHidden, setDobHidden] = useState(true);

  const [submitting, setSubmitting] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [suggestions, setSuggestions] = useState<string[]>([]);

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    setErrors({});
    setSuggestions([]);

    // Client-side guard rails. Detailed validation lives on the server (it owns the truth);
    // we only catch the obvious empty/short cases so the user gets immediate feedback.
    const fieldErrors: Record<string, string> = {};
    if (!firstName.trim()) {
      fieldErrors.firstName = t("auth.register.error.first_name_required");
    }
    if (!email.trim()) {
      fieldErrors.email = t("auth.register.error.email_required");
    }
    if (password.length < 8) {
      fieldErrors.password = t("auth.register.error.password_short");
    }
    if (Object.keys(fieldErrors).length > 0) {
      setErrors(fieldErrors);
      return;
    }

    const displayName = lastName.trim()
      ? `${firstName.trim()} ${lastName.trim()}`
      : firstName.trim();

    // Strip a leading @ if the user typed one — we store and compare bare usernames.
    const cleanedUsername = username.replace(/^@/, "").trim();

    const payload: RegisterRequest = {
      email: email.trim(),
      password,
      displayName,
      username: cleanedUsername || null,
      dateOfBirth: composeDob(day, month, year),
      dateOfBirthHidden: dobHidden,
      locale: i18n.resolvedLanguage ?? null,
    };

    setSubmitting(true);
    try {
      await register(payload);
      navigate("/home", { replace: true });
    } catch (err) {
      if (isApiError(err) && err.code === "USERNAME_TAKEN") {
        const sug = Array.isArray(err.extra?.suggestions)
          ? (err.extra!.suggestions as string[])
          : [];
        setSuggestions(sug);
        setErrors({ username: t("auth.register.error.username_taken") });
      } else if (err instanceof ApiError) {
        // Server returned a structured error we don't have a special UI for — show its message.
        setErrors({ _: err.message });
      } else {
        setErrors({ _: t("auth.error.generic") });
      }
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form className="auth-form" onSubmit={submit} noValidate>
      <button type="button" className="auth-back" onClick={() => navigate("/welcome")}>
        {"< "}
        {t("auth.register.back")}
      </button>
      <h1 className="auth-form__title">{t("auth.register.title")}</h1>

      <div className="auth-form__grid">
        <section className="auth-form__section">
          <div className="auth-form__section-title">{t("auth.register.required")}</div>

          <div className="auth-field">
            <label htmlFor="reg-first">{t("auth.register.name")}</label>
            <input
              id="reg-first"
              type="text"
              value={firstName}
              onChange={(e) => setFirstName(e.target.value)}
              placeholder={t("auth.register.name.first")}
              autoComplete="given-name"
            />
            <input
              type="text"
              value={lastName}
              onChange={(e) => setLastName(e.target.value)}
              placeholder={t("auth.register.name.last")}
              autoComplete="family-name"
            />
            {errors.firstName && <div className="auth-field__error">{errors.firstName}</div>}
          </div>

          <div className="auth-field">
            <label htmlFor="reg-email">{t("auth.register.email")}</label>
            <input
              id="reg-email"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder={t("auth.register.email.placeholder")}
              autoComplete="email"
            />
            {errors.email && <div className="auth-field__error">{errors.email}</div>}
          </div>

          <div className="auth-field">
            <label htmlFor="reg-password">{t("auth.register.password")}</label>
            <input
              id="reg-password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder={t("auth.register.password.placeholder")}
              autoComplete="new-password"
            />
            {errors.password && <div className="auth-field__error">{errors.password}</div>}
          </div>
        </section>

        <section className="auth-form__section">
          <div className="auth-form__section-title">{t("auth.register.optional")}</div>

          <div className="auth-field">
            <label>{t("auth.register.dob")}</label>
            <div className="auth-dob-row">
              <input
                type="text"
                inputMode="numeric"
                placeholder={t("auth.register.dob.day")}
                value={day}
                onChange={(e) => setDay(e.target.value.replace(/\D/g, ""))}
                maxLength={2}
                aria-label={t("auth.register.dob.day")}
              />
              <input
                type="text"
                inputMode="numeric"
                placeholder={t("auth.register.dob.month")}
                value={month}
                onChange={(e) => setMonth(e.target.value.replace(/\D/g, ""))}
                maxLength={2}
                aria-label={t("auth.register.dob.month")}
              />
              <input
                type="text"
                inputMode="numeric"
                placeholder={t("auth.register.dob.year")}
                value={year}
                onChange={(e) => setYear(e.target.value.replace(/\D/g, ""))}
                maxLength={4}
                aria-label={t("auth.register.dob.year")}
              />
            </div>
            <label className="auth-checkbox-row">
              <input
                type="checkbox"
                checked={dobHidden}
                onChange={(e) => setDobHidden(e.target.checked)}
              />
              {t("auth.register.dob.hide")}
            </label>
          </div>

          <div className="auth-field">
            <label htmlFor="reg-username">{t("auth.register.username")}</label>
            <input
              id="reg-username"
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder={t("auth.register.username.placeholder")}
              autoComplete="username"
            />
            <div className="auth-field__hint">{t("auth.register.username.hint")}</div>
            {errors.username && <div className="auth-field__error">{errors.username}</div>}
            {suggestions.length > 0 && (
              <div className="auth-suggestions">
                {suggestions.map((s) => (
                  <button
                    type="button"
                    key={s}
                    onClick={() => {
                      setUsername(s);
                      // Clearing the error+suggestions hints to the user that the chip
                      // selection has been accepted; a re-submit will re-fetch fresh ones.
                      setErrors({});
                      setSuggestions([]);
                    }}
                  >
                    @{s}
                  </button>
                ))}
              </div>
            )}
          </div>
        </section>
      </div>

      {errors._ && <div className="auth-error-summary">{errors._}</div>}

      <div className="auth-submit-row">
        <button className="auth-submit" type="submit" disabled={submitting}>
          {t("auth.register.submit")}
        </button>
      </div>
    </form>
  );
}
