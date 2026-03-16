use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Row types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RoomRow {
    pub id: Uuid,
    pub name: String,
    pub camera_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub is_private: bool,
    pub max_members: i16,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RoomMemberRow {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub joined_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Input structs
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct NewRoom {
    pub name: String,
    pub camera_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub is_private: bool,
    pub max_members: i16,
}

#[derive(Debug)]
pub struct UpdateRoom {
    pub name: Option<String>,
    pub camera_id: Option<Option<Uuid>>, // None = don't change, Some(None) = clear, Some(Some(id)) = set
    pub is_private: Option<bool>,
    pub max_members: Option<i16>,
}

// ---------------------------------------------------------------------------
// Repository functions
// ---------------------------------------------------------------------------

pub async fn create(pool: &PgPool, room: &NewRoom) -> Result<RoomRow, sqlx::Error> {
    sqlx::query_as::<_, RoomRow>(
        r#"INSERT INTO rooms (name, camera_id, owner_id, is_private, max_members)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *"#,
    )
    .bind(&room.name)
    .bind(room.camera_id)
    .bind(room.owner_id)
    .bind(room.is_private)
    .bind(room.max_members)
    .fetch_one(pool)
    .await
}

pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<RoomRow>, sqlx::Error> {
    sqlx::query_as::<_, RoomRow>("SELECT * FROM rooms WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_user_rooms(pool: &PgPool, user_id: Uuid) -> Result<Vec<RoomRow>, sqlx::Error> {
    sqlx::query_as::<_, RoomRow>(
        r#"SELECT DISTINCT r.* FROM rooms r
        LEFT JOIN room_members rm ON r.id = rm.room_id
        WHERE r.owner_id = $1 OR rm.user_id = $1
        ORDER BY r.created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    upd: &UpdateRoom,
) -> Result<Option<RoomRow>, sqlx::Error> {
    // For camera_id: if upd.camera_id is None, keep current; if Some(v), set to v
    let camera_id_val: Option<Uuid> = match &upd.camera_id {
        Some(v) => *v,
        None => {
            // Fetch current to preserve
            if let Some(row) = get_by_id(pool, id).await? {
                row.camera_id
            } else {
                return Ok(None);
            }
        }
    };

    sqlx::query_as::<_, RoomRow>(
        r#"UPDATE rooms SET
            name = COALESCE($2, name),
            camera_id = $3,
            is_private = COALESCE($4, is_private),
            max_members = COALESCE($5, max_members)
        WHERE id = $1
        RETURNING *"#,
    )
    .bind(id)
    .bind(&upd.name)
    .bind(camera_id_val)
    .bind(upd.is_private)
    .bind(upd.max_members)
    .fetch_optional(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM rooms WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn add_member(pool: &PgPool, room_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO room_members (room_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(room_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_member(
    pool: &PgPool,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result =
        sqlx::query("DELETE FROM room_members WHERE room_id = $1 AND user_id = $2")
            .bind(room_id)
            .bind(user_id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_members(
    pool: &PgPool,
    room_id: Uuid,
) -> Result<Vec<RoomMemberRow>, sqlx::Error> {
    sqlx::query_as::<_, RoomMemberRow>(
        r#"SELECT rm.user_id, u.display_name, u.avatar_url, rm.joined_at
        FROM room_members rm
        JOIN users u ON u.id = rm.user_id
        WHERE rm.room_id = $1
        ORDER BY rm.joined_at"#,
    )
    .bind(room_id)
    .fetch_all(pool)
    .await
}

pub async fn is_member(pool: &PgPool, room_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM room_members WHERE room_id = $1 AND user_id = $2)",
    )
    .bind(room_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn member_count(pool: &PgPool, room_id: Uuid) -> Result<i64, sqlx::Error> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM room_members WHERE room_id = $1")
            .bind(room_id)
            .fetch_one(pool)
            .await?;
    Ok(row.0)
}
