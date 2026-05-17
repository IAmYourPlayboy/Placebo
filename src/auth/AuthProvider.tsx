/**
 * Auth context provider.
 *
 * Lifecycle:
 *   1. On mount, read the persisted bearer token from tokenStorage. If absent, status -> "anonymous".
 *   2. If a token exists, fetch /me. On success status -> "authenticated" with the user payload.
 *      On 401, the token is dead — clear it and fall back to anonymous.
 *   3. register / login save the new token, then refetch /me so the user payload is the
 *      authoritative version (covers cases where AuthResponse and MeResponse drift).
 *   4. logout best-efforts the server-side delete and always wipes local state.
 *
 * Stable callbacks: register/login/logout/refetchMe never close over `user` or `status`,
 * so the context value's identity is mostly stable. Consumers re-render only when the
 * relevant slice (status, user) actually changes — same discipline as TabManager.
 */

import {
  createContext,
  ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from "react";
import * as authApi from "../api/auth";
import { ApiError } from "../api/errors";
import type { LoginRequest } from "../types/api/LoginRequest";
import type { MeResponse } from "../types/api/MeResponse";
import type { RegisterRequest } from "../types/api/RegisterRequest";
import { clearToken, loadToken, saveToken } from "./tokenStorage";

export type AuthStatus = "bootstrapping" | "anonymous" | "authenticated";

export interface AuthApi {
  status: AuthStatus;
  user: MeResponse | null;
  register(input: RegisterRequest): Promise<void>;
  login(input: LoginRequest): Promise<void>;
  logout(): Promise<void>;
  refetchMe(): Promise<void>;
}

export const AuthContext = createContext<AuthApi | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [status, setStatus] = useState<AuthStatus>("bootstrapping");
  const [user, setUser] = useState<MeResponse | null>(null);

  const refetchMe = useCallback(async () => {
    try {
      const me = await authApi.me();
      setUser(me);
      setStatus("authenticated");
    } catch (e) {
      if (e instanceof ApiError && e.status === 401) {
        // Token rejected — clear and downgrade to anonymous instead of bubbling.
        await clearToken();
        setUser(null);
        setStatus("anonymous");
        return;
      }
      // Network or 5xx: keep whatever state we had so the user isn't logged out by
      // a flaky connection. The caller (or the next interaction) will retry.
      throw e;
    }
  }, []);

  // Bootstrap on mount: read token, hit /me, settle into anonymous|authenticated.
  useEffect(() => {
    let cancelled = false;
    (async () => {
      const token = await loadToken();
      if (cancelled) return;
      if (!token) {
        setStatus("anonymous");
        return;
      }
      try {
        await refetchMe();
      } catch {
        // Bootstrap-time network failure: pretend we're anonymous so the UI is usable.
        // The user can log in manually; if the previously stored token still works the
        // next /me call after a successful login will pick it up.
        if (!cancelled) setStatus("anonymous");
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [refetchMe]);

  const register = useCallback<AuthApi["register"]>(
    async (input) => {
      const resp = await authApi.register(input);
      await saveToken(resp.token);
      await refetchMe();
    },
    [refetchMe],
  );

  const login = useCallback<AuthApi["login"]>(
    async (input) => {
      const resp = await authApi.login(input);
      await saveToken(resp.token);
      await refetchMe();
    },
    [refetchMe],
  );

  const logout = useCallback<AuthApi["logout"]>(async () => {
    try {
      await authApi.logout();
    } catch {
      // Server-side logout is best-effort: even if it fails (network down, token already
      // expired) we still wipe the local state so the user is locally logged out.
    }
    await clearToken();
    setUser(null);
    setStatus("anonymous");
  }, []);

  const value = useMemo<AuthApi>(
    () => ({ status, user, register, login, logout, refetchMe }),
    [status, user, register, login, logout, refetchMe],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
