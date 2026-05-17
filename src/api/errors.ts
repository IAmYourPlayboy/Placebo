/**
 * Typed wrapper for non-2xx responses from the placebo-api.
 *
 * The backend's error envelope is `{ error: { code, message, ...extra } }`. We flatten that
 * into discrete fields here so callers can pattern-match on `code` (e.g. "USERNAME_TAKEN")
 * and reach for typed extras (e.g. `extra.suggestions`) without re-walking the envelope.
 */
export class ApiError extends Error {
  constructor(
    /** HTTP status code (e.g. 401, 409). */
    public status: number,
    /** Stable string code from the server, e.g. "USERNAME_TAKEN". Lowercased on the wire is OK. */
    public code: string,
    /** Human-readable message; safe to render to the user. */
    message: string,
    /** Anything else inside the `error` envelope (e.g. `suggestions: string[]`). */
    public extra?: Record<string, unknown>,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export function isApiError(e: unknown): e is ApiError {
  return e instanceof ApiError;
}
