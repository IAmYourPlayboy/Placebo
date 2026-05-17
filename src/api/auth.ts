/**
 * Typed wrappers around the placebo-api auth endpoints.
 *
 * Types are generated from the Rust DTOs by ts-rs (`scripts/gen-types.sh`) and live under
 * `src/types/api/`. Don't redefine them here.
 */

import type { AuthResponse } from "../types/api/AuthResponse";
import type { LoginRequest } from "../types/api/LoginRequest";
import type { MeResponse } from "../types/api/MeResponse";
import type { MessageResponse } from "../types/api/MessageResponse";
import type { RefreshRequest } from "../types/api/RefreshRequest";
import type { RegisterRequest } from "../types/api/RegisterRequest";

import { apiRequest } from "./client";

export function register(input: RegisterRequest): Promise<AuthResponse> {
  return apiRequest<AuthResponse>("/auth/register", {
    method: "POST",
    body: input,
    auth: false,
  });
}

export function login(input: LoginRequest): Promise<AuthResponse> {
  return apiRequest<AuthResponse>("/auth/login", {
    method: "POST",
    body: input,
    auth: false,
  });
}

export function logout(): Promise<MessageResponse> {
  return apiRequest<MessageResponse>("/auth/logout", { method: "POST" });
}

/**
 * Extend the current session's TTL on the server. The backend keeps opaque tokens in Redis,
 * so refresh just bumps the expiry — the token itself is unchanged. Returns a message; the
 * caller doesn't get a new AuthResponse.
 */
export function refresh(input: RefreshRequest): Promise<MessageResponse> {
  return apiRequest<MessageResponse>("/auth/refresh", {
    method: "POST",
    body: input,
    auth: false,
  });
}

export function me(): Promise<MeResponse> {
  return apiRequest<MeResponse>("/me");
}
