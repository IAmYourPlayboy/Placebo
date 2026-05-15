# Milestone 6: Profile + Friends + Settings + Create Hub Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`.

**Goal:** Завершить "социальные" экраны альфы – Profile (по Figma), Friends (Люди) с поиском через pg_trgm, Create hub (4 тайла, работает "Комната для просмотра"), Avatar upload. Settings screen из M1 уже работает; здесь добавляем недостающие пункты (смена username / DOB).

**Architecture:**
- **Friendships**: новая таблица `friendships (user_id, friend_user_id, status, created_at)` со статусами `pending | accepted | declined`. В альфе сразу `accepted` (без двухстороннего подтверждения; post-альфа можно усложнить).
- **User search**: endpoint `GET /users/search?q=...` с pg_trgm по `username` + `display_name`. 20 результатов на страницу.
- **Public profile**: `GET /users/:username` возвращает `PublicProfile` (без email, без DOB если скрыт).
- **Avatar upload**:
  - В альфе – на сервер в `./uploads/avatars/{user_id}.jpg`, отдаётся через статический хэндлер `/uploads/...`.
  - Endpoint `POST /me/avatar` принимает multipart, сохраняет, обновляет `users.avatar_url`.
  - R2 интегрируется post-альфа, см. spec §13.3.
- **Create hub**: 4 тайла. Клик "Комната для просмотра" → открывается селектор камеры (модалка с `GET /cameras`) → создаётся комната. Остальные 3 – disabled с toast "Скоро".

**Tech Stack:** axum `Multipart`, sqlx, pg_trgm, React формы.

**Spec reference:** §6 (экраны), §2.3 (avatar fallback), §13.3 (поиск по обоим полям, приоритет TikTok-style).

**Зависимости:** M2 (users с username/dob), M3 (cameras), M5 (room creation).

---

## File Map

### Backend

- Create: `crates/placebo-api/migrations/012_friendships.sql`
- Modify: `crates/placebo-api/migrations/001_cameras.sql` – уже имеет pg_trgm; убедиться что он есть. Если нет – в 012 включить.
- Modify: `crates/placebo-shared/src/user.rs` – `PublicProfile`, `FriendSummary`, `UserSearchResult`.
- Modify: `crates/placebo-api/src/repositories/user_repo.rs` – search, get_public, friendships.
- Modify: `crates/placebo-api/src/services/user_service.rs` – бизнес-логика.
- Modify: `crates/placebo-api/src/handlers/users.rs` – `/users/search`, `/users/:username`.
- Create: `crates/placebo-api/src/handlers/friends.rs` – `/friends`, `/friends/:id`.
- Create: `crates/placebo-api/src/handlers/avatar.rs` – `/me/avatar`.
- Modify: `crates/placebo-api/Cargo.toml` – `axum` with `multipart` feature, `image = "0.25"` (для валидации/resize опционально).

### Frontend

- Create: `src/screens/profile/ProfileScreen.tsx` (заменяет ProfilePlaceholder).
- Create: `src/screens/profile/profile.css`.
- Create: `src/screens/create/CreateHubScreen.tsx` (заменяет старый CreateScreen).
- Create: `src/screens/create/CameraPickerModal.tsx`.
- Create: `src/screens/create/create.css`.
- Create: `src/screens/people/PeopleScreen.tsx` (заменяет skeleton).
- Create: `src/screens/people/people.css`.
- Create: `src/api/users.ts`, `src/api/friends.ts`, `src/api/avatar.ts`.
- Modify: `src/screens/settings/SettingsScreen.tsx` – добавить секцию "Редактировать профиль" (ссылка на ProfileEdit – можно inline в Settings).
- Modify: `src/shell/routes.tsx`.
- Modify: `src/i18n/locales/ru.json`.

### Delete

- `src/screens/CreateScreen.tsx`
- `src/screens/FriendsScreen.tsx`
- `src/screens/ProfileScreen.tsx`
- `src/screens/profile/ProfilePlaceholder.tsx`
- `src/screens/skeletons/PeopleScreen.tsx`

---

## Task 1: Ветка

```bash
git -C d:/Projects/Placebo checkout main && git pull
git -C d:/Projects/Placebo checkout -b feat/m6-profile-friends-create
```

---

## Task 2: Миграция 012 – friendships

- [ ] **Step 1:**

```sql
-- 012_friendships.sql
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE friendships (
    user_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    friend_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status         TEXT NOT NULL DEFAULT 'accepted' CHECK (status IN ('pending','accepted','declined')),
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, friend_user_id),
    CHECK (user_id <> friend_user_id)
);

CREATE INDEX idx_friendships_friend ON friendships (friend_user_id);

-- Trigram indexes for user search (username already case-insensitive via normalized column)
CREATE INDEX IF NOT EXISTS idx_users_display_name_trgm ON users USING GIN (display_name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_users_username_trgm     ON users USING GIN (username gin_trgm_ops);
```

- [ ] **Step 2:**

```bash
cd d:/Projects/Placebo/crates/placebo-api
cargo sqlx migrate run
git add migrations/012_friendships.sql
git commit -m "feat(db): 012 friendships + trigram indexes"
```

---

## Task 3: DTO в placebo-shared

**Files:** `crates/placebo-shared/src/user.rs`

- [ ] **Step 1:** Добавить типы

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct PublicProfile {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub is_premium: bool,
    pub date_of_birth: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub friend_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct FriendSummary {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub since: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct UserSearchResult {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    /// Higher = better (0..1 trigram similarity blended with exact-match boost)
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct UserSearchResponse {
    pub items: Vec<UserSearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub username: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub date_of_birth_hidden: Option<bool>,
}
```

- [ ] **Step 2:**

```bash
npm run gen-types
git add crates/placebo-shared/src/user.rs src/types/api/
git commit -m "feat(shared): PublicProfile, FriendSummary, UserSearchResult DTOs"
```

---

## Task 4: user_repo – поиск и friendships

**Files:** `crates/placebo-api/src/repositories/user_repo.rs`

- [ ] **Step 1:** Search

```rust
pub async fn search(pool: &PgPool, q: &str, limit: i64) -> sqlx::Result<Vec<SearchRow>> {
    // Simplified TikTok-ish ranking: exact username > username prefix > trigram similarity over both fields.
    sqlx::query_as::<_, SearchRow>(
        "SELECT id, username, display_name, avatar_url, \
           GREATEST( \
              CASE WHEN lower(username) = lower($1) THEN 1.0 ELSE 0 END, \
              CASE WHEN lower(username) LIKE lower($1) || '%' THEN 0.85 ELSE 0 END, \
              similarity(username,     $1), \
              similarity(display_name, $1) \
           ) AS score \
         FROM users \
         WHERE username     % $1 OR display_name % $1 \
            OR lower(username) LIKE lower($1) || '%' \
         ORDER BY score DESC \
         LIMIT $2",
    )
    .bind(q)
    .bind(limit)
    .fetch_all(pool)
    .await
}

#[derive(sqlx::FromRow)]
pub struct SearchRow {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub score: f32,
}
```

- [ ] **Step 2:** Public profile

```rust
pub async fn by_username(pool: &PgPool, username: &str) -> sqlx::Result<Option<User>> {
    sqlx::query_as::<_, User>(
        "SELECT id, email, username, display_name, avatar_url, locale, is_premium, premium_until, \
                password_hash, email_verified, date_of_birth, date_of_birth_hidden, created_at \
         FROM users WHERE username_normalized = lower($1)",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}

pub async fn friend_count(pool: &PgPool, user_id: Uuid) -> sqlx::Result<u32> {
    let c: (i64,) = sqlx::query_as(
        "SELECT count(*)::bigint FROM friendships WHERE user_id = $1 AND status = 'accepted'",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(c.0 as u32)
}
```

- [ ] **Step 3:** Friendships CRUD

```rust
pub async fn add_friend(pool: &PgPool, user_id: Uuid, friend_id: Uuid) -> sqlx::Result<()> {
    // Symmetric insert (both directions) so queries are simpler.
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO friendships (user_id, friend_user_id, status) VALUES ($1, $2, 'accepted') ON CONFLICT DO NOTHING")
        .bind(user_id).bind(friend_id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO friendships (user_id, friend_user_id, status) VALUES ($1, $2, 'accepted') ON CONFLICT DO NOTHING")
        .bind(friend_id).bind(user_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn remove_friend(pool: &PgPool, user_id: Uuid, friend_id: Uuid) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM friendships WHERE (user_id = $1 AND friend_user_id = $2) OR (user_id = $2 AND friend_user_id = $1)")
        .bind(user_id).bind(friend_id).execute(pool).await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
pub struct FriendRow {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub since: chrono::DateTime<chrono::Utc>,
}

pub async fn list_friends(pool: &PgPool, user_id: Uuid) -> sqlx::Result<Vec<FriendRow>> {
    sqlx::query_as::<_, FriendRow>(
        "SELECT u.id, u.username, u.display_name, u.avatar_url, f.created_at AS since \
         FROM friendships f JOIN users u ON u.id = f.friend_user_id \
         WHERE f.user_id = $1 AND f.status = 'accepted' \
         ORDER BY u.display_name",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}
```

- [ ] **Step 4:** Update profile

```rust
pub async fn update_profile(
    pool: &PgPool,
    user_id: Uuid,
    req: &placebo_shared::user::UpdateProfileRequest,
) -> sqlx::Result<()> {
    if let Some(name) = &req.display_name {
        sqlx::query("UPDATE users SET display_name = $1, updated_at = NOW() WHERE id = $2")
            .bind(name.trim()).bind(user_id).execute(pool).await?;
    }
    if let Some(u) = &req.username {
        // The service layer checks uniqueness before calling us.
        sqlx::query("UPDATE users SET username = $1, updated_at = NOW() WHERE id = $2")
            .bind(u.trim()).bind(user_id).execute(pool).await?;
    }
    if let Some(d) = req.date_of_birth {
        sqlx::query("UPDATE users SET date_of_birth = $1, updated_at = NOW() WHERE id = $2")
            .bind(d).bind(user_id).execute(pool).await?;
    }
    if let Some(h) = req.date_of_birth_hidden {
        sqlx::query("UPDATE users SET date_of_birth_hidden = $1, updated_at = NOW() WHERE id = $2")
            .bind(h).bind(user_id).execute(pool).await?;
    }
    Ok(())
}

pub async fn set_avatar_url(pool: &PgPool, user_id: Uuid, url: &str) -> sqlx::Result<()> {
    sqlx::query("UPDATE users SET avatar_url = $1, updated_at = NOW() WHERE id = $2")
        .bind(url).bind(user_id).execute(pool).await?;
    Ok(())
}
```

- [ ] **Step 5:** Commit

```bash
git add crates/placebo-api/src/repositories/user_repo.rs
git commit -m "feat(users): search/by_username/friendships/update helpers"
```

---

## Task 5: User service + handlers

**Files:**
- `crates/placebo-api/src/services/user_service.rs`
- `crates/placebo-api/src/handlers/users.rs`
- `crates/placebo-api/src/handlers/friends.rs`

- [ ] **Step 1:** Service

```rust
// user_service.rs
use placebo_shared::user::{
    FriendSummary, PublicProfile, UpdateProfileRequest, UserSearchResponse, UserSearchResult,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::repositories::user_repo;

pub async fn search(pool: &PgPool, q: &str) -> Result<UserSearchResponse, AppError> {
    let q = q.trim();
    if q.len() < 2 {
        return Ok(UserSearchResponse { items: vec![] });
    }
    let rows = user_repo::search(pool, q, 20).await?;
    Ok(UserSearchResponse {
        items: rows.into_iter().map(|r| UserSearchResult {
            id: r.id, username: r.username, display_name: r.display_name,
            avatar_url: r.avatar_url, score: r.score,
        }).collect(),
    })
}

pub async fn public_profile(pool: &PgPool, username: &str) -> Result<PublicProfile, AppError> {
    let u = user_repo::by_username(pool, username).await?
        .ok_or(AppError::NotFound("user".into()))?;
    let count = user_repo::friend_count(pool, u.id).await?;
    let dob = if u.date_of_birth_hidden { None } else { u.date_of_birth };
    Ok(PublicProfile {
        id: u.id, username: u.username, display_name: u.display_name,
        avatar_url: u.avatar_url, is_premium: u.is_premium,
        date_of_birth: dob, created_at: u.created_at, friend_count: count,
    })
}

pub async fn list_friends(pool: &PgPool, user_id: Uuid) -> Result<Vec<FriendSummary>, AppError> {
    Ok(user_repo::list_friends(pool, user_id).await?
        .into_iter()
        .map(|r| FriendSummary {
            id: r.id, username: r.username, display_name: r.display_name,
            avatar_url: r.avatar_url, since: r.since,
        }).collect())
}

pub async fn add_friend_by_username(
    pool: &PgPool, user_id: Uuid, username: &str,
) -> Result<FriendSummary, AppError> {
    let friend = user_repo::by_username(pool, username).await?
        .ok_or(AppError::NotFound("user".into()))?;
    if friend.id == user_id {
        return Err(AppError::BadRequest("cannot add yourself".into()));
    }
    user_repo::add_friend(pool, user_id, friend.id).await?;
    Ok(FriendSummary {
        id: friend.id, username: friend.username, display_name: friend.display_name,
        avatar_url: friend.avatar_url, since: chrono::Utc::now(),
    })
}

pub async fn remove_friend(
    pool: &PgPool, user_id: Uuid, friend_id: Uuid,
) -> Result<(), AppError> {
    user_repo::remove_friend(pool, user_id, friend_id).await?;
    Ok(())
}

pub async fn update_profile(
    pool: &PgPool, user_id: Uuid, req: UpdateProfileRequest,
) -> Result<(), AppError> {
    if let Some(u) = &req.username {
        if !user_repo::username_available(pool, u).await? {
            return Err(AppError::UsernameTaken { suggestions: vec![] });
        }
    }
    user_repo::update_profile(pool, user_id, &req).await?;
    Ok(())
}
```

- [ ] **Step 2:** Users handler

```rust
// handlers/users.rs
use axum::{
    extract::{Path, Query, State},
    routing::{get, patch},
    Json, Router,
};
use placebo_shared::user::{PublicProfile, UpdateProfileRequest, UserSearchResponse};
use serde::Deserialize;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::services::user_service;

#[derive(Deserialize)]
struct SearchQ { q: String }

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users/search", get(search))
        .route("/users/:username", get(public))
        .route("/me/profile", patch(update_me))
}

async fn search(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(q): Query<SearchQ>,
) -> Result<Json<UserSearchResponse>, AppError> {
    Ok(Json(user_service::search(&state.db, &q.q).await?))
}

async fn public(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(username): Path<String>,
) -> Result<Json<PublicProfile>, AppError> {
    Ok(Json(user_service::public_profile(&state.db, &username).await?))
}

async fn update_me(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<(), AppError> {
    user_service::update_profile(&state.db, auth.id, req).await
}
```

- [ ] **Step 3:** Friends handler

```rust
// handlers/friends.rs
use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use placebo_shared::user::FriendSummary;
use serde::Deserialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::services::user_service;

#[derive(Deserialize)]
struct AddBody { username: String }

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/friends", get(list).post(add))
        .route("/friends/:id", delete(remove))
}

async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<FriendSummary>>, AppError> {
    Ok(Json(user_service::list_friends(&state.db, auth.id).await?))
}

async fn add(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<AddBody>,
) -> Result<Json<FriendSummary>, AppError> {
    Ok(Json(user_service::add_friend_by_username(&state.db, auth.id, &body.username).await?))
}

async fn remove(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    user_service::remove_friend(&state.db, auth.id, id).await
}
```

- [ ] **Step 4:** Смонтировать роутеры и commit

```bash
# в lib.rs добавить .merge(handlers::users::router()).merge(handlers::friends::router())
cargo check
git add crates/placebo-api/src/
git commit -m "feat(api): /users/search, /users/:username, /friends CRUD, /me/profile"
```

---

## Task 6: Avatar upload endpoint

**Files:** `crates/placebo-api/src/handlers/avatar.rs`

- [ ] **Step 1:** Multipart

Включить feature `multipart` в `axum`:

```toml
axum = { version = "0.7", features = ["ws", "macros", "multipart"] }
```

Код:

```rust
use axum::{
    extract::{Multipart, State},
    routing::post,
    Json, Router,
};
use serde::Serialize;
use std::{fs, path::PathBuf};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::repositories::user_repo;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvatarResponse { pub avatar_url: String }

pub fn router() -> Router<AppState> {
    Router::new().route("/me/avatar", post(upload))
}

async fn upload(
    State(state): State<AppState>,
    auth: AuthUser,
    mut mp: Multipart,
) -> Result<Json<AvatarResponse>, AppError> {
    let mut bytes: Option<Vec<u8>> = None;
    let mut ext: String = "jpg".into();

    while let Some(field) = mp.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        if field.name() == Some("file") {
            if let Some(ct) = field.content_type() {
                ext = match ct {
                    "image/png" => "png".into(),
                    "image/jpeg" | "image/jpg" => "jpg".into(),
                    "image/webp" => "webp".into(),
                    other => return Err(AppError::BadRequest(format!("unsupported content type: {other}"))),
                };
            }
            let data = field.bytes().await.map_err(|e| AppError::BadRequest(e.to_string()))?;
            if data.len() > 5 * 1024 * 1024 {
                return Err(AppError::BadRequest("max 5MB".into()));
            }
            bytes = Some(data.to_vec());
        }
    }

    let bytes = bytes.ok_or_else(|| AppError::BadRequest("missing file field".into()))?;
    let filename = format!("{}.{ext}", auth.id);
    let dir = PathBuf::from("./uploads/avatars");
    fs::create_dir_all(&dir).map_err(|e| AppError::Internal(e.to_string()))?;
    let path = dir.join(&filename);
    fs::write(&path, &bytes).map_err(|e| AppError::Internal(e.to_string()))?;

    let public_url = format!("/uploads/avatars/{}", filename);
    user_repo::set_avatar_url(&state.db, auth.id, &public_url).await?;

    Ok(Json(AvatarResponse { avatar_url: public_url }))
}
```

- [ ] **Step 2:** Static-hosting `/uploads`

Добавить в сборку router:

```rust
.nest_service("/uploads", ServeDir::new("./uploads"))
```

- [ ] **Step 3:** Commit

```bash
git add crates/placebo-api/src/handlers/avatar.rs crates/placebo-api/Cargo.toml crates/placebo-api/src/lib.rs
git commit -m "feat(api): /me/avatar multipart upload to ./uploads/avatars"
```

---

## Task 7: Frontend API wrappers

**Files:** `src/api/users.ts`, `src/api/friends.ts`, `src/api/avatar.ts`

```ts
// users.ts
import { apiRequest } from "./client";
import type { UserSearchResponse } from "../types/api/UserSearchResponse";
import type { PublicProfile } from "../types/api/PublicProfile";
import type { UpdateProfileRequest } from "../types/api/UpdateProfileRequest";

export const searchUsers = (q: string) =>
  apiRequest<UserSearchResponse>(`/users/search?q=${encodeURIComponent(q)}`);

export const getPublicProfile = (username: string) =>
  apiRequest<PublicProfile>(`/users/${encodeURIComponent(username)}`);

export const updateMyProfile = (req: UpdateProfileRequest) =>
  apiRequest<void>("/me/profile", { method: "PATCH", body: req });
```

```ts
// friends.ts
import { apiRequest } from "./client";
import type { FriendSummary } from "../types/api/FriendSummary";

export const listFriends = () => apiRequest<FriendSummary[]>("/friends");
export const addFriend = (username: string) =>
  apiRequest<FriendSummary>("/friends", { method: "POST", body: { username } });
export const removeFriend = (id: string) =>
  apiRequest<void>(`/friends/${id}`, { method: "DELETE" });
```

```ts
// avatar.ts
import { loadToken } from "../auth/tokenStorage";

const BASE = import.meta.env.VITE_API_BASE_URL ?? "http://localhost:3001/api/v1";

export async function uploadAvatar(file: File): Promise<{ avatarUrl: string }> {
  const token = await loadToken();
  const fd = new FormData();
  fd.append("file", file);
  const res = await fetch(`${BASE}/me/avatar`, {
    method: "POST",
    headers: token ? { Authorization: `Bearer ${token}` } : {},
    body: fd,
  });
  if (!res.ok) throw new Error(`upload failed: ${res.status}`);
  return res.json();
}
```

Commit:

```bash
git add src/api/users.ts src/api/friends.ts src/api/avatar.ts
git commit -m "feat(api): users/friends/avatar client wrappers"
```

---

## Task 8: PeopleScreen (Люди)

**Files:** `src/screens/people/PeopleScreen.tsx`, `people.css`

- [ ] **Step 1:** Компонент

```tsx
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { listFriends, addFriend, removeFriend } from "../../api/friends";
import { searchUsers } from "../../api/users";
import type { FriendSummary } from "../../types/api/FriendSummary";
import type { UserSearchResult } from "../../types/api/UserSearchResult";
import { useTabs } from "../../shell/tabs/useTabs";
import "./people.css";

export default function PeopleScreen() {
  const { t } = useTranslation();
  const nav = useNavigate();
  const { openTab } = useTabs();
  const [friends, setFriends] = useState<FriendSummary[]>([]);
  const [q, setQ] = useState("");
  const [results, setResults] = useState<UserSearchResult[]>([]);

  const refresh = () => listFriends().then(setFriends);
  useEffect(() => { refresh(); }, []);

  useEffect(() => {
    if (q.trim().length < 2) { setResults([]); return; }
    const h = window.setTimeout(() => {
      searchUsers(q.trim()).then((r) => setResults(r.items));
    }, 300);
    return () => window.clearTimeout(h);
  }, [q]);

  const openProfile = (username: string) => openTab(`/profile/${username}`, `@${username}`);

  return (
    <div className="people">
      <h2>{t("people.title")}</h2>

      <section className="people__search">
        <input
          placeholder={t("people.search.placeholder")}
          value={q}
          onChange={(e) => setQ(e.target.value)}
        />
        {results.length > 0 && (
          <ul className="people__results">
            {results.map((u) => (
              <li key={u.id}>
                <button className="people__who" onClick={() => openProfile(u.username)}>
                  <span className="people__avatar">{u.displayName[0]?.toUpperCase()}</span>
                  <span className="people__who-name">
                    <b>@{u.username}</b> <span>{u.displayName}</span>
                  </span>
                </button>
                <button className="people__add" onClick={async () => { await addFriend(u.username); refresh(); }}>
                  +
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>

      <section>
        <h3>{t("people.friends")} ({friends.length})</h3>
        {friends.length === 0 ? (
          <p style={{ color: "var(--t3)" }}>{t("people.empty.hint")}</p>
        ) : (
          <ul className="people__list">
            {friends.map((f) => (
              <li key={f.id}>
                <button className="people__who" onClick={() => openProfile(f.username)}>
                  <span className="people__avatar">{f.displayName[0]?.toUpperCase()}</span>
                  <span className="people__who-name">
                    <b>@{f.username}</b> <span>{f.displayName}</span>
                  </span>
                </button>
                <button className="people__remove" onClick={async () => { await removeFriend(f.id); refresh(); }}>
                  {t("people.remove")}
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
```

- [ ] **Step 2:** CSS

```css
.people { padding: 24px 32px; max-width: 720px; }
.people h2 { font-size: 22px; margin-bottom: 16px; }
.people__search input { width: 100%; padding: 10px 12px; border-radius: 8px; border: 1px solid var(--border); background: var(--bg-2); }
.people__results, .people__list { list-style: none; padding: 0; margin: 12px 0 0; display: flex; flex-direction: column; gap: 6px; }
.people__results li, .people__list li { display: flex; align-items: center; gap: 8px; padding: 8px; border-radius: 10px; }
.people__results li:hover, .people__list li:hover { background: var(--bg-2); }
.people__who { flex: 1; display: flex; align-items: center; gap: 10px; background: transparent; border: 0; text-align: left; cursor: pointer; color: var(--t1); }
.people__avatar { width: 36px; height: 36px; border-radius: 50%; background: var(--accent); color: #fff; display: grid; place-items: center; font-weight: 700; }
.people__who-name { display: flex; flex-direction: column; }
.people__who-name b { font-size: 14px; }
.people__who-name span { color: var(--t3); font-size: 12px; }
.people__add, .people__remove { background: var(--t1); color: var(--bg); border: 0; border-radius: 8px; padding: 6px 10px; cursor: pointer; font-size: 13px; }
.people__remove { background: transparent; color: var(--t2); border: 1px solid var(--border); }
```

- [ ] **Step 3:** i18n

```json
{
  "people.title": "Люди",
  "people.friends": "Друзья",
  "people.search.placeholder": "Поиск по @юзернейму или имени",
  "people.remove": "Убрать"
}
```

(`people.empty.hint` уже есть из M1.)

- [ ] **Step 4:** Route + удалить skeleton

```bash
rm src/screens/skeletons/PeopleScreen.tsx src/screens/FriendsScreen.tsx
```

В `routes.tsx`:

```tsx
import PeopleScreen from "../screens/people/PeopleScreen";
// заменить строку People
{ path: "/people", element: guarded(<PeopleScreen />) },
```

- [ ] **Step 5:** Commit

```bash
git add -A
git commit -m "feat(people): PeopleScreen with debounced search + add/remove friend"
```

---

## Task 9: ProfileScreen

**Files:** `src/screens/profile/ProfileScreen.tsx`, `profile.css`

- [ ] **Step 1:** Компонент

```tsx
import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { getPublicProfile } from "../../api/users";
import { addFriend } from "../../api/friends";
import { useAuth } from "../../auth/useAuth";
import type { PublicProfile } from "../../types/api/PublicProfile";
import "./profile.css";

export default function ProfileScreen() {
  const { t } = useTranslation();
  const { username } = useParams();
  const { user } = useAuth();
  const [profile, setProfile] = useState<PublicProfile | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!username) return;
    getPublicProfile(username).then(setProfile).catch((e) => setError(e.message));
  }, [username]);

  if (error) return <div style={{ padding: 32, color: "#D12850" }}>{error}</div>;
  if (!profile) return <div style={{ padding: 32 }}>{t("app.loading")}</div>;

  const isSelf = user?.username === profile.username;

  return (
    <div className="profile">
      <div className="profile__head">
        <div className="profile__avatar" style={profile.avatarUrl ? { backgroundImage: `url(${profile.avatarUrl})` } : {}}>
          {!profile.avatarUrl && profile.displayName[0]?.toUpperCase()}
        </div>
        <div className="profile__identity">
          <div className="profile__name">
            {profile.displayName}
            {profile.isPremium && <span className="profile__premium">★</span>}
          </div>
          <div className="profile__username">@{profile.username}</div>
          <div className="profile__meta">
            <span>{profile.friendCount} {t("profile.friends")}</span>
          </div>
        </div>
        {!isSelf && (
          <div className="profile__actions">
            <button className="profile__follow" onClick={() => addFriend(profile.username)}>
              {t("profile.add_friend")}
            </button>
            <button className="profile__message" disabled>{t("profile.message")}</button>
          </div>
        )}
      </div>

      <div className="profile__body">
        <div className="profile__section">
          <div className="profile__section-title">{t("profile.media")}</div>
          <div className="profile__empty">{t("profile.media.empty")}</div>
        </div>
        <div className="profile__section">
          <div className="profile__section-title">{t("profile.achievements")}</div>
          <div className="profile__empty">{t("profile.achievements.empty")}</div>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2:** CSS

```css
.profile { padding: 24px 32px; }
.profile__head { display: flex; gap: 24px; align-items: center; padding-bottom: 16px; border-bottom: 1px solid var(--border); }
.profile__avatar {
  width: 120px; height: 120px; border-radius: 50%;
  background: var(--accent); color: #fff;
  display: grid; place-items: center;
  font-size: 48px; font-weight: 700;
  background-size: cover; background-position: center;
}
.profile__identity { flex: 1; }
.profile__name { font-size: 22px; font-weight: 700; color: var(--t1); }
.profile__premium { color: var(--accent); margin-left: 6px; }
.profile__username { color: var(--t3); font-size: 14px; }
.profile__meta { color: var(--t2); font-size: 13px; margin-top: 8px; }
.profile__actions { display: flex; gap: 8px; }
.profile__follow { background: var(--accent); color: #fff; border: 0; border-radius: 999px; padding: 10px 20px; cursor: pointer; }
.profile__message { background: transparent; border: 1px solid var(--border); border-radius: 999px; padding: 10px 20px; }
.profile__body { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-top: 16px; }
.profile__section { border: 1px solid var(--border); border-radius: 12px; padding: 16px; background: var(--bg); }
.profile__section-title { font-size: 14px; color: var(--t2); margin-bottom: 8px; }
.profile__empty { color: var(--t3); font-size: 13px; }
```

- [ ] **Step 3:** i18n + route

```json
{
  "profile.friends": "друзей",
  "profile.add_friend": "Добавить",
  "profile.message": "Написать",
  "profile.media": "Все медиа",
  "profile.media.empty": "Скоро появятся посты.",
  "profile.achievements": "Достижения",
  "profile.achievements.empty": "Пока нет."
}
```

`routes.tsx`: `{ path: "/profile/:username", element: guarded(<ProfileScreen />) }`; `/profile` – редиректим на `/profile/<current user username>`:

```tsx
function MyProfileRedirect() {
  const { user } = useAuth();
  return <Navigate to={user ? `/profile/${user.username}` : "/welcome"} replace />;
}
// { path: "/profile", element: guarded(<MyProfileRedirect />) }
```

Удалить `ProfilePlaceholder.tsx` и `src/screens/ProfileScreen.tsx`.

- [ ] **Step 4:** Commit

```bash
rm -f src/screens/profile/ProfilePlaceholder.tsx src/screens/ProfileScreen.tsx
git add -A
git commit -m "feat(profile): ProfileScreen per Figma with add-friend action"
```

---

## Task 10: Create Hub

**Files:** `src/screens/create/CreateHubScreen.tsx`, `CameraPickerModal.tsx`, `create.css`

- [ ] **Step 1:** Hub

```tsx
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate, useSearchParams } from "react-router-dom";
import { useToast } from "../../components/ui/Toast";
import CameraPickerModal from "./CameraPickerModal";
import { createRoom } from "../../api/rooms";
import "./create.css";

export default function CreateHubScreen() {
  const { t } = useTranslation();
  const nav = useNavigate();
  const [params] = useSearchParams();
  const preCam = params.get("camera");
  const { show } = useToast();
  const [pickerOpen, setPickerOpen] = useState(Boolean(preCam));

  const onCameraChosen = async (cameraId: string) => {
    const room = await createRoom({ cameraId });
    nav(`/room/${room.id}`);
  };

  const tiles = [
    { key: "film",   title: t("create.film"),    enabled: true, onClick: () => setPickerOpen(true) },
    { key: "call",   title: t("create.call"),    enabled: false },
    { key: "stream", title: t("create.stream"),  enabled: false },
    { key: "games",  title: t("create.games"),   enabled: false },
  ];

  return (
    <div className="create">
      <h1>{t("create.title")}</h1>
      <div className="create__grid">
        {tiles.map((tile) => (
          <button
            key={tile.key}
            className={"create__tile" + (tile.enabled ? "" : " create__tile--disabled")}
            onClick={() => tile.enabled ? tile.onClick?.() : show(t("categories.coming_soon"))}
          >
            <span>{tile.title}</span>
          </button>
        ))}
      </div>
      <p className="create__note">{t("create.hint")}</p>
      {pickerOpen && (
        <CameraPickerModal
          initialCameraId={preCam ?? undefined}
          onPick={onCameraChosen}
          onClose={() => setPickerOpen(false)}
        />
      )}
    </div>
  );
}
```

- [ ] **Step 2:** CameraPickerModal

```tsx
import { useEffect, useState } from "react";
import { listCameras } from "../../api/cameras";
import type { CameraSummary } from "../../types/api/CameraSummary";
import { useTranslation } from "react-i18next";

type Props = {
  initialCameraId?: string;
  onPick: (id: string) => void;
  onClose: () => void;
};

export default function CameraPickerModal({ initialCameraId, onPick, onClose }: Props) {
  const { t } = useTranslation();
  const [items, setItems] = useState<CameraSummary[]>([]);
  const [q, setQ] = useState("");

  useEffect(() => {
    listCameras({ limit: 50, q: q || undefined }).then((r) => setItems(r.items));
  }, [q]);

  useEffect(() => {
    if (initialCameraId && items.length) {
      const c = items.find((x) => x.id === initialCameraId);
      if (c) onPick(c.id);
    }
  }, [initialCameraId, items, onPick]);

  return (
    <div className="picker-backdrop" onClick={onClose}>
      <div className="picker" onClick={(e) => e.stopPropagation()}>
        <div className="picker__head">
          <h2>{t("create.picker.title")}</h2>
          <button onClick={onClose}>✕</button>
        </div>
        <input
          className="picker__search"
          placeholder={t("create.picker.search")}
          value={q}
          onChange={(e) => setQ(e.target.value)}
        />
        <ul className="picker__list">
          {items.map((c) => (
            <li key={c.id}>
              <button onClick={() => onPick(c.id)}>
                <span className="picker__name">{c.name}</span>
                <span className="picker__where">{c.city}, {c.country}</span>
              </button>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
```

- [ ] **Step 3:** CSS

```css
.create { padding: 32px; text-align: center; }
.create h1 { font-size: 28px; margin-bottom: 24px; }
.create__grid { display: grid; grid-template-columns: repeat(2, 260px); gap: 24px; justify-content: center; }
.create__tile {
  aspect-ratio: 1/0.7; border-radius: 16px; border: 0;
  background: var(--bg-3); color: var(--t1); font-weight: 700;
  cursor: pointer; font-size: 16px;
}
.create__tile--disabled { opacity: 0.6; }
.create__note { color: var(--t3); margin-top: 24px; font-size: 13px; }

.picker-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: grid; place-items: center; z-index: 1000; }
.picker { background: var(--bg); color: var(--t1); width: 520px; max-height: 80vh; border-radius: 16px; padding: 16px; display: flex; flex-direction: column; }
.picker__head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
.picker__head button { background: transparent; border: 0; cursor: pointer; font-size: 18px; color: var(--t2); }
.picker__search { padding: 10px; border-radius: 8px; border: 1px solid var(--border); background: var(--bg-2); margin-bottom: 12px; }
.picker__list { list-style: none; padding: 0; margin: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 6px; }
.picker__list li button { display: flex; flex-direction: column; align-items: flex-start; width: 100%; padding: 10px; background: var(--bg-2); border: 0; border-radius: 10px; cursor: pointer; color: var(--t1); text-align: left; }
.picker__name { font-weight: 600; }
.picker__where { color: var(--t3); font-size: 13px; }
```

- [ ] **Step 4:** i18n + route

```json
{
  "create.title": "Что вы хотите создать?",
  "create.film": "Комнату для просмотра фильма",
  "create.call": "Комнату для созвона",
  "create.stream": "Онлайн-трансляцию",
  "create.games": "Комнату для совместных игр с друзьями",
  "create.hint": "Нужно чтобы эти блоки будто из пустоты вылетали и останавливались...",
  "create.picker.title": "Выберите камеру",
  "create.picker.search": "Поиск по названию"
}
```

`routes.tsx`:

```tsx
import CreateHubScreen from "../screens/create/CreateHubScreen";
// заменить CreateScreen:
{ path: "/create", element: guarded(<CreateHubScreen />) },
```

Удалить `src/screens/CreateScreen.tsx`.

- [ ] **Step 5:** Commit

```bash
rm -f src/screens/CreateScreen.tsx
git add -A
git commit -m "feat(create): Create hub with 4 tiles + camera picker modal"
```

---

## Task 11: Settings расширяется (редактирование профиля)

**Files:** `src/screens/settings/SettingsScreen.tsx`

Добавить секцию "Редактировать профиль" с полями display_name, username, date_of_birth, hide-toggle и avatar uploader.

- [ ] **Step 1:** Расширение

```tsx
// В SettingsScreen, новая секция:
const [displayName, setDisplayName] = useState(user?.displayName ?? "");
const [username, setUsername] = useState(user?.username ?? "");
// ... (аналогично для DOB, hidden)

const saveProfile = async () => {
  await updateMyProfile({
    displayName: displayName !== user?.displayName ? displayName : null,
    username: username !== user?.username ? username : null,
    // etc.
  } as any);
  // refetch /me
  refetchMe();
};

<section className="settings__group">
  <h2>{t("settings.profile.title")}</h2>
  <label>{t("auth.register.name")}</label>
  <input value={displayName} onChange={(e) => setDisplayName(e.target.value)} />
  <label>{t("auth.register.username")}</label>
  <input value={username} onChange={(e) => setUsername(e.target.value)} />
  <button onClick={saveProfile}>{t("settings.profile.save")}</button>

  <label className="settings__avatar-field">
    <span>{t("settings.profile.avatar")}</span>
    <input type="file" accept="image/*" onChange={async (e) => {
      const f = e.target.files?.[0];
      if (!f) return;
      await uploadAvatar(f);
      refetchMe();
    }} />
  </label>
</section>
```

- [ ] **Step 2:** i18n

```json
{
  "settings.profile.title": "Профиль",
  "settings.profile.save": "Сохранить",
  "settings.profile.avatar": "Фото профиля"
}
```

- [ ] **Step 3:** Commit

```bash
git add -A
git commit -m "feat(settings): inline edit profile (display name, username, avatar)"
```

---

## Task 12: E2E

- [ ] **Step 1: Запустить**

```bash
cd crates/placebo-api && cargo run &
cd d:/Projects/Placebo && npm run tauri dev
```

- [ ] **Step 2: Проверки**

1. Profile по клику "@username" в PeopleScreen работает, показывает данные.
2. Search /people возвращает пользователей (создать 2-3 тест-аккаунта заранее).
3. Add friend → появляется в списке.
4. CreateHub: клик "Комната для просмотра" → модалка с камерами → клик на камеру → создаётся комната и редиректит.
5. Settings: поменять username → сохраняется; загрузить avatar → отображается на профиле.

- [ ] **Step 3: Commit + PR**

```bash
git push -u origin feat/m6-profile-friends-create
```

---

## Acceptance Criteria

1. ✅ Friendships таблица + trigram-индексы применяются.
2. ✅ `GET /users/search?q=...` возвращает результаты с score, отсортированные.
3. ✅ `GET /users/:username` возвращает PublicProfile; DOB скрыт если `date_of_birth_hidden = true`.
4. ✅ `POST /friends { username }` добавляет симметричную запись; `DELETE /friends/:id` удаляет обе стороны.
5. ✅ `PATCH /me/profile` обновляет display_name, username (с проверкой уникальности), dob.
6. ✅ `POST /me/avatar` multipart: сохраняет в `./uploads/avatars/<user_id>.<ext>`, обновляет `users.avatar_url`, отдаёт URL.
7. ✅ `/uploads/avatars/<file>` доступен статически.
8. ✅ ProfileScreen отображает Figma-layout (аватар, премиум-звезда, счётчик друзей, кнопка "Добавить").
9. ✅ PeopleScreen ищет через 300ms debounce и добавляет/удаляет друзей.
10. ✅ CreateHub: 4 тайла, работает "Комната для просмотра" через CameraPickerModal → создаёт комнату.
11. ✅ Settings: inline edit профиля + avatar upload.
12. ✅ Все удалённые legacy-файлы ушли; ссылки в `routes.tsx` обновлены.

---

## Дальше

M7: интеграционный прогон всего главного сценария, багфикс, сборка .msi, раздача, документация для тестеров.
