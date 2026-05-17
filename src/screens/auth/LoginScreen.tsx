/**
 * Login form. Tiny by design — just email + password. Forgot-password is rendered as a
 * disabled link in alpha; the password-reset flow itself ships in a later milestone.
 */

import { FormEvent, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { ApiError, isApiError } from "../../api/errors";
import { useAuth } from "../../auth/useAuth";
import type { LoginRequest } from "../../types/api/LoginRequest";
import "./auth.css";

export default function LoginScreen() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { login } = useAuth();

  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      const payload: LoginRequest = { email: email.trim(), password };
      await login(payload);
      navigate("/home", { replace: true });
    } catch (err) {
      if (isApiError(err) && (err.code === "UNAUTHORIZED" || err.status === 401)) {
        setError(t("auth.error.invalid_credentials"));
      } else if (err instanceof ApiError) {
        setError(err.message);
      } else {
        setError(t("auth.error.generic"));
      }
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form
      className="auth-form"
      onSubmit={submit}
      noValidate
      style={{ maxWidth: 460 }}
    >
      <button type="button" className="auth-back" onClick={() => navigate("/welcome")}>
        {"< "}
        {t("auth.register.back")}
      </button>
      <h1 className="auth-form__title">{t("auth.login.title")}</h1>

      <section className="auth-form__section">
        <div className="auth-field">
          <label htmlFor="login-email">{t("auth.register.email")}</label>
          <input
            id="login-email"
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder={t("auth.register.email.placeholder")}
            autoComplete="email"
          />
        </div>
        <div className="auth-field">
          <label htmlFor="login-password">{t("auth.register.password")}</label>
          <input
            id="login-password"
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            autoComplete="current-password"
          />
        </div>
        {error && <div className="auth-field__error">{error}</div>}
      </section>

      <div className="auth-submit-row">
        <button className="auth-submit" type="submit" disabled={submitting}>
          {t("auth.login.submit")}
        </button>
      </div>

      <div className="auth-hint">
        {/* Forgot-password is intentionally inert in alpha — the reset endpoint exists
            on the server but the email side and dedicated UI land in a later milestone. */}
        <span
          className="auth-form__small-link"
          aria-disabled="true"
          style={{ pointerEvents: "none" }}
        >
          {t("auth.login.forgot")}
        </span>
      </div>
    </form>
  );
}
