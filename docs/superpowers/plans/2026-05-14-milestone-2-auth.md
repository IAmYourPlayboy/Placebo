# Milestone 2: Auth Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Довести авторизацию до уровня "пользователь регистрируется, логинится, получает сессию, может разлогиниться". Реализовать UI Welcome, Register, Login по Figma + auth-контекст + API-клиент с JWT refresh + AuthGuard для роутов.

**Architecture:**
- **Миграция схемы**: добавить в `users` поля `username`, `username_normalized`, `date_of_birth`, `date_of_birth_hidden`.
- **Бэкенд**: расширить `RegisterRequest` + `AuthResponse` новыми полями. Добавить эндпоинт `GET /api/v1/me`. Убедиться, что ошибки "username taken" возвращаются структурированно.
- **Фронт**: новый `AuthProvider` хранит user + токен, читает/пишет через Tauri keychain (fallback `localStorage`). API-клиент с автоматическим refresh на 401. `AuthGuard` wrapper для защиты роутов – redirect на `/welcome`, если не авторизован.
- **UI по Figma**: Welcome, Register (обязательные + опциональные поля), Login. Социальные кнопки показываем, но disabled. Кнопка "Попробовать без аккаунта" disabled.
- **ts-rs экспорт**: все auth-типы получают `#[derive(TS)]`, попадают в `src/types/api/*.ts` автоматически.

**Tech Stack:** axum, sqlx, Argon2id, JWT-like opaque tokens (текущая реализация использует opaque tokens в Redis), react-i18next, React Router v6.

**Spec reference:** `docs/superpowers/specs/2026-05-14-alpha-design.md`, разделы 2.1 (scope), 6 (экраны), 8 (безопасность), 13.3 (решения по username/display_name/DOB).

**Зависимость от M0 и M1:** i18n, ThemeProvider, TabManager, ts-rs pipeline, Tauri prefs.

---

## File Map

### Backend (crates/placebo-api)

- Create: `crates/placebo-api/migrations/008_users_username_dob.sql` – добавить username, DOB.
- Modify: `crates/placebo-shared/src/auth.rs` – расширить requests/responses, добавить `MeResponse`, `UsernameAvailability`, ts-rs derives.
- Modify: `crates/placebo-api/src/repositories/user_repo.rs` – username/DOB в create/select.
- Modify: `crates/placebo-api/src/services/auth_service.rs` – валидация username, генерация fallback-username, ON CONFLICT user-friendly error.
- Create: `crates/placebo-api/src/handlers/me.rs` – `GET /api/v1/me`.
- Modify: `crates/placebo-api/src/handlers/mod.rs` – подключить.
- Modify: `crates/placebo-api/src/app_state.rs` / routes – смонтировать `/me`.
- Modify: `crates/placebo-api/src/extractors/auth.rs` – убедиться, что `AuthUser` содержит все нужные поля.

### Frontend

- Create: `src/api/client.ts` – HTTP-клиент с токеном, auto-refresh на 401.
- Create: `src/api/auth.ts` – функции `register`, `login`, `logout`, `me`, `refresh`.
- Create: `src/auth/AuthProvider.tsx` – контекст.
- Create: `src/auth/useAuth.ts` – хук.
- Create: `src/auth/AuthGuard.tsx` – HOC для защиты роутов.
- Create: `src/auth/tokenStorage.ts` – Tauri keychain + localStorage fallback.
- Create: `src/screens/auth/WelcomeScreen.tsx`
- Create: `src/screens/auth/RegisterScreen.tsx`
- Create: `src/screens/auth/LoginScreen.tsx`
- Create: `src/screens/auth/auth.css` – стили для трёх экранов.
- Modify: `src/shell/routes.tsx` – добавить auth-роуты, обернуть защищённые в `<AuthGuard>`.
- Modify: `src/App.tsx` – добавить `<AuthProvider>`.
- Modify: `src/i18n/locales/ru.json` – ключи для auth.
- Modify: `src/screens/settings/SettingsScreen.tsx` – теперь кнопка "Выйти" работает.
- Create: `.env` / `.env.development` – `VITE_API_BASE_URL`.

### Tauri (для keychain)

- Modify: `src-tauri/Cargo.toml` – добавить `tauri-plugin-stronghold` **или** `keyring` (выбираем `keyring` – проще и не требует мастер-пароля).
- Modify: `src-tauri/src/lib.rs` + `src-tauri/src/commands/` – команды `secure_get`, `secure_set`, `secure_delete` для токенов.
- Create: `src-tauri/src/commands/secure.rs`.

---

## Task 1: Ветка и подготовка

- [ ] **Step 1: Ветка**

```bash
git -C d:/Projects/Placebo checkout main
git -C d:/Projects/Placebo pull
git -C d:/Projects/Placebo checkout -b feat/m2-auth
```

- [ ] **Step 2: Создать `.env.development` в корне**

```
VITE_API_BASE_URL=http://localhost:3001/api/v1
VITE_WS_BASE_URL=ws://localhost:3001/ws
```

`.env.development` – не коммитим (добавить в `.gitignore` если ещё нет). Создать `.env.example` с теми же ключами для других разработчиков.

- [ ] **Step 3: Проверить, что API-сервер поднимается**

```bash
cd d:/Projects/Placebo/crates/placebo-api
cargo run
```

Expected: "listening on 0.0.0.0:3001". Остановить.

- [ ] **Step 4: Commit .env.example**

```bash
git add .env.example .gitignore
git commit -m "chore: add .env.example, gitignore .env.development"
```

---

## Task 2: Миграция схемы – username + DOB

**Files:**
- Create: `crates/placebo-api/migrations/008_users_username_dob.sql`

**Спец-требования из спеки (13.3):**
- `username` – латиница + цифры + `_`, case-insensitive уникальность.
- `display_name` – любые символы, НЕ уникален.
- `date_of_birth` – опционально.
- `date_of_birth_hidden` – скрыть ли ДР в публичном профиле.

- [ ] **Step 1: Написать миграцию**

Создать `crates/placebo-api/migrations/008_users_username_dob.sql`:

```sql
-- 008_users_username_dob.sql
-- Add username (unique, case-insensitive), date of birth, and DOB visibility

ALTER TABLE users
    ADD COLUMN username              TEXT,
    ADD COLUMN username_normalized   TEXT GENERATED ALWAYS AS (lower(username)) STORED,
    ADD COLUMN date_of_birth         DATE,
    ADD COLUMN date_of_birth_hidden  BOOLEAN NOT NULL DEFAULT TRUE;

-- Unique on lowercased username; NULL allowed during migration
CREATE UNIQUE INDEX idx_users_username_normalized
    ON users (username_normalized)
    WHERE username_normalized IS NOT NULL;

-- Backfill existing dev users with a generated username derived from email prefix
UPDATE users
SET username = split_part(email, '@', 1) || '_' || substring(id::text, 1, 6)
WHERE username IS NULL;

-- From now on, username is mandatory for new rows; keep it nullable for legacy, but
-- application-level validation enforces it in register flow.
```

- [ ] **Step 2: Прогнать миграции**

```bash
cd d:/Projects/Placebo/crates/placebo-api
cargo sqlx migrate run
```

Expected: "Applied 8/008_users_username_dob" без ошибок.

- [ ] **Step 3: Проверить руками**

```bash
psql -d placebo_dev -c "\d users"
```

Expected: видны поля `username`, `username_normalized`, `date_of_birth`, `date_of_birth_hidden`.

- [ ] **Step 4: Commit**

```bash
git add crates/placebo-api/migrations/008_users_username_dob.sql
git commit -m "$(cat <<'EOF'
feat(db): migration 008 adds username + date_of_birth to users

- username is nullable for legacy rows but validated at the application
  layer for new registrations; username_normalized is a generated column
  used for case-insensitive uniqueness.
- date_of_birth is optional; date_of_birth_hidden defaults to TRUE so
  users opt in to showing DOB publicly.
- Dev users are back-filled with emailprefix_<uuid6> usernames.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Расширить auth-типы в placebo-shared (с ts-rs)

**Files:**
- Modify: `crates/placebo-shared/src/auth.rs` (добавить поля, ts-rs, UsernameAvailability)
- Modify: `crates/placebo-shared/src/user.rs` (добавить MeResponse)
- Modify: `crates/placebo-shared/Cargo.toml` (ts-rs feature уже добавлен в M0)

- [ ] **Step 1: Расширить RegisterRequest**

Заменить `RegisterRequest` в `crates/placebo-shared/src/auth.rs`:

```rust
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[cfg(feature = "export-types")]
use ts_rs::TS;

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
    /// Desired username. If None, the server generates one.
    pub username: Option<String>,
    /// Optional ISO-8601 date (yyyy-mm-dd).
    pub date_of_birth: Option<NaiveDate>,
    /// Hide DOB from public profile. Defaults to true.
    pub date_of_birth_hidden: Option<bool>,
    /// BCP-47 locale tag, e.g. "ru".
    pub locale: Option<String>,
}
```

- [ ] **Step 2: Расширить AuthResponse**

```rust
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub is_premium: bool,
    pub expires_in_seconds: u64,
}
```

- [ ] **Step 3: Добавить типы username availability**

```rust
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct UsernameSuggestionSet {
    pub suggestions: Vec<String>,
}
```

- [ ] **Step 4: Остальные auth-типы**

`LoginRequest`, `RefreshRequest`, `PasswordResetRequest`, `PasswordResetConfirm`, `MessageResponse` – добавить `#[cfg_attr(feature = "export-types", derive(TS), ts(export, ...))]`.

- [ ] **Step 5: Добавить MeResponse в user.rs**

Открыть `crates/placebo-shared/src/user.rs`. Добавить:

```rust
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[cfg(feature = "export-types")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub locale: String,
    pub is_premium: bool,
    pub premium_until: Option<DateTime<Utc>>,
    pub date_of_birth: Option<NaiveDate>,
    pub date_of_birth_hidden: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}
```

- [ ] **Step 6: Расширить валидацию RegisterRequest**

Добавить в метод `validate()`:

```rust
// username (optional on client; required in DB – server generates if None)
if let Some(u) = &self.username {
    let u = u.trim();
    if u.is_empty() {
        // treat explicit empty as "not provided"
    } else if u.len() < 3 {
        errors.push("Username must be at least 3 characters".into());
    } else if u.len() > 24 {
        errors.push("Username too long (max 24 chars)".into());
    } else if !u.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        errors.push("Username may contain only Latin letters, digits, and underscore".into());
    } else if u.starts_with('_') || u.ends_with('_') {
        errors.push("Username must not start or end with underscore".into());
    }
}

// Date of birth sanity: must be in the past and age <= 120
if let Some(dob) = self.date_of_birth {
    let today = chrono::Utc::now().date_naive();
    if dob > today {
        errors.push("Date of birth cannot be in the future".into());
    }
    if today.years_since(dob).unwrap_or(0) > 120 {
        errors.push("Date of birth looks invalid".into());
    }
}
```

- [ ] **Step 7: Поправить существующие тесты**

Существующие тесты в `auth.rs` передают `RegisterRequest` без новых полей. Обновить их:

```rust
RegisterRequest {
    email: "bad".into(),
    password: "12345678".into(),
    display_name: "Test".into(),
    username: None,
    date_of_birth: None,
    date_of_birth_hidden: None,
    locale: None,
}
```

Добавить новые тесты:

```rust
#[test]
fn register_request_rejects_bad_username() {
    let mut req = RegisterRequest {
        email: "user@test.com".into(),
        password: "12345678".into(),
        display_name: "Test".into(),
        username: Some("ab".into()),
        date_of_birth: None,
        date_of_birth_hidden: None,
        locale: None,
    };
    assert!(req.validate().iter().any(|e| e.contains("3 characters")));

    req.username = Some("has space".into());
    assert!(req.validate().iter().any(|e| e.contains("Latin")));

    req.username = Some("_leading".into());
    assert!(req.validate().iter().any(|e| e.contains("underscore")));
}

#[test]
fn register_request_accepts_good_username() {
    let req = RegisterRequest {
        email: "user@test.com".into(),
        password: "12345678".into(),
        display_name: "Test".into(),
        username: Some("cool_user123".into()),
        date_of_birth: Some(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
        date_of_birth_hidden: Some(true),
        locale: None,
    };
    assert!(req.validate().is_empty());
}

#[test]
fn register_request_rejects_future_dob() {
    let future = chrono::Utc::now().date_naive() + chrono::Duration::days(1);
    let req = RegisterRequest {
        email: "u@t.com".into(), password: "12345678".into(), display_name: "T".into(),
        username: None, date_of_birth: Some(future), date_of_birth_hidden: None, locale: None,
    };
    assert!(req.validate().iter().any(|e| e.contains("future")));
}
```

- [ ] **Step 8: Запустить тесты**

```bash
cd d:/Projects/Placebo
cargo test -p placebo-shared
```

Expected: все тесты проходят.

- [ ] **Step 9: Запустить экспорт типов**

```bash
npm run gen-types
ls src/types/api/
```

Expected: появляются файлы `RegisterRequest.ts`, `AuthResponse.ts`, `LoginRequest.ts`, `MeResponse.ts` и другие.

- [ ] **Step 10: Commit**

```bash
git add crates/placebo-shared/ src/types/api/
git commit -m "$(cat <<'EOF'
feat(auth): extend RegisterRequest with username + dob, add MeResponse

- ts-rs derives on all shared auth/user DTOs -> auto-generated TS.
- RegisterRequest validates username (3-24, latin+digits+_, no edge _)
  and date_of_birth sanity.
- AuthResponse returns username alongside displayName so clients show
  both forms without an extra /me round-trip after login.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Обновить user_repo и auth_service

**Files:**
- Modify: `crates/placebo-api/src/repositories/user_repo.rs`
- Modify: `crates/placebo-api/src/services/auth_service.rs`

- [ ] **Step 1: Добавить helper для генерации username**

В `user_repo.rs` добавить:

```rust
/// Check if username (case-insensitive) is available.
pub async fn username_available(pool: &PgPool, username: &str) -> sqlx::Result<bool> {
    let found: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM users WHERE username_normalized = lower($1) LIMIT 1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;
    Ok(found.is_none())
}

/// Generate a candidate username from display_name or email, falling back
/// to a random suffix when collisions happen.
pub async fn generate_unique_username(pool: &PgPool, hint: &str) -> sqlx::Result<String> {
    // Reduce hint to ascii-alnum-underscore, lowercase, trimmed to 12 chars.
    let base: String = hint
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .take(12)
        .collect();
    let base = if base.len() < 3 { "user".to_string() } else { base };

    for _ in 0..10 {
        let suffix: u32 = rand::random::<u32>() % 10_000;
        let candidate = format!("{base}_{suffix:04}");
        if username_available(pool, &candidate).await? {
            return Ok(candidate);
        }
    }
    // Last-resort: timestamp suffix
    let ts = chrono::Utc::now().timestamp_millis();
    Ok(format!("{base}_{ts}"))
}
```

- [ ] **Step 2: Обновить create_user**

Заменить `create_user` (или добавить `create_user_v2`, если боимся ломать существующие тесты) так, чтобы принимать username и dob:

```rust
pub struct CreateUserArgs<'a> {
    pub email: &'a str,
    pub display_name: &'a str,
    pub username: &'a str,
    pub password_hash: &'a str,
    pub locale: &'a str,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub date_of_birth_hidden: bool,
    pub email_verified: bool,
}

pub async fn create_user(pool: &PgPool, args: CreateUserArgs<'_>) -> sqlx::Result<Uuid> {
    let rec: (Uuid,) = sqlx::query_as(
        "INSERT INTO users \
         (email, display_name, username, password_hash, locale, date_of_birth, date_of_birth_hidden, email_verified) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         RETURNING id",
    )
    .bind(args.email)
    .bind(args.display_name)
    .bind(args.username)
    .bind(args.password_hash)
    .bind(args.locale)
    .bind(args.date_of_birth)
    .bind(args.date_of_birth_hidden)
    .bind(args.email_verified)
    .fetch_one(pool)
    .await?;
    Ok(rec.0)
}
```

- [ ] **Step 3: Поправить auth_service::register**

В `auth_service::register` перед вставкой:

1. Если `req.username` Some: проверить `username_available` → если занят, вернуть `AppError::Conflict("username_taken", suggestions)` (см. ниже). Если свободен – использовать.
2. Если `req.username` None – вызвать `generate_unique_username(pool, &req.display_name)`.
3. Вставить через `create_user(CreateUserArgs { ... })`.

Фрагмент:

```rust
let username = match &req.username {
    Some(u) if !u.trim().is_empty() => {
        let u = u.trim();
        if !user_repo::username_available(pool, u).await? {
            // Generate 3 alternative suggestions
            let mut sugg = Vec::new();
            for _ in 0..3 {
                sugg.push(user_repo::generate_unique_username(pool, u).await?);
            }
            return Err(AppError::UsernameTaken { suggestions: sugg });
        }
        u.to_string()
    }
    _ => user_repo::generate_unique_username(pool, &req.display_name).await?,
};
```

- [ ] **Step 4: Добавить AppError::UsernameTaken**

В `crates/placebo-api/src/error.rs` добавить вариант и маппинг:

```rust
#[error("Username already taken")]
UsernameTaken { suggestions: Vec<String> },

// В impl IntoResponse for AppError:
Self::UsernameTaken { suggestions } => (
    StatusCode::CONFLICT,
    Json(json!({
        "error": "username_taken",
        "message": "Username already taken",
        "suggestions": suggestions,
    })),
).into_response(),
```

- [ ] **Step 5: AuthResponse теперь возвращает username**

В `auth_service` везде, где строится `AuthResponse`, добавить `username: user.username`.

- [ ] **Step 6: Rust-тесты auth_service (integration)**

Если у `auth_service` есть integration-тесты (через `#[sqlx::test]` или testcontainers) – обновить. Если нет, создать минимальный:

```rust
#[sqlx::test]
async fn register_generates_username_when_not_provided(pool: PgPool) -> sqlx::Result<()> {
    // minimal redis stub not set up here; focus on DB side
    // (Skipped if the service requires live redis; leave as TODO to cover in M5 integration test harness.)
    Ok(())
}
```

Если live-тесты слишком сложны – пометить как TODO и покрыть ручной проверкой в Task 6.

- [ ] **Step 7: cargo check + test**

```bash
cd d:/Projects/Placebo
cargo check --workspace
cargo test -p placebo-shared
cargo test -p placebo-api --lib
```

Expected: всё проходит.

- [ ] **Step 8: Commit**

```bash
git add crates/placebo-api/src/repositories/user_repo.rs crates/placebo-api/src/services/auth_service.rs crates/placebo-api/src/error.rs
git commit -m "$(cat <<'EOF'
feat(auth): register flow honours username/dob; 409 returns suggestions

- user_repo::username_available + generate_unique_username helpers.
- CreateUserArgs struct replaces the positional create_user signature,
  keeping it extensible as the users table grows.
- auth_service generates a fallback username from display_name when
  none is provided; returns AppError::UsernameTaken with 3 alternatives
  on conflict, matching the spec's "suggest 3 free variants" decision.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: GET /api/v1/me

**Files:**
- Create: `crates/placebo-api/src/handlers/me.rs`
- Modify: `crates/placebo-api/src/handlers/mod.rs`
- Modify: место монтирования router в `lib.rs` / app-сборщике.

- [ ] **Step 1: Handler**

```rust
// src/handlers/me.rs
use axum::{extract::State, routing::get, Json, Router};
use placebo_shared::user::MeResponse;
use crate::app_state::AppState;
use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::repositories::user_repo;

pub fn router() -> Router<AppState> {
    Router::new().route("/me", get(me))
}

async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<MeResponse>, AppError> {
    let u = user_repo::get_by_id(&state.db, auth.id)
        .await?
        .ok_or(AppError::Unauthorized("User not found".into()))?;
    Ok(Json(MeResponse {
        id: u.id,
        email: u.email,
        username: u.username,
        display_name: u.display_name,
        avatar_url: u.avatar_url,
        locale: u.locale,
        is_premium: u.is_premium,
        premium_until: u.premium_until,
        date_of_birth: u.date_of_birth,
        date_of_birth_hidden: u.date_of_birth_hidden,
        email_verified: u.email_verified,
        created_at: u.created_at,
    }))
}
```

- [ ] **Step 2: Проверить/расширить `user_repo::get_by_id`**

После миграции 008 запрос должен возвращать новые поля. Обновить структуру `User` в `user_repo.rs`:

```rust
#[derive(sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String, // non-null after migration backfill
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub locale: String,
    pub is_premium: bool,
    pub premium_until: Option<chrono::DateTime<chrono::Utc>>,
    pub password_hash: Option<String>,
    pub email_verified: bool,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub date_of_birth_hidden: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

и SQL `SELECT ... FROM users WHERE id = $1` с перечислением колонок.

- [ ] **Step 3: Смонтировать в app**

В `handlers/mod.rs`:

```rust
pub mod me;
```

В месте сборки router (обычно `lib.rs` или `app_state.rs` – поискать `Router::new().nest("/auth"`):

```rust
.merge(handlers::me::router())
```

- [ ] **Step 4: Смоук-тест через curl**

```bash
cd d:/Projects/Placebo/crates/placebo-api
cargo run &
sleep 3

# 1) Register
curl -s -X POST http://localhost:3001/api/v1/auth/register \
  -H "content-type: application/json" \
  -d '{"email":"u1@test.com","password":"secret12345","displayName":"Test User","username":"test_user"}' \
  | tee /tmp/register.json

TOKEN=$(jq -r .token /tmp/register.json)

# 2) /me
curl -s -H "Authorization: Bearer $TOKEN" http://localhost:3001/api/v1/me | jq .

kill %1 2>/dev/null
```

Expected: `/me` возвращает JSON с `username: "test_user"`, `displayName: "Test User"`, правильными остальными полями.

- [ ] **Step 5: Commit**

```bash
git add crates/placebo-api/src/handlers/me.rs crates/placebo-api/src/handlers/mod.rs crates/placebo-api/src/repositories/user_repo.rs crates/placebo-api/src/lib.rs
git commit -m "feat(api): GET /api/v1/me returns current user profile"
```

---

## Task 6: Tauri keychain-команды

**Files:**
- Modify: `src-tauri/Cargo.toml` – добавить `keyring = "3"`.
- Create: `src-tauri/src/commands/secure.rs`.
- Modify: `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`.

- [ ] **Step 1: Добавить зависимость**

В `src-tauri/Cargo.toml`:

```toml
keyring = "3"
```

- [ ] **Step 2: Команды**

```rust
// src-tauri/src/commands/secure.rs
use keyring::Entry;

const SERVICE: &str = "placebo";

#[tauri::command]
pub fn secure_set(key: String, value: String) -> Result<(), String> {
    Entry::new(SERVICE, &key)
        .and_then(|e| e.set_password(&value))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn secure_get(key: String) -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE, &key).map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn secure_delete(key: String) -> Result<(), String> {
    let entry = Entry::new(SERVICE, &key).map_err(|e| e.to_string())?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
```

- [ ] **Step 3: Регистрация**

В `src-tauri/src/commands/mod.rs`:

```rust
pub mod secure;
```

В `src-tauri/src/lib.rs` в `invoke_handler!`:

```rust
commands::secure::secure_get,
commands::secure::secure_set,
commands::secure::secure_delete,
```

- [ ] **Step 4: cargo check**

```bash
cd d:/Projects/Placebo
cargo check --workspace
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/commands/
git commit -m "feat(tauri): secure_{get,set,delete} via OS keychain"
```

---

## Task 7: Token storage (frontend)

**Files:**
- Create: `src/auth/tokenStorage.ts`

- [ ] **Step 1: Написать модуль**

```ts
// src/auth/tokenStorage.ts
import { invoke } from "@tauri-apps/api/core";

const KEY = "placebo.auth.token";

async function tauriAvailable(): Promise<boolean> {
  return typeof (window as any).__TAURI_INTERNALS__ !== "undefined";
}

export async function saveToken(token: string): Promise<void> {
  if (await tauriAvailable()) {
    try {
      await invoke<void>("secure_set", { key: KEY, value: token });
      return;
    } catch { /* fall through */ }
  }
  localStorage.setItem(KEY, token);
}

export async function loadToken(): Promise<string | null> {
  if (await tauriAvailable()) {
    try {
      const v = await invoke<string | null>("secure_get", { key: KEY });
      if (v) return v;
    } catch { /* fall through */ }
  }
  return localStorage.getItem(KEY);
}

export async function clearToken(): Promise<void> {
  if (await tauriAvailable()) {
    try { await invoke<void>("secure_delete", { key: KEY }); } catch { /* ignore */ }
  }
  localStorage.removeItem(KEY);
}
```

- [ ] **Step 2: Commit**

```bash
git add src/auth/tokenStorage.ts
git commit -m "feat(auth): tokenStorage via Tauri keychain with localStorage fallback"
```

---

## Task 8: HTTP-клиент

**Files:**
- Create: `src/api/client.ts`
- Create: `src/api/errors.ts`

- [ ] **Step 1: Ошибки**

```ts
// src/api/errors.ts
export class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
    public extra?: Record<string, unknown>,
  ) {
    super(message);
  }
}

export function isApiError(e: unknown): e is ApiError {
  return e instanceof ApiError;
}
```

- [ ] **Step 2: Клиент**

```ts
// src/api/client.ts
import { ApiError } from "./errors";
import { loadToken } from "../auth/tokenStorage";

const BASE = import.meta.env.VITE_API_BASE_URL ?? "http://localhost:3001/api/v1";

type Method = "GET" | "POST" | "DELETE" | "PATCH" | "PUT";

type RequestOptions = {
  method?: Method;
  body?: unknown;
  auth?: boolean;
  headers?: Record<string, string>;
};

async function parseJson(res: Response): Promise<any> {
  const text = await res.text();
  if (!text) return null;
  try { return JSON.parse(text); } catch { return text; }
}

export async function apiRequest<T>(path: string, opts: RequestOptions = {}): Promise<T> {
  const { method = "GET", body, auth = true, headers = {} } = opts;
  const h: Record<string, string> = { "content-type": "application/json", ...headers };

  if (auth) {
    const token = await loadToken();
    if (token) h["Authorization"] = `Bearer ${token}`;
  }

  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: h,
    body: body === undefined ? undefined : JSON.stringify(body),
  });

  const payload = await parseJson(res);

  if (!res.ok) {
    const code = payload?.error ?? `http_${res.status}`;
    const msg = payload?.message ?? `Request failed (${res.status})`;
    throw new ApiError(res.status, code, msg, payload ?? undefined);
  }

  return payload as T;
}
```

**Примечание по refresh:** у нас в auth – opaque tokens в Redis (не JWT). Обновление происходит через `POST /auth/refresh { token }`. Это обрабатывается не в каждом запросе, а в `AuthProvider` при получении 401 – см. Task 9.

- [ ] **Step 3: Commit**

```bash
git add src/api/client.ts src/api/errors.ts
git commit -m "feat(api): fetch-based client with bearer token"
```

---

## Task 9: Auth API-функции

**Files:**
- Create: `src/api/auth.ts`

- [ ] **Step 1: Типы**

Типы генерируются ts-rs в `src/types/api/`. Импортируем оттуда:

```ts
// src/api/auth.ts
import { apiRequest } from "./client";
import type { RegisterRequest } from "../types/api/RegisterRequest";
import type { LoginRequest } from "../types/api/LoginRequest";
import type { AuthResponse } from "../types/api/AuthResponse";
import type { MeResponse } from "../types/api/MeResponse";
import type { RefreshRequest } from "../types/api/RefreshRequest";
import type { MessageResponse } from "../types/api/MessageResponse";

export async function register(input: RegisterRequest): Promise<AuthResponse> {
  return apiRequest<AuthResponse>("/auth/register", { method: "POST", body: input, auth: false });
}

export async function login(input: LoginRequest): Promise<AuthResponse> {
  return apiRequest<AuthResponse>("/auth/login", { method: "POST", body: input, auth: false });
}

export async function logout(): Promise<MessageResponse> {
  return apiRequest<MessageResponse>("/auth/logout", { method: "POST" });
}

export async function refresh(input: RefreshRequest): Promise<AuthResponse> {
  return apiRequest<AuthResponse>("/auth/refresh", { method: "POST", body: input, auth: false });
}

export async function me(): Promise<MeResponse> {
  return apiRequest<MeResponse>("/me");
}
```

- [ ] **Step 2: Commit**

```bash
git add src/api/auth.ts
git commit -m "feat(api): auth endpoints wrapper"
```

---

## Task 10: AuthProvider и AuthGuard

**Files:**
- Create: `src/auth/AuthProvider.tsx`
- Create: `src/auth/useAuth.ts`
- Create: `src/auth/AuthGuard.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: Провайдер**

```tsx
// src/auth/AuthProvider.tsx
import { createContext, useCallback, useEffect, useMemo, useState, ReactNode } from "react";
import type { MeResponse } from "../types/api/MeResponse";
import type { RegisterRequest } from "../types/api/RegisterRequest";
import type { LoginRequest } from "../types/api/LoginRequest";
import * as authApi from "../api/auth";
import { saveToken, loadToken, clearToken } from "./tokenStorage";
import { ApiError } from "../api/errors";

type Status = "bootstrapping" | "anonymous" | "authenticated";

type AuthApi = {
  status: Status;
  user: MeResponse | null;
  register(input: RegisterRequest): Promise<void>;
  login(input: LoginRequest): Promise<void>;
  logout(): Promise<void>;
  refetchMe(): Promise<void>;
};

export const AuthContext = createContext<AuthApi | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [status, setStatus] = useState<Status>("bootstrapping");
  const [user, setUser] = useState<MeResponse | null>(null);

  const refetchMe = useCallback(async () => {
    try {
      const me = await authApi.me();
      setUser(me);
      setStatus("authenticated");
    } catch (e) {
      if (e instanceof ApiError && e.status === 401) {
        await clearToken();
        setUser(null);
        setStatus("anonymous");
      } else {
        throw e;
      }
    }
  }, []);

  useEffect(() => {
    (async () => {
      const token = await loadToken();
      if (!token) { setStatus("anonymous"); return; }
      try { await refetchMe(); }
      catch { setStatus("anonymous"); }
    })();
  }, [refetchMe]);

  const register = useCallback<AuthApi["register"]>(async (input) => {
    const resp = await authApi.register(input);
    await saveToken(resp.token);
    await refetchMe();
  }, [refetchMe]);

  const login = useCallback<AuthApi["login"]>(async (input) => {
    const resp = await authApi.login(input);
    await saveToken(resp.token);
    await refetchMe();
  }, [refetchMe]);

  const logout = useCallback<AuthApi["logout"]>(async () => {
    try { await authApi.logout(); } catch { /* even if server rejects, clear locally */ }
    await clearToken();
    setUser(null);
    setStatus("anonymous");
  }, []);

  const value = useMemo<AuthApi>(() => ({ status, user, register, login, logout, refetchMe }),
    [status, user, register, login, logout, refetchMe]);

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
```

- [ ] **Step 2: useAuth**

```ts
// src/auth/useAuth.ts
import { useContext } from "react";
import { AuthContext } from "./AuthProvider";

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used within <AuthProvider>");
  return ctx;
}
```

- [ ] **Step 3: AuthGuard**

```tsx
// src/auth/AuthGuard.tsx
import { ReactNode } from "react";
import { Navigate } from "react-router-dom";
import { useAuth } from "./useAuth";
import { useTranslation } from "react-i18next";

export function AuthGuard({ children }: { children: ReactNode }) {
  const { status } = useAuth();
  const { t } = useTranslation();

  if (status === "bootstrapping") {
    return <div style={{ padding: 32, color: "var(--t2)" }}>{t("app.loading")}</div>;
  }
  if (status === "anonymous") {
    return <Navigate to="/welcome" replace />;
  }
  return <>{children}</>;
}
```

- [ ] **Step 4: Обновить App.tsx**

```tsx
import { AuthProvider } from "./auth/AuthProvider";
// ...
export default function App() {
  return (
    <ThemeProvider>
      <AuthProvider>
        <Scene3DRegistry>
          <TabManager initialPath="/home">
            <ShellRoot />
          </TabManager>
        </Scene3DRegistry>
      </AuthProvider>
    </ThemeProvider>
  );
}
```

- [ ] **Step 5: Commit**

```bash
git add src/auth/ src/App.tsx
git commit -m "feat(auth): AuthProvider + useAuth + AuthGuard"
```

---

## Task 11: Welcome screen по Figma

**Files:**
- Create: `src/screens/auth/WelcomeScreen.tsx`
- Create: `src/screens/auth/auth.css`

- [ ] **Step 1: Компонент**

```tsx
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import "./auth.css";

const SOCIAL = ["google", "facebook", "apple", "vk", "telegram", "discord", "x", "phone"] as const;

export default function WelcomeScreen() {
  const { t, i18n } = useTranslation();
  return (
    <div className="auth-welcome">
      <div className="auth-welcome__center">
        <h1 className="auth-welcome__title">Placebo</h1>
        <p className="auth-welcome__subtitle">{t("auth.welcome.subtitle")}</p>

        <div className="auth-welcome__actions">
          <Link to="/register" className="auth-btn auth-btn--primary">{t("auth.welcome.register")}</Link>
          <Link to="/login" className="auth-btn">{t("auth.welcome.login")}</Link>
          <button className="auth-btn" disabled>{t("auth.welcome.try_as_guest")}</button>
        </div>

        <p className="auth-welcome__or">{t("auth.welcome.or_via")}</p>
        <div className="auth-welcome__social">
          {SOCIAL.map((s) => (
            <button key={s} className="auth-social" disabled title={t("auth.welcome.social.soon")}>
              <span className={`auth-social__icon auth-social__icon--${s}`} aria-hidden />
            </button>
          ))}
        </div>

        <div className="auth-welcome__footer">
          <button className="auth-lang" onClick={() => i18n.changeLanguage(i18n.resolvedLanguage === "ru" ? "en" : "ru")}>
            Your language is <b>{i18n.resolvedLanguage === "ru" ? "Russian" : "English"}</b>
          </button>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Стили по Figma**

`src/screens/auth/auth.css`:

```css
.auth-welcome { min-height: 100%; display: grid; place-items: center; padding: 48px; }
.auth-welcome__center { max-width: 680px; text-align: center; }
.auth-welcome__title { font-size: 48px; font-weight: 800; color: var(--t1); margin: 0 0 16px; }
.auth-welcome__subtitle { font-size: 15px; color: var(--t2); margin: 0 0 32px; }
.auth-welcome__actions { display: flex; gap: 12px; justify-content: center; flex-wrap: wrap; margin-bottom: 32px; }

.auth-btn {
  padding: 10px 24px; border-radius: 8px;
  border: 1px solid var(--t1); background: var(--bg);
  color: var(--t1); text-decoration: none; font-size: 15px;
  cursor: pointer;
}
.auth-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.auth-btn--primary { background: var(--t1); color: var(--bg); }

.auth-welcome__or { color: var(--t2); font-size: 13px; margin: 16px 0; }
.auth-welcome__social { display: flex; gap: 12px; justify-content: center; flex-wrap: wrap; }
.auth-social { width: 40px; height: 40px; border-radius: 50%; border: 0; background: var(--bg-2); cursor: pointer; }
.auth-social:disabled { opacity: 0.7; cursor: not-allowed; }
.auth-social__icon { display: block; width: 100%; height: 100%; background-size: 22px 22px; background-repeat: no-repeat; background-position: center; }
/* Individual icon urls plug in later via Figma export. Placeholder: generic dot. */

.auth-welcome__footer { margin-top: 48px; color: var(--t2); font-size: 13px; }
.auth-lang { background: transparent; border: 0; color: inherit; cursor: pointer; }
.auth-lang b { color: var(--t1); }

/* Form shared styles */
.auth-form { max-width: 720px; margin: 0 auto; padding: 32px 24px; }
.auth-form__title { font-size: 28px; font-weight: 700; color: var(--t1); margin: 16px 0 24px; text-align: center; }
.auth-form__grid { display: grid; grid-template-columns: 1fr 1fr; gap: 24px; }
@media (max-width: 720px) { .auth-form__grid { grid-template-columns: 1fr; } }
.auth-form__section { background: var(--bg); border: 1px solid var(--border); border-radius: 12px; padding: 20px; }
.auth-form__section-title { font-size: 14px; color: var(--t2); margin-bottom: 16px; }
.auth-field { display: flex; flex-direction: column; gap: 6px; margin-bottom: 12px; }
.auth-field label { font-size: 13px; color: var(--t2); }
.auth-field input {
  padding: 10px 12px; border-radius: 8px;
  border: 1px solid var(--border); background: var(--bg-2);
  color: var(--t1); font-size: 14px;
}
.auth-field input:focus { outline: 2px solid var(--accent); outline-offset: 1px; }
.auth-field__error { color: #D12850; font-size: 12px; }
.auth-submit-row { display: flex; justify-content: center; margin-top: 24px; }
.auth-submit { padding: 12px 28px; background: var(--t1); color: var(--bg); border: 0; border-radius: 10px; font-size: 15px; cursor: pointer; }
.auth-submit:disabled { opacity: 0.5; cursor: not-allowed; }
.auth-back { background: transparent; border: 0; color: var(--t2); cursor: pointer; padding: 8px 0; }
.auth-hint { color: var(--t3); font-size: 13px; margin-top: 12px; text-align: center; }
.auth-suggestions { display: flex; gap: 8px; flex-wrap: wrap; margin-top: 6px; }
.auth-suggestions button {
  background: var(--bg-2); border: 1px solid var(--border);
  border-radius: 999px; padding: 4px 10px; font-size: 12px;
  color: var(--t1); cursor: pointer;
}
```

- [ ] **Step 3: Ключи i18n**

Добавить в `ru.json`:

```json
{
  "auth.welcome.subtitle": "Рады видеть Вас в этом уютном уголке интернета, где собирается множество людей для совместного онлайн времяпровождения",
  "auth.welcome.register": "Зарегистрироваться",
  "auth.welcome.login": "Войти",
  "auth.welcome.try_as_guest": "Попробовать без аккаунта",
  "auth.welcome.or_via": "Или войти через:",
  "auth.welcome.social.soon": "Скоро",

  "auth.register.title": "Создание аккаунта",
  "auth.register.required": "Обязательные данные:",
  "auth.register.optional": "Необязательные данные:",
  "auth.register.name": "Напишите ваш будущий никнейм:",
  "auth.register.name.first": "Имя",
  "auth.register.name.last": "Фамилия (опционально)",
  "auth.register.email": "Ваша эл. почта:",
  "auth.register.email.placeholder": "example@gmail.com",
  "auth.register.password": "Пароль",
  "auth.register.password.placeholder": "минимум 8 символов",
  "auth.register.dob": "Выберите дату вашего рождения:",
  "auth.register.dob.day": "День",
  "auth.register.dob.month": "Месяц",
  "auth.register.dob.year": "Год",
  "auth.register.dob.hide": "Скрыть в профиле дату рождения для других людей",
  "auth.register.username": "Ваш будущий юзернейм (@):",
  "auth.register.username.placeholder": "@",
  "auth.register.username.hint": "Латиница, цифры и _ (3–24 знака)",
  "auth.register.avatar": "Добавить фото профиля",
  "auth.register.submit": "Создать аккаунт",
  "auth.register.back": "< Вернуться назад",
  "auth.register.error.username_taken": "Этот юзернейм занят. Попробуйте:",

  "auth.login.title": "Вход",
  "auth.login.submit": "Войти",
  "auth.login.forgot": "Забыли пароль?",

  "auth.error.generic": "Ошибка. Попробуйте ещё раз.",
  "auth.error.invalid_credentials": "Неверный email или пароль."
}
```

- [ ] **Step 4: Commit**

```bash
git add src/screens/auth/ src/i18n/locales/ru.json
git commit -m "feat(auth): WelcomeScreen per Figma, social buttons disabled"
```

---

## Task 12: Register screen

**Files:**
- Create: `src/screens/auth/RegisterScreen.tsx`

Форма разделена на две карточки: обязательные и опциональные поля. При submit делаем единый `register` вызов. Если сервер вернул `username_taken` с suggestions – показываем их как чипы с возможностью кликнуть и подставить.

- [ ] **Step 1: Компонент**

```tsx
// src/screens/auth/RegisterScreen.tsx
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../../auth/useAuth";
import { ApiError, isApiError } from "../../api/errors";
import "./auth.css";

export default function RegisterScreen() {
  const { t } = useTranslation();
  const nav = useNavigate();
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

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrors({});
    setSuggestions([]);

    const fieldErrors: Record<string, string> = {};
    if (!firstName.trim()) fieldErrors.firstName = t("auth.register.name.first");
    if (!email.trim()) fieldErrors.email = t("auth.register.email");
    if (!password || password.length < 8) fieldErrors.password = "8+";
    if (Object.keys(fieldErrors).length > 0) {
      setErrors(fieldErrors);
      return;
    }

    const displayName = lastName.trim() ? `${firstName.trim()} ${lastName.trim()}` : firstName.trim();
    const dobAll = day && month && year;
    const dob = dobAll ? `${year.padStart(4, "0")}-${month.padStart(2, "0")}-${day.padStart(2, "0")}` : null;

    setSubmitting(true);
    try {
      await register({
        email: email.trim(),
        password,
        displayName,
        username: username.replace(/^@/, "").trim() || null,
        dateOfBirth: dob,
        dateOfBirthHidden: dobHidden,
        locale: null,
      } as any /* branded ts-rs type matches shape */);
      nav("/home", { replace: true });
    } catch (err) {
      if (isApiError(err) && err.code === "username_taken") {
        const sug = (err.extra?.suggestions as string[] | undefined) ?? [];
        setSuggestions(sug);
        setErrors({ username: t("auth.register.error.username_taken") });
      } else if (err instanceof ApiError) {
        setErrors({ _: err.message });
      } else {
        setErrors({ _: t("auth.error.generic") });
      }
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form className="auth-form" onSubmit={submit}>
      <button type="button" className="auth-back" onClick={() => nav("/welcome")}>{t("auth.register.back")}</button>
      <h1 className="auth-form__title">{t("auth.register.title")}</h1>

      <div className="auth-form__grid">
        <section className="auth-form__section">
          <div className="auth-form__section-title">{t("auth.register.required")}</div>

          <div className="auth-field">
            <label>{t("auth.register.name")}</label>
            <input value={firstName} onChange={(e) => setFirstName(e.target.value)} placeholder={t("auth.register.name.first")} />
            <input value={lastName} onChange={(e) => setLastName(e.target.value)} placeholder={t("auth.register.name.last")} />
            {errors.firstName && <div className="auth-field__error">{errors.firstName}</div>}
          </div>

          <div className="auth-field">
            <label>{t("auth.register.email")}</label>
            <input type="email" value={email} onChange={(e) => setEmail(e.target.value)} placeholder={t("auth.register.email.placeholder")} />
            {errors.email && <div className="auth-field__error">{errors.email}</div>}
          </div>

          <div className="auth-field">
            <label>{t("auth.register.password")}</label>
            <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} placeholder={t("auth.register.password.placeholder")} />
            {errors.password && <div className="auth-field__error">{errors.password}</div>}
          </div>
        </section>

        <section className="auth-form__section">
          <div className="auth-form__section-title">{t("auth.register.optional")}</div>

          <div className="auth-field">
            <label>{t("auth.register.dob")}</label>
            <div style={{ display: "flex", gap: 8 }}>
              <input placeholder={t("auth.register.dob.day")} value={day} onChange={(e) => setDay(e.target.value)} maxLength={2} />
              <input placeholder={t("auth.register.dob.month")} value={month} onChange={(e) => setMonth(e.target.value)} maxLength={2} />
              <input placeholder={t("auth.register.dob.year")} value={year} onChange={(e) => setYear(e.target.value)} maxLength={4} />
            </div>
            <label style={{ display: "flex", gap: 8, alignItems: "center", marginTop: 8 }}>
              <input type="checkbox" checked={dobHidden} onChange={(e) => setDobHidden(e.target.checked)} />
              {t("auth.register.dob.hide")}
            </label>
          </div>

          <div className="auth-field">
            <label>{t("auth.register.username")}</label>
            <input value={username} onChange={(e) => setUsername(e.target.value)} placeholder={t("auth.register.username.placeholder")} />
            <div className="auth-field__error" style={{ color: "var(--t3)", fontSize: 12 }}>{t("auth.register.username.hint")}</div>
            {errors.username && <div className="auth-field__error">{errors.username}</div>}
            {suggestions.length > 0 && (
              <div className="auth-suggestions">
                {suggestions.map((s) => (
                  <button type="button" key={s} onClick={() => setUsername(s)}>@{s}</button>
                ))}
              </div>
            )}
          </div>
        </section>
      </div>

      {errors._ && <div className="auth-hint" style={{ color: "#D12850" }}>{errors._}</div>}

      <div className="auth-submit-row">
        <button className="auth-submit" type="submit" disabled={submitting}>
          {t("auth.register.submit")}
        </button>
      </div>
    </form>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/screens/auth/RegisterScreen.tsx
git commit -m "feat(auth): RegisterScreen with username suggestions on 409"
```

---

## Task 13: Login screen

**Files:**
- Create: `src/screens/auth/LoginScreen.tsx`

- [ ] **Step 1: Компонент**

```tsx
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate, Link } from "react-router-dom";
import { useAuth } from "../../auth/useAuth";
import { ApiError, isApiError } from "../../api/errors";
import "./auth.css";

export default function LoginScreen() {
  const { t } = useTranslation();
  const nav = useNavigate();
  const { login } = useAuth();

  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      await login({ email: email.trim(), password } as any);
      nav("/home", { replace: true });
    } catch (err) {
      if (isApiError(err) && (err.code === "invalid_credentials" || err.status === 401)) {
        setError(t("auth.error.invalid_credentials"));
      } else {
        setError(t("auth.error.generic"));
      }
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form className="auth-form" onSubmit={submit} style={{ maxWidth: 440 }}>
      <button type="button" className="auth-back" onClick={() => nav("/welcome")}>{t("auth.register.back")}</button>
      <h1 className="auth-form__title">{t("auth.login.title")}</h1>

      <section className="auth-form__section">
        <div className="auth-field">
          <label>{t("auth.register.email")}</label>
          <input type="email" value={email} onChange={(e) => setEmail(e.target.value)} placeholder={t("auth.register.email.placeholder")} />
        </div>
        <div className="auth-field">
          <label>{t("auth.register.password")}</label>
          <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} />
        </div>
        {error && <div className="auth-field__error">{error}</div>}
      </section>

      <div className="auth-submit-row">
        <button className="auth-submit" type="submit" disabled={submitting}>{t("auth.login.submit")}</button>
      </div>

      <div className="auth-hint">
        <Link to="#" onClick={(e) => e.preventDefault()} style={{ color: "var(--t3)", pointerEvents: "none" }}>
          {t("auth.login.forgot")}
        </Link>
      </div>
    </form>
  );
}
```

Forgot password в альфе – disabled link.

- [ ] **Step 2: Commit**

```bash
git add src/screens/auth/LoginScreen.tsx
git commit -m "feat(auth): LoginScreen with friendly error mapping"
```

---

## Task 14: Роуты + AuthGuard

**Files:**
- Modify: `src/shell/routes.tsx`

- [ ] **Step 1: Добавить auth-роуты и защитить остальные**

```tsx
import { RouteObject, Navigate } from "react-router-dom";
import WelcomeScreen from "../screens/auth/WelcomeScreen";
import RegisterScreen from "../screens/auth/RegisterScreen";
import LoginScreen from "../screens/auth/LoginScreen";
import { AuthGuard } from "../auth/AuthGuard";
// ... все остальные импорты

const guarded = (el: JSX.Element) => <AuthGuard>{el}</AuthGuard>;

export const routes: RouteObject[] = [
  { path: "/welcome", element: <WelcomeScreen /> },
  { path: "/register", element: <RegisterScreen /> },
  { path: "/login", element: <LoginScreen /> },

  { path: "/", element: <Navigate to="/home" replace /> },
  { path: "/home", element: guarded(<HomePlaceholder />) },
  { path: "/categories", element: guarded(<ExploreScreen />) },
  { path: "/create", element: guarded(<CreateScreen />) },
  { path: "/people", element: guarded(<PeopleScreen />) },
  { path: "/notifications", element: guarded(<NotificationsScreen />) },
  { path: "/history", element: guarded(<HistoryScreen />) },
  { path: "/favorites", element: guarded(<FavoritesScreen />) },
  { path: "/folders", element: guarded(<FoldersScreen />) },
  { path: "/settings", element: guarded(<SettingsScreen />) },
  { path: "/profile", element: guarded(<ProfilePlaceholder />) },
  { path: "/profile/:username", element: guarded(<ProfilePlaceholder />) },
  { path: "/room/:id", element: guarded(<WatchRoomScreen onBack={() => window.history.back()} />) },
  { path: "/world", element: guarded(<World3DScreen onBack={() => window.history.back()} />) },
  { path: "*", element: <Navigate to="/home" replace /> },
];
```

- [ ] **Step 2: Commit**

```bash
git add src/shell/routes.tsx
git commit -m "feat(routes): protect app routes with AuthGuard, add /welcome /register /login"
```

---

## Task 15: Settings logout включён

**Files:**
- Modify: `src/screens/settings/SettingsScreen.tsx`

- [ ] **Step 1: Подключить logout**

```tsx
import { useAuth } from "../../auth/useAuth";

// внутри компонента:
const { status, logout } = useAuth();

// заменить <button disabled>:
<button className="settings__danger" disabled={status !== "authenticated"} onClick={logout}>
  {t("settings.account.logout")}
</button>
{status !== "authenticated" && (
  <p className="settings__hint">{t("settings.account.logout.hint")}</p>
)}
```

- [ ] **Step 2: Стиль для активного danger-button**

```css
.settings__danger:not(:disabled) {
  background: #D12850; color: #fff; border-color: #D12850; cursor: pointer;
}
.settings__danger:not(:disabled):hover { background: #A61F40; }
```

- [ ] **Step 3: Commit**

```bash
git add src/screens/settings/SettingsScreen.tsx src/App.css
git commit -m "feat(settings): enable logout when authenticated"
```

---

## Task 16: End-to-end прогон альфа-сценария auth

- [ ] **Step 1: Поднять бэкенд**

```bash
cd d:/Projects/Placebo/crates/placebo-api
cargo run
```

- [ ] **Step 2: Поднять frontend**

```bash
cd d:/Projects/Placebo
npm run tauri dev
```

- [ ] **Step 3: Прогон вручную**

1. Открывается Welcome (потому что `/home` защищён, редирект на `/welcome`).
2. "Попробовать без аккаунта" и все соцсети – disabled.
3. "Зарегистрироваться" → RegisterScreen.
4. Заполнить: имя "Тест", email "u1@test.com", пароль "secret12345", username "test_user".
5. Submit → редирект на `/home`, показывается HomePlaceholder.
6. Открыть Настройки → тема+язык работают, кнопка "Выйти" активна.
7. Нажать "Выйти" → редирект на `/welcome`.
8. Войти обратно: Login → email + password → Home.
9. Закрыть приложение.
10. Открыть снова → **остаёмся авторизованы** (token в keychain; AuthProvider на старте делает `/me`).
11. В Registr'е создать ещё один аккаунт с тем же username "test_user" → показывается "Этот юзернейм занят. Попробуйте:" + 3 чипа, клик на чип подставляет.

- [ ] **Step 4: Багфикс (если что-то не так)**

Исправить и сделать incremental commits.

- [ ] **Step 5: Обновить CLAUDE.md**

В секции Milestones поставить галочку M2.

- [ ] **Step 6: Push и PR**

```bash
git push -u origin feat/m2-auth
```

PR: `feat/m2-auth → main`, тайтл "M2: Auth flow (register/login/logout + username/dob)".

---

## Acceptance Criteria для Milestone 2

1. ✅ Пользователь может зарегистрироваться через Welcome → Register, заполнив обязательные и опциональные поля.
2. ✅ Пароли хешируются Argon2id на сервере.
3. ✅ `AuthResponse` содержит `username` и `displayName`, оба сохраняются в `users` таблице.
4. ✅ При конфликте username сервер возвращает 409 с `suggestions: [3 username]`; клиент показывает чипы.
5. ✅ Дата рождения опциональна и сохраняется как NaiveDate; галочка "Скрыть" определяет `date_of_birth_hidden`.
6. ✅ Логин работает, сессия опирается на существующий redis-based token flow.
7. ✅ `GET /api/v1/me` возвращает `MeResponse` с новыми полями.
8. ✅ Токен сохраняется в OS keychain через Tauri; после перезапуска приложения пользователь остаётся авторизованным.
9. ✅ Logout чистит token keychain + localStorage + Redis session.
10. ✅ Защищённые роуты (всё кроме /welcome /register /login) редиректят на /welcome если не авторизован.
11. ✅ `npm run gen-types` успешно экспортирует RegisterRequest, LoginRequest, AuthResponse, MeResponse, RefreshRequest, MessageResponse.
12. ✅ Все auth-строки проходят через i18n.
13. ✅ `cargo test --workspace` зелёный, включая новые тесты на username-валидацию.
14. ✅ Соцсети-кнопки и "Попробовать без аккаунта" – disabled с подсказкой "Скоро".

---

## Что идёт дальше

После approval M2 – переход к `2026-05-14-milestone-3-cameras-hls.md`: миграция схемы cameras под альфа-структуру, HLS-прокси в axum, seed-камеры, удаление рудиментов прототипа.
