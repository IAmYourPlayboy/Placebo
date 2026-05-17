/**
 * Renders children only when the user is authenticated; otherwise redirects to /welcome.
 *
 * The bootstrapping case (we don't yet know the user's status) deliberately renders a
 * placeholder rather than redirecting — redirecting on every reload while we wait for
 * /me would flash the welcome screen for any logged-in user with a slow network.
 */

import { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Navigate } from "react-router-dom";
import { useAuth } from "./useAuth";

export function AuthGuard({ children }: { children: ReactNode }) {
  const { status } = useAuth();
  const { t } = useTranslation();

  if (status === "bootstrapping") {
    return (
      <div
        style={{
          padding: 32,
          color: "var(--t2)",
          fontSize: 14,
        }}
      >
        {t("app.loading", "Загрузка...")}
      </div>
    );
  }
  if (status === "anonymous") {
    return <Navigate to="/welcome" replace />;
  }
  return <>{children}</>;
}
