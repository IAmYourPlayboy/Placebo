/**
 * Thin fetch wrapper for the placebo-api server.
 *
 * Responsibilities:
 *  - Prefix all paths with VITE_API_BASE_URL.
 *  - Attach the bearer token from tokenStorage when `auth !== false`.
 *  - JSON-encode the body and parse the response.
 *  - Map non-2xx responses to a typed `ApiError`, preserving server-supplied error code,
 *    message, and any extras (e.g. username suggestions).
 *
 * Refresh policy: the backend uses opaque session tokens stored in Redis with sliding TTL —
 * there is no JWT exp field for the client to inspect. AuthProvider takes care of session
 * lifecycle (clearing on 401, calling /auth/refresh in response to its own signals); the
 * client itself stays dumb on purpose.
 */

import { loadToken } from "../auth/tokenStorage";
import { ApiError } from "./errors";

const BASE: string =
  (import.meta.env.VITE_API_BASE_URL as string | undefined) ?? "http://localhost:3001/api/v1";

type Method = "GET" | "POST" | "DELETE" | "PATCH" | "PUT";

export interface RequestOptions {
  method?: Method;
  /** JSON-encoded as the request body. Pass `undefined` to send no body. */
  body?: unknown;
  /** Attach the bearer token. Defaults to `true`. Set `false` for register/login/refresh. */
  auth?: boolean;
  /** Extra headers (merged on top of `content-type: application/json`). */
  headers?: Record<string, string>;
  /** AbortSignal for cancelling the request. */
  signal?: AbortSignal;
}

async function parseBody(res: Response): Promise<unknown> {
  const text = await res.text();
  if (!text) return null;
  try {
    return JSON.parse(text);
  } catch {
    // Server returned something non-JSON (HTML error page, plain text). Surface it raw
    // so the caller can decide what to do.
    return text;
  }
}

/**
 * The server wraps errors as `{ error: { code, message, ...extra } }`. Walk that envelope
 * and produce an `ApiError`. If the body is missing or malformed, fall back to status-derived
 * placeholders so the caller still gets a typed error.
 */
function toApiError(status: number, payload: unknown): ApiError {
  if (payload && typeof payload === "object" && "error" in payload) {
    const env = (payload as { error: unknown }).error;
    if (env && typeof env === "object") {
      const obj = env as Record<string, unknown>;
      const code = typeof obj.code === "string" ? obj.code : `http_${status}`;
      const message =
        typeof obj.message === "string" ? obj.message : `Request failed (${status})`;
      const { code: _c, message: _m, ...extra } = obj;
      return new ApiError(status, code, message, extra);
    }
  }
  return new ApiError(status, `http_${status}`, `Request failed (${status})`);
}

export async function apiRequest<T>(path: string, opts: RequestOptions = {}): Promise<T> {
  const { method = "GET", body, auth = true, headers = {}, signal } = opts;

  const h: Record<string, string> = {
    "content-type": "application/json",
    ...headers,
  };

  if (auth) {
    const token = await loadToken();
    if (token) h["Authorization"] = `Bearer ${token}`;
  }

  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: h,
    body: body === undefined ? undefined : JSON.stringify(body),
    signal,
  });

  const payload = await parseBody(res);

  if (!res.ok) {
    throw toApiError(res.status, payload);
  }

  return payload as T;
}
