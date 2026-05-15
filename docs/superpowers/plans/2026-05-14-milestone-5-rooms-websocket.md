# Milestone 5: Rooms + WebSocket + Chat Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use `- [ ]` syntax.

**Goal:** Завершить главный сценарий альфы: пользователь A создаёт комнату из World3D, получает invite-ссылку, B переходит по ней, оба видят ту же камеру, host переключает – у guest'а меняется, оба пишут в чате, счётчик viewers показывает 2. Это самый сложный milestone альфы.

**Architecture:**
- **Комнаты в PostgreSQL**: `rooms`, `room_members` (миграция уже есть частично – проверить 003_rooms и расширить).
- **Invite-код**: base58 8 символов, храним в `rooms.invite_code`; endpoint `GET /rooms/by-invite/:code`.
- **WebSocket**: один WS на пользователя `wss://.../ws?token=...`. Протокол в `placebo-shared/src/ws.rs` с ts-rs exports (ClientMessage/ServerMessage enums, `#[serde(tag="type", content="payload")]`).
- **Redis pub/sub**: канал `room:{uuid}` для broadcast. Каждая WS-задача подписывается на каналы комнат, в которых юзер.
- **Presence**: Redis `SADD room:{uuid}:members {user_id}` с периодическим touch через ping; cleanup при dropping WS.
- **Viewer count**: Redis `SADD camera:{id}:viewers {user_id}` с TTL через sorted-set по timestamp; публикуется через `CameraViewersUpdate` при изменении.
- **Chat**: 50 последних в Redis `LIST`, история загружается при `RoomJoin` как часть `RoomStateSnapshot`.
- **Host-wins**: только создатель может менять камеру.
- **Frontend**: Room screen по Figma ("Канал" макет) – плеер по центру, чат справа. Плеер использует HLS через тот же `/api/v1/hls-proxy/:slug`.

**Tech Stack:** axum WS, tokio::sync::broadcast для локального fan-out, deadpool-redis для pub/sub, ts-rs, hls.js.

**Spec reference:** разделы 4.1–4.5 спеки; 13.3 (invite как 8-символьный base58).

**Зависимости:** M2 (auth), M3 (cameras), M4 (World3D "Смотреть вместе" кнопка).

---

## File Map

### Backend

- Modify: `crates/placebo-api/migrations/003_rooms.sql` – **не трогать** если уже применено. Создать 011_rooms_alpha.sql с invite_code + расширениями.
- Create: `crates/placebo-api/migrations/011_rooms_alpha.sql`
- Modify: `crates/placebo-shared/src/room.rs` – Room DTO + RoomStateSnapshot + invite-типы.
- Create: `crates/placebo-shared/src/ws.rs` – ClientMessage, ServerMessage с ts-rs.
- Modify: `crates/placebo-api/src/repositories/room_repo.rs`
- Modify: `crates/placebo-api/src/services/room_service.rs` – create_room, join_by_invite, change_camera.
- Modify: `crates/placebo-api/src/handlers/rooms.rs` – REST: POST /rooms, GET /rooms/:id, POST /rooms/:id/invite, GET /rooms/by-invite/:code.
- Modify: `crates/placebo-api/src/websocket/mod.rs` – полный WS-handler.
- Create: `crates/placebo-api/src/websocket/session.rs` – per-connection state + loop.
- Create: `crates/placebo-api/src/websocket/presence.rs` – Redis presence helper.
- Create: `crates/placebo-api/src/websocket/chat.rs` – Redis chat history.
- Create: `crates/placebo-api/src/websocket/viewers.rs` – viewer counter.

### Frontend

- Create: `src/api/rooms.ts`
- Create: `src/ws/WebSocketProvider.tsx`
- Create: `src/ws/useWebSocket.ts`
- Create: `src/ws/useRoom.ts`
- Create: `src/ws/useCameraViewers.ts`
- Create: `src/screens/room/RoomScreen.tsx` (по Figma "Канал")
- Create: `src/screens/room/room.css`
- Create: `src/screens/room/RoomChat.tsx`
- Create: `src/screens/room/RoomPlayer.tsx` (HLS без 3D, просто видео-плеер)
- Modify: `src/shell/routes.tsx` – `/room/:id`.
- Modify: `src/App.tsx` – WebSocketProvider.
- Modify: `src/screens/world/CameraDetailPanel.tsx` – "Смотреть вместе" теперь реально создаёт комнату.
- Modify: `src/i18n/locales/ru.json`.

---

## Task 1: Ветка

```bash
git -C d:/Projects/Placebo checkout main && git -C d:/Projects/Placebo pull
git -C d:/Projects/Placebo checkout -b feat/m5-rooms-websocket
```

---

## Task 2: Миграция 011 – rooms для альфы

**Files:** `crates/placebo-api/migrations/011_rooms_alpha.sql`

Сначала проверить, что уже есть в 003_rooms.sql:

```bash
cat d:/Projects/Placebo/crates/placebo-api/migrations/003_rooms.sql
```

Если в 003 уже есть поля host_user_id, camera_id – расширяем, не дублируем.

- [ ] **Step 1: Миграция**

```sql
-- 011_rooms_alpha.sql
-- Alpha room fields: invite code, current camera, host semantics.

ALTER TABLE rooms
    ADD COLUMN IF NOT EXISTS invite_code    TEXT UNIQUE,
    ADD COLUMN IF NOT EXISTS current_camera_id UUID REFERENCES cameras(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS host_user_id   UUID REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS closed_at      TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_rooms_invite_code ON rooms (invite_code) WHERE invite_code IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_rooms_host        ON rooms (host_user_id);

-- Ensure room_members exists
CREATE TABLE IF NOT EXISTS room_members (
    room_id    UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (room_id, user_id)
);
CREATE INDEX IF NOT EXISTS idx_room_members_user ON room_members (user_id);
```

- [ ] **Step 2: Apply + commit**

```bash
cd d:/Projects/Placebo/crates/placebo-api
cargo sqlx migrate run
psql -d placebo_dev -c "\d rooms" | head -30
git add migrations/011_rooms_alpha.sql
git commit -m "feat(db): 011 adds rooms.invite_code + host + room_members"
```

---

## Task 3: DTO – Room + RoomStateSnapshot в placebo-shared

**Files:** `crates/placebo-shared/src/room.rs`

- [ ] **Step 1: Types**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[cfg(feature = "export-types")]
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct RoomSummary {
    pub id: Uuid,
    pub host_user_id: Uuid,
    pub invite_code: String,
    pub current_camera_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct RoomMemberSummary {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub from_user_id: Uuid,
    pub from_username: String,
    pub text: String,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct RoomStateSnapshot {
    pub room: RoomSummary,
    pub members: Vec<RoomMemberSummary>,
    pub recent_chat: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomRequest {
    pub camera_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(rename_all = "camelCase")]
pub struct InviteResponse {
    pub invite_code: String,
    pub invite_url: String,
}
```

- [ ] **Step 2: Commit**

```bash
npm run gen-types
git add crates/placebo-shared/src/room.rs src/types/api/
git commit -m "feat(shared): Room DTOs + RoomStateSnapshot + ChatMessage (ts-rs)"
```

---

## Task 4: WebSocket protocol types

**Files:** `crates/placebo-shared/src/ws.rs`, подключить в `lib.rs`.

- [ ] **Step 1: Types**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[cfg(feature = "export-types")]
use ts_rs::TS;

use crate::room::{ChatMessage, RoomMemberSummary, RoomStateSnapshot};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum ClientMessage {
    RoomJoin { room_id: Uuid },
    RoomLeave { room_id: Uuid },
    RoomChangeCamera { room_id: Uuid, camera_id: Uuid },
    RoomChatSend { room_id: Uuid, text: String },
    CameraWatchStart { camera_id: Uuid },
    CameraWatchStop { camera_id: Uuid },
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "export-types", derive(TS), ts(export, export_to = "../../bindings/"))]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum ServerMessage {
    RoomStateSnapshot(RoomStateSnapshot),
    RoomCameraChanged {
        room_id: Uuid,
        camera_id: Uuid,
        by_user_id: Uuid,
    },
    RoomChatMessage {
        room_id: Uuid,
        message: ChatMessage,
    },
    RoomMemberJoined {
        room_id: Uuid,
        member: RoomMemberSummary,
    },
    RoomMemberLeft {
        room_id: Uuid,
        user_id: Uuid,
    },
    CameraViewersUpdate {
        camera_id: Uuid,
        count: u32,
    },
    Error {
        code: String,
        message: String,
    },
    Pong { at: DateTime<Utc> },
}
```

- [ ] **Step 2: Export + commit**

```bash
npm run gen-types
git add crates/placebo-shared/src/ws.rs crates/placebo-shared/src/lib.rs src/types/api/
git commit -m "feat(shared): WebSocket ClientMessage/ServerMessage enums (ts-rs)"
```

---

## Task 5: Room repository

**Files:** `crates/placebo-api/src/repositories/room_repo.rs`

- [ ] **Step 1: Helpers**

```rust
use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

const BASE58: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

fn generate_invite_code(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len).map(|_| BASE58[rng.gen_range(0..BASE58.len())] as char).collect()
}

pub async fn create_room(
    pool: &PgPool,
    host_user_id: Uuid,
    camera_id: Uuid,
) -> sqlx::Result<(Uuid, String)> {
    for _ in 0..5 {
        let code = generate_invite_code(8);
        let res: sqlx::Result<(Uuid,)> = sqlx::query_as(
            "INSERT INTO rooms (host_user_id, invite_code, current_camera_id) \
             VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(host_user_id)
        .bind(&code)
        .bind(camera_id)
        .fetch_one(pool)
        .await;
        match res {
            Ok((id,)) => return Ok((id, code)),
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => continue,
            Err(e) => return Err(e),
        }
    }
    Err(sqlx::Error::Protocol("failed to generate unique invite code".into()))
}

pub async fn by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<RoomRow>> {
    sqlx::query_as::<_, RoomRow>(
        "SELECT id, host_user_id, invite_code, current_camera_id, created_at, closed_at \
         FROM rooms WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn by_invite(pool: &PgPool, code: &str) -> sqlx::Result<Option<RoomRow>> {
    sqlx::query_as::<_, RoomRow>(
        "SELECT id, host_user_id, invite_code, current_camera_id, created_at, closed_at \
         FROM rooms WHERE invite_code = $1 AND closed_at IS NULL",
    )
    .bind(code)
    .fetch_optional(pool)
    .await
}

pub async fn set_current_camera(pool: &PgPool, room_id: Uuid, camera_id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE rooms SET current_camera_id = $1 WHERE id = $2")
        .bind(camera_id)
        .bind(room_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_member(pool: &PgPool, room_id: Uuid, user_id: Uuid) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO room_members (room_id, user_id, last_seen) VALUES ($1, $2, NOW()) \
         ON CONFLICT (room_id, user_id) DO UPDATE SET last_seen = NOW()",
    )
    .bind(room_id).bind(user_id).execute(pool).await?;
    Ok(())
}

pub async fn remove_member(pool: &PgPool, room_id: Uuid, user_id: Uuid) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM room_members WHERE room_id = $1 AND user_id = $2")
        .bind(room_id).bind(user_id).execute(pool).await?;
    Ok(())
}

pub async fn members(pool: &PgPool, room_id: Uuid) -> sqlx::Result<Vec<MemberRow>> {
    sqlx::query_as::<_, MemberRow>(
        "SELECT u.id AS user_id, u.username, u.display_name, m.joined_at \
         FROM room_members m JOIN users u ON u.id = m.user_id \
         WHERE m.room_id = $1 ORDER BY m.joined_at",
    )
    .bind(room_id)
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RoomRow {
    pub id: Uuid,
    pub host_user_id: Uuid,
    pub invite_code: String,
    pub current_camera_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MemberRow {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub joined_at: DateTime<Utc>,
}
```

- [ ] **Step 2: Unit-test invite-code uniqueness**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn invite_code_is_8_chars_base58() {
        let c = generate_invite_code(8);
        assert_eq!(c.len(), 8);
        for ch in c.chars() { assert!(BASE58.contains(&(ch as u8))); }
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add crates/placebo-api/src/repositories/room_repo.rs
git commit -m "feat(rooms): room_repo with invite-code gen, members table helpers"
```

---

## Task 6: Room service

**Files:** `crates/placebo-api/src/services/room_service.rs`

- [ ] **Step 1: Logic**

```rust
use placebo_shared::room::{
    ChatMessage, CreateRoomRequest, InviteResponse, RoomMemberSummary, RoomStateSnapshot, RoomSummary,
};
use uuid::Uuid;
use sqlx::PgPool;
use deadpool_redis::Pool as RedisPool;

use crate::error::AppError;
use crate::repositories::room_repo;
use crate::websocket::chat;

pub async fn create_room(
    pg: &PgPool,
    host_user_id: Uuid,
    req: &CreateRoomRequest,
) -> Result<RoomSummary, AppError> {
    let (id, code) = room_repo::create_room(pg, host_user_id, req.camera_id).await?;
    room_repo::add_member(pg, id, host_user_id).await?;
    let row = room_repo::by_id(pg, id).await?.ok_or_else(|| AppError::Internal("room vanished".into()))?;
    Ok(RoomSummary {
        id: row.id,
        host_user_id: row.host_user_id,
        invite_code: code,
        current_camera_id: row.current_camera_id,
        created_at: row.created_at,
        closed_at: row.closed_at,
    })
}

pub async fn snapshot(
    pg: &PgPool,
    redis: &RedisPool,
    room_id: Uuid,
) -> Result<RoomStateSnapshot, AppError> {
    let row = room_repo::by_id(pg, room_id).await?.ok_or(AppError::NotFound("room".into()))?;
    let members = room_repo::members(pg, room_id).await?
        .into_iter()
        .map(|m| RoomMemberSummary {
            user_id: m.user_id,
            username: m.username,
            display_name: m.display_name,
            joined_at: m.joined_at,
        })
        .collect();
    let recent_chat = chat::recent(redis, room_id).await.unwrap_or_default();
    Ok(RoomStateSnapshot {
        room: RoomSummary {
            id: row.id,
            host_user_id: row.host_user_id,
            invite_code: row.invite_code,
            current_camera_id: row.current_camera_id,
            created_at: row.created_at,
            closed_at: row.closed_at,
        },
        members,
        recent_chat,
    })
}

pub async fn join_by_invite(
    pg: &PgPool,
    code: &str,
    user_id: Uuid,
) -> Result<Uuid, AppError> {
    let room = room_repo::by_invite(pg, code).await?
        .ok_or(AppError::NotFound("invite".into()))?;
    room_repo::add_member(pg, room.id, user_id).await?;
    Ok(room.id)
}

pub async fn host_change_camera(
    pg: &PgPool,
    room_id: Uuid,
    camera_id: Uuid,
    actor_user_id: Uuid,
) -> Result<(), AppError> {
    let room = room_repo::by_id(pg, room_id).await?.ok_or(AppError::NotFound("room".into()))?;
    if room.host_user_id != actor_user_id {
        return Err(AppError::Forbidden("only host may change camera".into()));
    }
    room_repo::set_current_camera(pg, room_id, camera_id).await?;
    Ok(())
}

pub fn build_invite_response(base_url: &str, code: &str) -> InviteResponse {
    InviteResponse {
        invite_code: code.to_string(),
        invite_url: format!("{base_url}/r/{code}"),
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add crates/placebo-api/src/services/room_service.rs
git commit -m "feat(rooms): room_service create/join/host-change-camera/snapshot"
```

---

## Task 7: Chat/presence/viewers helpers

**Files:**
- `crates/placebo-api/src/websocket/chat.rs`
- `crates/placebo-api/src/websocket/presence.rs`
- `crates/placebo-api/src/websocket/viewers.rs`

- [ ] **Step 1: chat.rs**

```rust
use anyhow::Result;
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::Pool as RedisPool;
use placebo_shared::room::ChatMessage;
use uuid::Uuid;

const MAX: usize = 50;

fn key(room: Uuid) -> String { format!("room:{room}:chat") }

pub async fn append(redis: &RedisPool, room: Uuid, msg: &ChatMessage) -> Result<()> {
    let mut conn = redis.get().await?;
    let payload = serde_json::to_string(msg)?;
    let _: () = conn.lpush(key(room), &payload).await?;
    let _: () = conn.ltrim(key(room), 0, (MAX as isize) - 1).await?;
    Ok(())
}

pub async fn recent(redis: &RedisPool, room: Uuid) -> Result<Vec<ChatMessage>> {
    let mut conn = redis.get().await?;
    let rows: Vec<String> = conn.lrange(key(room), 0, (MAX as isize) - 1).await?;
    let mut out: Vec<ChatMessage> = rows.into_iter()
        .filter_map(|s| serde_json::from_str::<ChatMessage>(&s).ok())
        .collect();
    out.reverse(); // LPUSH + LRANGE → newest first; we want oldest→newest
    Ok(out)
}
```

- [ ] **Step 2: presence.rs**

```rust
use anyhow::Result;
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::Pool as RedisPool;
use uuid::Uuid;

fn key(room: Uuid) -> String { format!("room:{room}:members") }

pub async fn touch(redis: &RedisPool, room: Uuid, user: Uuid) -> Result<()> {
    let mut conn = redis.get().await?;
    let _: () = conn.sadd(key(room), user.to_string()).await?;
    let _: () = conn.expire(key(room), 60).await?;
    Ok(())
}

pub async fn leave(redis: &RedisPool, room: Uuid, user: Uuid) -> Result<()> {
    let mut conn = redis.get().await?;
    let _: () = conn.srem(key(room), user.to_string()).await?;
    Ok(())
}

pub async fn count(redis: &RedisPool, room: Uuid) -> Result<u32> {
    let mut conn = redis.get().await?;
    let n: u32 = conn.scard(key(room)).await?;
    Ok(n)
}
```

- [ ] **Step 3: viewers.rs**

```rust
use anyhow::Result;
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::Pool as RedisPool;
use uuid::Uuid;

fn key(camera: Uuid) -> String { format!("camera:{camera}:viewers") }

pub async fn add(redis: &RedisPool, camera: Uuid, user: Uuid) -> Result<u32> {
    let mut conn = redis.get().await?;
    let _: () = conn.sadd(key(camera), user.to_string()).await?;
    let _: () = conn.expire(key(camera), 120).await?;
    Ok(conn.scard(key(camera)).await?)
}

pub async fn remove(redis: &RedisPool, camera: Uuid, user: Uuid) -> Result<u32> {
    let mut conn = redis.get().await?;
    let _: () = conn.srem(key(camera), user.to_string()).await?;
    Ok(conn.scard(key(camera)).await.unwrap_or(0))
}
```

- [ ] **Step 4: Commit**

```bash
git add crates/placebo-api/src/websocket/
git commit -m "feat(ws): chat/presence/viewers helpers on Redis"
```

---

## Task 8: WebSocket handler

**Files:** `crates/placebo-api/src/websocket/mod.rs`, `crates/placebo-api/src/websocket/session.rs`

- [ ] **Step 1: WS-router**

```rust
// websocket/mod.rs
use axum::{
    extract::{Query, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::Deserialize;

use crate::app_state::AppState;
use crate::services::auth_service;
use crate::error::AppError;

pub mod chat;
pub mod presence;
pub mod viewers;
mod session;

#[derive(Deserialize)]
pub struct WsQuery { token: String }

pub fn router() -> Router<AppState> {
    Router::new().route("/ws", get(upgrade))
}

async fn upgrade(
    State(state): State<AppState>,
    Query(q): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_service::user_from_token(&state.redis, &state.db, &q.token)
        .await
        .map_err(|_| AppError::Unauthorized("invalid token".into()))?;
    Ok(ws.on_upgrade(move |socket| session::run(state, user, socket)))
}
```

Нужен helper в `auth_service::user_from_token` – если его нет, добавить:

```rust
pub async fn user_from_token(redis: &RedisPool, pg: &PgPool, token: &str) -> Result<crate::extractors::auth::AuthUser, AppError> {
    // Resolve token → user_id via Redis (same as AuthUser extractor) → load user row
    // The exact impl depends on your existing session format; inline here for reference.
    unimplemented!("factor from existing extractor logic")
}
```

- [ ] **Step 2: Session loop**

```rust
// websocket/session.rs
use axum::extract::ws::{Message, WebSocket};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use placebo_shared::room::{ChatMessage, RoomMemberSummary};
use placebo_shared::ws::{ClientMessage, ServerMessage};
use std::collections::HashSet;
use tokio::sync::mpsc;
use tracing::warn;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::extractors::auth::AuthUser;
use crate::repositories::room_repo;
use crate::services::room_service;
use super::{chat, presence, viewers};

pub async fn run(state: AppState, user: AuthUser, mut socket: WebSocket) {
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();
    let mut subscribed_rooms: HashSet<Uuid> = HashSet::new();
    let mut watched_camera: Option<Uuid> = None;

    // Writer task
    let (mut sink, mut stream) = socket.split();
    let writer = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = match serde_json::to_string(&msg) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if sink.send(Message::Text(json)).await.is_err() { break; }
        }
    });

    // TODO: Redis pub/sub subscription per room. For alpha we rely on a simple
    // broadcast-channel registry kept in AppState (see Task 9).

    while let Some(Ok(msg)) = stream.next().await {
        match msg {
            Message::Text(txt) => {
                let Ok(cm) = serde_json::from_str::<ClientMessage>(&txt) else {
                    let _ = tx.send(ServerMessage::Error { code: "bad_message".into(), message: "Invalid JSON".into() });
                    continue;
                };
                handle_client(&state, &user, &tx, &mut subscribed_rooms, &mut watched_camera, cm).await;
            }
            Message::Close(_) => break,
            Message::Ping(p) => { /* axum replies Pong automatically */ let _ = p; }
            _ => {}
        }
    }

    // Cleanup
    for room_id in subscribed_rooms {
        let _ = presence::leave(&state.redis, room_id, user.id).await;
        state.rooms_bus.broadcast(room_id, ServerMessage::RoomMemberLeft { room_id, user_id: user.id });
    }
    if let Some(cam) = watched_camera {
        let count = viewers::remove(&state.redis, cam, user.id).await.unwrap_or(0);
        // best-effort broadcast
        state.cameras_bus.broadcast(cam, ServerMessage::CameraViewersUpdate { camera_id: cam, count });
    }
    writer.abort();
}

async fn handle_client(
    state: &AppState,
    user: &AuthUser,
    tx: &mpsc::UnboundedSender<ServerMessage>,
    subscribed: &mut HashSet<Uuid>,
    watched: &mut Option<Uuid>,
    cm: ClientMessage,
) {
    match cm {
        ClientMessage::RoomJoin { room_id } => {
            if let Err(e) = room_repo::add_member(&state.db, room_id, user.id).await {
                warn!(?e, "add_member failed");
                let _ = tx.send(ServerMessage::Error { code: "join_failed".into(), message: "Cannot join".into() });
                return;
            }
            let _ = presence::touch(&state.redis, room_id, user.id).await;
            subscribed.insert(room_id);
            // Snapshot
            match room_service::snapshot(&state.db, &state.redis, room_id).await {
                Ok(snap) => {
                    let member = RoomMemberSummary {
                        user_id: user.id, username: user.username.clone(), display_name: user.display_name.clone(),
                        joined_at: Utc::now(),
                    };
                    let _ = tx.send(ServerMessage::RoomStateSnapshot(snap));
                    state.rooms_bus.broadcast(room_id, ServerMessage::RoomMemberJoined { room_id, member });
                    // Subscribe this session to the bus for this room
                    state.rooms_bus.subscribe_session(room_id, tx.clone());
                }
                Err(_) => {
                    let _ = tx.send(ServerMessage::Error { code: "snapshot_failed".into(), message: "snapshot error".into() });
                }
            }
        }
        ClientMessage::RoomLeave { room_id } => {
            subscribed.remove(&room_id);
            let _ = presence::leave(&state.redis, room_id, user.id).await;
            state.rooms_bus.unsubscribe_session(room_id, tx);
            state.rooms_bus.broadcast(room_id, ServerMessage::RoomMemberLeft { room_id, user_id: user.id });
        }
        ClientMessage::RoomChangeCamera { room_id, camera_id } => {
            match room_service::host_change_camera(&state.db, room_id, camera_id, user.id).await {
                Ok(()) => {
                    state.rooms_bus.broadcast(room_id, ServerMessage::RoomCameraChanged {
                        room_id, camera_id, by_user_id: user.id,
                    });
                }
                Err(e) => {
                    let _ = tx.send(ServerMessage::Error { code: "forbidden".into(), message: e.to_string() });
                }
            }
        }
        ClientMessage::RoomChatSend { room_id, text } => {
            let text = text.trim().to_string();
            if text.is_empty() || text.len() > 2000 { return; }
            let msg = ChatMessage {
                from_user_id: user.id,
                from_username: user.username.clone(),
                text,
                at: Utc::now(),
            };
            let _ = chat::append(&state.redis, room_id, &msg).await;
            state.rooms_bus.broadcast(room_id, ServerMessage::RoomChatMessage { room_id, message: msg });
        }
        ClientMessage::CameraWatchStart { camera_id } => {
            if *watched != Some(camera_id) {
                if let Some(old) = watched.take() {
                    let c = viewers::remove(&state.redis, old, user.id).await.unwrap_or(0);
                    state.cameras_bus.broadcast(old, ServerMessage::CameraViewersUpdate { camera_id: old, count: c });
                }
                *watched = Some(camera_id);
                state.cameras_bus.subscribe_session(camera_id, tx.clone());
            }
            let count = viewers::add(&state.redis, camera_id, user.id).await.unwrap_or(1);
            state.cameras_bus.broadcast(camera_id, ServerMessage::CameraViewersUpdate { camera_id, count });
        }
        ClientMessage::CameraWatchStop { camera_id } => {
            if *watched == Some(camera_id) { *watched = None; }
            state.cameras_bus.unsubscribe_session(camera_id, tx);
            let count = viewers::remove(&state.redis, camera_id, user.id).await.unwrap_or(0);
            state.cameras_bus.broadcast(camera_id, ServerMessage::CameraViewersUpdate { camera_id, count });
        }
        ClientMessage::Ping => {
            let _ = tx.send(ServerMessage::Pong { at: Utc::now() });
        }
    }
}
```

- [ ] **Step 3: Commit (WIP – bus реализуется в Task 9)**

```bash
git add crates/placebo-api/src/websocket/
git commit -m "feat(ws): session loop handling ClientMessage variants (bus pending)"
```

---

## Task 9: In-process bus для комнат и камер

**Files:**
- Create: `crates/placebo-api/src/bus.rs`
- Modify: `crates/placebo-api/src/app_state.rs` – добавить `rooms_bus`, `cameras_bus`.

В альфе один API-процесс, поэтому достаточно **локального bus'а**, не Redis pub/sub. Redis pub/sub подключим post-альфа при scale-out. Это упрощение сильно сокращает код. Фиксирую в spec.

- [ ] **Step 1: Bus**

```rust
// src/bus.rs
use dashmap::DashMap;
use placebo_shared::ws::ServerMessage;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct Bus {
    inner: Arc<DashMap<Uuid, Vec<UnboundedSender<ServerMessage>>>>,
}

impl Bus {
    pub fn new() -> Self { Self::default() }

    pub fn subscribe_session(&self, topic: Uuid, tx: UnboundedSender<ServerMessage>) {
        self.inner.entry(topic).or_default().push(tx);
    }

    pub fn unsubscribe_session(&self, topic: Uuid, tx: &UnboundedSender<ServerMessage>) {
        if let Some(mut list) = self.inner.get_mut(&topic) {
            list.retain(|existing| !same_sender(existing, tx));
        }
    }

    pub fn broadcast(&self, topic: Uuid, msg: ServerMessage) {
        if let Some(list) = self.inner.get(&topic) {
            for tx in list.iter() {
                let _ = tx.send(msg.clone());
            }
        }
    }
}

fn same_sender(
    a: &UnboundedSender<ServerMessage>,
    b: &UnboundedSender<ServerMessage>,
) -> bool {
    // Tokio's UnboundedSender has no pointer equality, but we can compare
    // via the same underlying channel's `same_channel` method.
    a.same_channel(b)
}
```

- [ ] **Step 2: Добавить dashmap**

`crates/placebo-api/Cargo.toml`:

```toml
dashmap = "6"
```

- [ ] **Step 3: app_state**

```rust
// app_state.rs
pub struct AppState {
    pub db: PgPool,
    pub redis: RedisPool,
    pub config: Config,
    pub rooms_bus: crate::bus::Bus,
    pub cameras_bus: crate::bus::Bus,
}
```

И при создании:

```rust
AppState {
    db, redis, config,
    rooms_bus: crate::bus::Bus::new(),
    cameras_bus: crate::bus::Bus::new(),
}
```

- [ ] **Step 4: Commit**

```bash
git add crates/placebo-api/src/bus.rs crates/placebo-api/src/app_state.rs crates/placebo-api/src/lib.rs crates/placebo-api/Cargo.toml
git commit -m "feat(ws): in-process Bus for room and camera fan-out"
```

**В спеку:** добавить в разделе "Отложенные решения":

> 12. **WebSocket fan-out через Redis pub/sub** – альфа работает на одном процессе, поэтому bus live в памяти. При переходе на несколько API-узлов (post-альфа) заменить `Bus` на Redis pub/sub с теми же методами.

---

## Task 10: REST handlers для rooms

**Files:** `crates/placebo-api/src/handlers/rooms.rs`

- [ ] **Step 1: Router**

```rust
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use placebo_shared::room::{CreateRoomRequest, InviteResponse, RoomStateSnapshot, RoomSummary};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::services::room_service;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/rooms", post(create))
        .route("/rooms/:id", get(get_one))
        .route("/rooms/:id/invite", post(get_invite))
        .route("/rooms/by-invite/:code", get(by_invite))
}

async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateRoomRequest>,
) -> Result<Json<RoomSummary>, AppError> {
    Ok(Json(room_service::create_room(&state.db, auth.id, &req).await?))
}

async fn get_one(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<RoomStateSnapshot>, AppError> {
    Ok(Json(room_service::snapshot(&state.db, &state.redis, id).await?))
}

async fn get_invite(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<InviteResponse>, AppError> {
    let snap = room_service::snapshot(&state.db, &state.redis, id).await?;
    Ok(Json(room_service::build_invite_response(
        &state.config.public_base_url,
        &snap.room.invite_code,
    )))
}

async fn by_invite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(code): Path<String>,
) -> Result<Json<RoomSummary>, AppError> {
    let room_id = room_service::join_by_invite(&state.db, &code, auth.id).await?;
    let snap = room_service::snapshot(&state.db, &state.redis, room_id).await?;
    Ok(Json(snap.room))
}
```

Добавить поле `public_base_url` в `Config` (читать из `PUBLIC_BASE_URL` env, default `http://localhost:1420`).

- [ ] **Step 2: Смонтировать в router**

`.merge(handlers::rooms::router())` + `.merge(websocket::router())`.

- [ ] **Step 3: Smoke**

```bash
cargo run -p placebo-api &
sleep 3
# логин
TOKEN=$(curl -s -X POST http://localhost:3001/api/v1/auth/login -H "content-type: application/json" -d '{"email":"u1@test.com","password":"secret12345"}' | jq -r .token)
# получить камеру
CAMERA_ID=$(curl -s http://localhost:3001/api/v1/cameras | jq -r '.items[0].id')
# создать комнату
curl -s -X POST http://localhost:3001/api/v1/rooms -H "Authorization: Bearer $TOKEN" -H "content-type: application/json" -d "{\"cameraId\":\"$CAMERA_ID\"}" | jq .
kill %1
```

Expected: JSON с `id`, `inviteCode` (8 base58).

- [ ] **Step 4: Commit**

```bash
git add crates/placebo-api/src/handlers/rooms.rs crates/placebo-api/src/config.rs crates/placebo-api/src/lib.rs
git commit -m "feat(api): rooms REST – create, snapshot, invite, by-invite"
```

---

## Task 11: Frontend – WebSocketProvider

**Files:** `src/ws/WebSocketProvider.tsx`, `src/ws/useWebSocket.ts`, `src/ws/useRoom.ts`, `src/ws/useCameraViewers.ts`

- [ ] **Step 1: Provider**

```tsx
// src/ws/WebSocketProvider.tsx
import { createContext, useCallback, useEffect, useRef, useState, ReactNode } from "react";
import { loadToken } from "../auth/tokenStorage";
import type { ServerMessage } from "../types/api/ServerMessage";
import type { ClientMessage } from "../types/api/ClientMessage";

type Listener = (msg: ServerMessage) => void;

type Api = {
  send(msg: ClientMessage): void;
  subscribe(l: Listener): () => void;
  connected: boolean;
};

export const WsContext = createContext<Api | null>(null);

const WS_BASE = import.meta.env.VITE_WS_BASE_URL ?? "ws://localhost:3001/ws";

export function WebSocketProvider({ children }: { children: ReactNode }) {
  const wsRef = useRef<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);
  const listenersRef = useRef<Set<Listener>>(new Set());
  const backoffRef = useRef(1000);

  const connect = useCallback(async () => {
    const token = await loadToken();
    if (!token) return;
    const url = `${WS_BASE}?token=${encodeURIComponent(token)}`;
    const ws = new WebSocket(url);
    wsRef.current = ws;
    ws.onopen = () => { setConnected(true); backoffRef.current = 1000; };
    ws.onmessage = (ev) => {
      try {
        const msg = JSON.parse(ev.data) as ServerMessage;
        listenersRef.current.forEach((l) => l(msg));
      } catch { /* malformed */ }
    };
    ws.onclose = () => {
      setConnected(false);
      wsRef.current = null;
      setTimeout(() => connect(), backoffRef.current);
      backoffRef.current = Math.min(backoffRef.current * 2, 30000);
    };
    ws.onerror = () => ws.close();
  }, []);

  useEffect(() => {
    connect();
    return () => { wsRef.current?.close(); };
  }, [connect]);

  // Ping every 25s to keep connection alive
  useEffect(() => {
    const t = window.setInterval(() => {
      wsRef.current?.send(JSON.stringify({ type: "Ping" } as ClientMessage));
    }, 25_000);
    return () => window.clearInterval(t);
  }, []);

  const send = useCallback((msg: ClientMessage) => {
    wsRef.current?.send(JSON.stringify(msg));
  }, []);

  const subscribe = useCallback((l: Listener) => {
    listenersRef.current.add(l);
    return () => { listenersRef.current.delete(l); };
  }, []);

  return <WsContext.Provider value={{ send, subscribe, connected }}>{children}</WsContext.Provider>;
}
```

- [ ] **Step 2: useWebSocket / useRoom / useCameraViewers**

```ts
// src/ws/useWebSocket.ts
import { useContext } from "react";
import { WsContext } from "./WebSocketProvider";
export function useWebSocket() {
  const c = useContext(WsContext);
  if (!c) throw new Error("WebSocketProvider missing");
  return c;
}
```

```ts
// src/ws/useRoom.ts
import { useEffect, useState } from "react";
import { useWebSocket } from "./useWebSocket";
import type { RoomStateSnapshot } from "../types/api/RoomStateSnapshot";
import type { ChatMessage } from "../types/api/ChatMessage";
import type { ServerMessage } from "../types/api/ServerMessage";

export function useRoom(roomId: string) {
  const ws = useWebSocket();
  const [snapshot, setSnapshot] = useState<RoomStateSnapshot | null>(null);
  const [chat, setChat] = useState<ChatMessage[]>([]);

  useEffect(() => {
    ws.send({ type: "RoomJoin", payload: { roomId } } as any);
    const unsub = ws.subscribe((msg: ServerMessage) => {
      switch (msg.type) {
        case "RoomStateSnapshot":
          setSnapshot(msg.payload);
          setChat(msg.payload.recentChat);
          break;
        case "RoomChatMessage":
          if ((msg.payload as any).roomId === roomId)
            setChat((prev) => [...prev, (msg.payload as any).message]);
          break;
        case "RoomCameraChanged":
          if ((msg.payload as any).roomId === roomId)
            setSnapshot((prev) => prev ? { ...prev, room: { ...prev.room, currentCameraId: (msg.payload as any).cameraId } } : prev);
          break;
      }
    });
    return () => {
      ws.send({ type: "RoomLeave", payload: { roomId } } as any);
      unsub();
    };
  }, [roomId, ws]);

  const sendChat = (text: string) =>
    ws.send({ type: "RoomChatSend", payload: { roomId, text } } as any);
  const changeCamera = (cameraId: string) =>
    ws.send({ type: "RoomChangeCamera", payload: { roomId, cameraId } } as any);

  return { snapshot, chat, sendChat, changeCamera };
}
```

```ts
// src/ws/useCameraViewers.ts
import { useEffect, useState } from "react";
import { useWebSocket } from "./useWebSocket";
import type { ServerMessage } from "../types/api/ServerMessage";

export function useCameraViewers(cameraId: string | null) {
  const ws = useWebSocket();
  const [count, setCount] = useState(0);
  useEffect(() => {
    if (!cameraId) return;
    ws.send({ type: "CameraWatchStart", payload: { cameraId } } as any);
    const unsub = ws.subscribe((msg: ServerMessage) => {
      if (msg.type === "CameraViewersUpdate" && (msg.payload as any).cameraId === cameraId) {
        setCount((msg.payload as any).count);
      }
    });
    return () => {
      ws.send({ type: "CameraWatchStop", payload: { cameraId } } as any);
      unsub();
    };
  }, [cameraId, ws]);
  return count;
}
```

- [ ] **Step 3: Commit**

```bash
git add src/ws/
git commit -m "feat(ws): frontend provider + useRoom + useCameraViewers"
```

---

## Task 12: Rooms API (frontend)

**Files:** `src/api/rooms.ts`

```ts
import { apiRequest } from "./client";
import type { CreateRoomRequest } from "../types/api/CreateRoomRequest";
import type { RoomSummary } from "../types/api/RoomSummary";
import type { RoomStateSnapshot } from "../types/api/RoomStateSnapshot";
import type { InviteResponse } from "../types/api/InviteResponse";

export async function createRoom(req: CreateRoomRequest): Promise<RoomSummary> {
  return apiRequest<RoomSummary>("/rooms", { method: "POST", body: req });
}
export async function getRoom(id: string): Promise<RoomStateSnapshot> {
  return apiRequest<RoomStateSnapshot>(`/rooms/${id}`);
}
export async function getInvite(id: string): Promise<InviteResponse> {
  return apiRequest<InviteResponse>(`/rooms/${id}/invite`, { method: "POST" });
}
export async function joinByInvite(code: string): Promise<RoomSummary> {
  return apiRequest<RoomSummary>(`/rooms/by-invite/${code}`);
}
```

Commit:

```bash
git add src/api/rooms.ts
git commit -m "feat(api): rooms client wrapper"
```

---

## Task 13: RoomScreen по Figma "Канал"

**Files:** `src/screens/room/RoomScreen.tsx`, `RoomChat.tsx`, `RoomPlayer.tsx`, `room.css`

- [ ] **Step 1: RoomPlayer**

```tsx
import { useEffect, useRef } from "react";
import Hls from "hls.js";

const API_BASE = import.meta.env.VITE_API_BASE_URL ?? "http://localhost:3001/api/v1";

export default function RoomPlayer({ cameraSlug }: { cameraSlug: string | null }) {
  const ref = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    if (!cameraSlug || !ref.current) return;
    const url = `${API_BASE.replace(/\/api\/v1$/, "")}/api/v1/hls-proxy/${cameraSlug}`;
    let hls: Hls | null = null;
    if (Hls.isSupported()) {
      hls = new Hls();
      hls.loadSource(url);
      hls.attachMedia(ref.current);
    } else if (ref.current.canPlayType("application/vnd.apple.mpegurl")) {
      ref.current.src = url;
    }
    return () => { hls?.destroy(); };
  }, [cameraSlug]);

  return <video ref={ref} controls autoPlay muted className="room-player__video" />;
}
```

- [ ] **Step 2: RoomChat**

```tsx
import { useState } from "react";
import type { ChatMessage } from "../../types/api/ChatMessage";
import { useTranslation } from "react-i18next";

type Props = {
  messages: ChatMessage[];
  onSend(text: string): void;
};

export default function RoomChat({ messages, onSend }: Props) {
  const { t } = useTranslation();
  const [text, setText] = useState("");
  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!text.trim()) return;
    onSend(text);
    setText("");
  };
  return (
    <aside className="room-chat">
      <div className="room-chat__messages">
        {messages.map((m, i) => (
          <div key={i} className="room-chat__msg">
            <span className="room-chat__from">@{m.fromUsername}</span>
            <span className="room-chat__text">{m.text}</span>
          </div>
        ))}
      </div>
      <form className="room-chat__form" onSubmit={submit}>
        <input value={text} onChange={(e) => setText(e.target.value)} placeholder={t("room.chat.placeholder")} />
        <button type="submit" aria-label="send">➤</button>
      </form>
    </aside>
  );
}
```

- [ ] **Step 3: RoomScreen**

```tsx
import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useRoom } from "../../ws/useRoom";
import { useCameraViewers } from "../../ws/useCameraViewers";
import { getCamera } from "../../api/cameras";
import { getInvite } from "../../api/rooms";
import { useAuth } from "../../auth/useAuth";
import type { CameraDetail } from "../../types/api/CameraDetail";
import RoomPlayer from "./RoomPlayer";
import RoomChat from "./RoomChat";
import "./room.css";

export default function RoomScreen() {
  const { id } = useParams();
  const { user } = useAuth();
  const { t } = useTranslation();
  const { snapshot, chat, sendChat, changeCamera } = useRoom(id!);
  const [camera, setCamera] = useState<CameraDetail | null>(null);
  const viewers = useCameraViewers(snapshot?.room.currentCameraId ?? null);
  const [invite, setInvite] = useState<string | null>(null);

  useEffect(() => {
    const cid = snapshot?.room.currentCameraId;
    if (!cid) { setCamera(null); return; }
    let cancelled = false;
    getCamera(cid).then((c) => { if (!cancelled) setCamera(c); });
    return () => { cancelled = true; };
  }, [snapshot?.room.currentCameraId]);

  const copyInvite = async () => {
    if (!id) return;
    const { inviteUrl } = await getInvite(id);
    setInvite(inviteUrl);
    navigator.clipboard?.writeText(inviteUrl).catch(() => {});
  };

  const isHost = user?.id && snapshot?.room.hostUserId === user.id;

  return (
    <div className="room">
      <div className="room__main">
        <header className="room__head">
          <div className="room__title">{camera?.summary.name ?? t("room.loading")}</div>
          <div className="room__viewers">👥 {viewers}</div>
          <button className="room__invite" onClick={copyInvite}>{t("room.invite.copy")}</button>
        </header>

        <div className="room__player">
          <RoomPlayer cameraSlug={camera?.summary.slug ?? null} />
        </div>

        {isHost && (
          <div className="room__host-tools">
            <button onClick={() => {
              // M6 will replace this with a proper camera-picker modal;
              // for now we prompt for a slug or camera id.
              const cid = window.prompt(t("room.host.change_camera_prompt"));
              if (cid) changeCamera(cid);
            }}>
              {t("room.host.change_camera")}
            </button>
          </div>
        )}

        {invite && <div className="room__invite-toast">{t("room.invite.copied")}: {invite}</div>}
      </div>

      <RoomChat messages={chat} onSend={sendChat} />
    </div>
  );
}
```

- [ ] **Step 4: CSS**

```css
.room { display: grid; grid-template-columns: 1fr 320px; height: 100%; }
.room__main { display: flex; flex-direction: column; padding: 16px 24px; gap: 12px; overflow: hidden; }
.room__head { display: flex; align-items: center; gap: 16px; }
.room__title { flex: 1 1 auto; font-weight: 700; color: var(--t1); }
.room__viewers { color: var(--t2); }
.room__invite { padding: 8px 14px; border-radius: 8px; background: var(--t1); color: var(--bg); border: 0; cursor: pointer; }
.room__player { flex: 1 1 auto; background: var(--bg-3); border-radius: 12px; overflow: hidden; display: grid; place-items: center; }
.room-player__video { width: 100%; height: 100%; object-fit: contain; background: #000; }
.room__host-tools { display: flex; gap: 8px; }
.room__invite-toast { font-size: 12px; color: var(--t2); padding: 6px 10px; background: var(--bg-2); border-radius: 8px; }

.room-chat { display: flex; flex-direction: column; border-left: 1px solid var(--border); background: var(--bg); }
.room-chat__messages { flex: 1 1 auto; overflow-y: auto; padding: 12px; display: flex; flex-direction: column; gap: 8px; }
.room-chat__msg { display: flex; gap: 8px; align-items: baseline; }
.room-chat__from { color: var(--t3); font-weight: 600; font-size: 12px; }
.room-chat__text { color: var(--t1); font-size: 14px; }
.room-chat__form { display: flex; gap: 6px; padding: 10px; border-top: 1px solid var(--border); }
.room-chat__form input { flex: 1; padding: 8px 10px; border: 1px solid var(--border); border-radius: 8px; background: var(--bg-2); color: var(--t1); }
.room-chat__form button { border: 0; background: var(--accent); color: #fff; padding: 0 14px; border-radius: 8px; cursor: pointer; }
```

- [ ] **Step 5: i18n**

```json
{
  "room.loading": "Загрузка...",
  "room.invite.copy": "Пригласить",
  "room.invite.copied": "Ссылка скопирована",
  "room.chat.placeholder": "Сообщение...",
  "room.host.change_camera": "Сменить камеру",
  "room.host.change_camera_prompt": "Введите ID или slug камеры:"
}
```

- [ ] **Step 6: Routes + removal legacy**

Заменить `<WatchRoomScreen ...>` на `<RoomScreen />`:

```tsx
import RoomScreen from "../screens/room/RoomScreen";
// ...
{ path: "/room/:id", element: guarded(<RoomScreen />) },
```

Удалить старый `src/screens/WatchRoomScreen.tsx`.

- [ ] **Step 7: WebSocketProvider в App**

```tsx
import { WebSocketProvider } from "./ws/WebSocketProvider";
// внутри:
<AuthProvider>
  <WebSocketProvider>
    ...
  </WebSocketProvider>
</AuthProvider>
```

- [ ] **Step 8: Commit**

```bash
rm -f src/screens/WatchRoomScreen.tsx
git add -A
git commit -m "$(cat <<'EOF'
feat(room): RoomScreen per Figma with WebSocket sync + chat

- RoomPlayer renders HLS via hls.js from /api/v1/hls-proxy/:slug.
- RoomChat subscribes to room events, sends chat messages.
- Host-only "change camera" button; guest view is read-only.
- useCameraViewers shows live viewer count.
- Invite button copies the /r/:code URL to clipboard.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 14: Invite-роут

**Files:** `src/shell/routes.tsx`, `src/screens/room/InviteRedirect.tsx`

- [ ] **Step 1: Redirect-компонент**

```tsx
import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { joinByInvite } from "../../api/rooms";
import { useTranslation } from "react-i18next";

export default function InviteRedirect() {
  const { code } = useParams();
  const nav = useNavigate();
  const [err, setErr] = useState<string | null>(null);
  const { t } = useTranslation();
  useEffect(() => {
    if (!code) return;
    joinByInvite(code)
      .then((r) => nav(`/room/${r.id}`, { replace: true }))
      .catch((e) => setErr(e?.message ?? "error"));
  }, [code, nav]);
  return <div style={{ padding: 32 }}>{err ? `${t("app.error.generic")}: ${err}` : t("app.loading")}</div>;
}
```

- [ ] **Step 2: Route**

```tsx
{ path: "/r/:code", element: guarded(<InviteRedirect />) },
```

- [ ] **Step 3: CameraDetailPanel "Смотреть вместе" → реальное создание**

Открыть `src/screens/world/CameraDetailPanel.tsx`, заменить:

```tsx
import { createRoom, getInvite } from "../../api/rooms";

const watchTogether = async () => {
  const room = await createRoom({ cameraId: camera.id });
  const { inviteUrl } = await getInvite(room.id);
  navigator.clipboard?.writeText(inviteUrl).catch(() => {});
  nav(`/room/${room.id}`);
};
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(rooms): /r/:code invite flow + World3D Watch Together creates room"
```

---

## Task 15: End-to-end test

- [ ] **Step 1: Локальный туннель**

```bash
cloudflared tunnel --url http://localhost:1420
# ИЛИ отдельно для API:
cloudflared tunnel --url http://localhost:3001
```

Заметь: `.env.development` должен знать PUBLIC_BASE_URL для API; invite-ссылки генерируются с этой базой. В альфе для локального теста достаточно открыть в двух окнах одного хоста.

- [ ] **Step 2: Прогон сценария**

1. Запустить API (`cargo run -p placebo-api`) и фронт (`npm run tauri dev`).
2. Пользователь A: зарегистрироваться, зайти на World3D, кликнуть Shibuya Crossing, "Смотреть вместе". Комната создана, invite скопирован.
3. В другом окне Tauri (запустить вторую копию, использовать другой email): логин.
4. Открыть URL `/r/<code>` вставкой в адресную строку (временно делаем такую кнопку "вставить invite" в UI позже; в альфе запускаем приложение с аргументом deep-link или вставляем через форму).
   - Для простоты альфы: создать в Settings/Главная поле "Вставить invite-код" → `joinByInvite(code)` → переход.
5. Оба в комнате, видят Shibuya. Viewer count = 2.
6. Host жмёт "Сменить камеру", вводит slug `yt-times-square` → guest видит смену.
7. Оба пишут в чат, сообщения приходят мгновенно.

- [ ] **Step 3: Добавить поле для invite в Settings или Home**

В `SettingsScreen` или `HomeScreen` добавить секцию "Войти в комнату по коду":

```tsx
<section className="settings__group">
  <h2>{t("room.invite.join.title")}</h2>
  <form onSubmit={async (e) => {
    e.preventDefault();
    const code = new FormData(e.currentTarget).get("code") as string;
    const room = await joinByInvite(code);
    nav(`/room/${room.id}`);
  }}>
    <input name="code" placeholder="abc12345" />
    <button type="submit">{t("room.invite.join.submit")}</button>
  </form>
</section>
```

Позже (М7 полишинг) – сделать обработку deep-link'ов Tauri.

- [ ] **Step 4: Commit + PR**

```bash
git push -u origin feat/m5-rooms-websocket
```

PR: `feat/m5-rooms-websocket → main`.

---

## Acceptance Criteria

1. ✅ `POST /api/v1/rooms { cameraId }` создаёт комнату, возвращает RoomSummary с 8-символьным `inviteCode` base58.
2. ✅ `GET /api/v1/rooms/:id` возвращает RoomStateSnapshot (room + members + recentChat).
3. ✅ `GET /api/v1/rooms/by-invite/:code` привязывает пользователя и возвращает Room.
4. ✅ WebSocket `ws://.../ws?token=...` открывается только с валидным токеном.
5. ✅ `ClientMessage.RoomJoin` приводит к `ServerMessage.RoomStateSnapshot` + броадкасту `RoomMemberJoined` другим участникам.
6. ✅ `RoomChangeCamera` от host'а броадкастит `RoomCameraChanged`; от guest'а – `Error{code:"forbidden"}`.
7. ✅ `RoomChatSend` сохраняет сообщение в Redis (TRIM до 50) и броадкастит `RoomChatMessage`.
8. ✅ `CameraWatchStart/Stop` обновляет `CameraViewersUpdate` count.
9. ✅ Клиент реконнектится при обрыве с exponential backoff.
10. ✅ В UI RoomScreen рендерит HLS-стрим, чат, счётчик зрителей, кнопку "Пригласить" с clipboard-копированием.
11. ✅ Сценарий альфы (A создаёт, B заходит по invite, оба видят переключения и чат) работает end-to-end.

---

## Дальше

M6: Profile + Friends (pg_trgm поиск) + Settings logout уже работает + Create hub 4-тайла + Avatar upload через R2/local.
