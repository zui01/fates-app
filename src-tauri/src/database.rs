// https://github.com/RandomEngy/tauri-sqlite/blob/main/src-tauri/src/database.rs

use crate::utils;
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::RwLock;
use tauri::AppHandle;

const CURRENT_DB_VERSION: u32 = 1;

const DB_NAME: &str = "fates.db";

fn default_datetime() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap()
}

fn check_is_default_datetime(datetime: DateTime<Utc>) -> bool {
    datetime.timestamp() == 0 && datetime.timestamp_subsec_nanos() == 0
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Matter {
    #[serde(default)]
    pub id: String, // UUID
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
    #[serde(default = "default_datetime")]
    pub start_time: DateTime<Utc>,
    #[serde(default = "default_datetime")]
    pub end_time: DateTime<Utc>,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub type_: i32, // 0 normal, 1 repeat, 2 todo
    #[serde(default = "default_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "default_datetime")]
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub reserved_1: Option<String>,
    #[serde(default)]
    pub reserved_2: Option<String>,
    #[serde(default)]
    pub reserved_3: Option<String>,
    #[serde(default)]
    pub reserved_4: Option<String>,
    #[serde(default)]
    pub reserved_5: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepeatTask {
    pub id: String,
    pub title: String,
    pub tags: Option<String>,
    pub repeat_time: String,
    pub status: i32, // 1=Active, 0=Stopped, -1=Archived
    #[serde(default = "default_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "default_datetime")]
    pub updated_at: DateTime<Utc>,
    pub priority: i32,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KVStore {
    pub key: String,
    pub value: String,
    #[serde(default = "default_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "default_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    #[serde(default = "default_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "default_datetime")]
    pub last_used_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: String, // UUID
    pub title: String,
    pub status: String, // "todo", "in_progress", "completed"
    #[serde(default = "default_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "default_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecord {
    pub id: String,
    pub title: String,
    pub content: String,
    pub type_: i32,
    pub status: i32,
    pub related_task_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
    pub expire_at: Option<DateTime<Utc>>,
    pub action_url: Option<String>,
    pub reserved_1: Option<String>,
    pub reserved_2: Option<String>,
    pub reserved_3: Option<String>,
    pub reserved_4: Option<String>,
    pub reserved_5: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationStatus {
    Unread = 0,
    Read = 1,
}

pub struct SafeConnection {
    conn: RwLock<Connection>,
}

impl SafeConnection {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: RwLock::new(conn),
        }
    }
}

unsafe impl Send for SafeConnection {}
unsafe impl Sync for SafeConnection {}

pub fn initialize_database(app_handle: &AppHandle) -> Result<Arc<SafeConnection>> {
    let app_dir = utils::get_app_data_dir(app_handle.clone()).unwrap();
    let db_path = app_dir.join(DB_NAME);

    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_CREATE
        | OpenFlags::SQLITE_OPEN_NO_MUTEX;

    let conn = Connection::open_with_flags(db_path, flags)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS matter (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT DEFAULT '',
            tags TEXT DEFAULT '',
            start_time DATETIME NOT NULL,
            end_time DATETIME NOT NULL,
            priority INTEGER DEFAULT 0,
            type INTEGER DEFAULT 0,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            reserved_1 TEXT DEFAULT '',
            reserved_2 TEXT DEFAULT '',
            reserved_3 TEXT DEFAULT '',
            reserved_4 TEXT DEFAULT '',
            reserved_5 TEXT DEFAULT ''
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS kvstore (
            key TEXT PRIMARY KEY,
            value TEXT DEFAULT '',
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            name TEXT PRIMARY KEY,
            created_at DATETIME NOT NULL,
            last_used_at DATETIME NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_matter_time ON matter(start_time, end_time)",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS repeat_task (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            tags TEXT DEFAULT '',
            repeat_time TEXT NOT NULL,
            status INTEGER DEFAULT 1,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            priority INTEGER DEFAULT 0,
            description TEXT DEFAULT ''
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS todo (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS notification_records (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            type INTEGER NOT NULL,
            status INTEGER NOT NULL DEFAULT 0,
            related_task_id TEXT,
            created_at DATETIME NOT NULL,
            read_at DATETIME,
            expire_at DATETIME,
            action_url TEXT,
            reserved_1 TEXT,
            reserved_2 TEXT,
            reserved_3 TEXT,
            reserved_4 TEXT,
            reserved_5 TEXT
        )",
        [],
    )?;

    Ok(Arc::new(SafeConnection::new(conn)))
}

impl Matter {
    pub fn create(conn: &Arc<SafeConnection>, matter: &Matter) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute(
            "INSERT INTO matter (
                id, title, description, tags, start_time, end_time,
                priority, type, created_at, updated_at,
                reserved_1, reserved_2, reserved_3, reserved_4, reserved_5
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15
            )",
            params![
                matter.id,
                matter.title,
                matter.description,
                matter.tags,
                matter.start_time,
                matter.end_time,
                matter.priority,
                matter.type_,
                matter.created_at,
                matter.updated_at,
                matter.reserved_1,
                matter.reserved_2,
                matter.reserved_3,
                matter.reserved_4,
                matter.reserved_5
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(conn: &Arc<SafeConnection>, id: &str) -> Result<Option<Matter>> {
        let conn = conn.conn.read().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM matter WHERE id = ?1")?;

        let matter = stmt
            .query_row(params![id], |row| {
                Ok(Matter {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    tags: row.get(3)?,
                    start_time: row.get(4)?,
                    end_time: row.get(5)?,
                    priority: row.get(6)?,
                    type_: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    reserved_1: row.get(10)?,
                    reserved_2: row.get(11)?,
                    reserved_3: row.get(12)?,
                    reserved_4: row.get(13)?,
                    reserved_5: row.get(14)?,
                })
            })
            .optional()?;

        Ok(matter)
    }

    pub fn get_all(conn: &Arc<SafeConnection>) -> Result<Vec<Matter>> {
        let conn = conn.conn.read().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM matter ORDER BY start_time")?;
        let matters = stmt
            .query_map([], |row| {
                Ok(Matter {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    tags: row.get(3)?,
                    start_time: row.get(4)?,
                    end_time: row.get(5)?,
                    priority: row.get(6)?,
                    type_: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    reserved_1: row.get(10)?,
                    reserved_2: row.get(11)?,
                    reserved_3: row.get(12)?,
                    reserved_4: row.get(13)?,
                    reserved_5: row.get(14)?,
                })
            })?
            .collect();
        matters
    }

    pub fn get_by_time_range(
        conn: &Arc<SafeConnection>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Matter>> {
        let conn = conn.conn.read().unwrap();
        let mut stmt = conn.prepare(
            "SELECT * FROM matter
            WHERE (start_time BETWEEN ?1 AND ?2)
            OR (end_time BETWEEN ?1 AND ?2)
            OR (start_time <= ?1 AND end_time >= ?2)
            ORDER BY start_time",
        )?;

        let matters = stmt
            .query_map(params![start, end], |row| {
                Ok(Matter {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    tags: row.get(3)?,
                    start_time: row.get(4)?,
                    end_time: row.get(5)?,
                    priority: row.get(6)?,
                    type_: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    reserved_1: row.get(10)?,
                    reserved_2: row.get(11)?,
                    reserved_3: row.get(12)?,
                    reserved_4: row.get(13)?,
                    reserved_5: row.get(14)?,
                })
            })?
            .collect();

        matters
    }

    pub fn update(&self, conn: &Arc<SafeConnection>) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute(
            "UPDATE matter SET
                title = ?1, description = ?2, tags = ?3,
                start_time = ?4, end_time = ?5, priority = ?6,
                type = ?7, updated_at = ?8,
                reserved_1 = ?9, reserved_2 = ?10, reserved_3 = ?11,
                reserved_4 = ?12, reserved_5 = ?13
            WHERE id = ?14",
            params![
                self.title,
                self.description,
                self.tags,
                self.start_time,
                self.end_time,
                self.priority,
                self.type_,
                self.updated_at,
                self.reserved_1,
                self.reserved_2,
                self.reserved_3,
                self.reserved_4,
                self.reserved_5,
                self.id
            ],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Arc<SafeConnection>, id: &str) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute("DELETE FROM matter WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn query_by_field(
        conn: &Arc<SafeConnection>,
        field: &str,
        value: &str,
        exact_match: bool,
    ) -> Result<Vec<Matter>> {
        let conn = conn.conn.read().unwrap();

        // 构建查询语句
        let query = if exact_match {
            format!(
                "SELECT * FROM matter WHERE {} = ?1 ORDER BY start_time",
                field
            )
        } else {
            format!(
                "SELECT * FROM matter WHERE {} LIKE ?1 ORDER BY start_time",
                field
            )
        };

        let mut stmt = conn.prepare(&query)?;

        // 如果不是精确匹配，则使用模糊查询
        let search_value = if exact_match {
            value.to_string()
        } else {
            format!("%{}%", value)
        };

        let matters = stmt
            .query_map([search_value], |row| {
                Ok(Matter {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    tags: row.get(3)?,
                    start_time: row.get(4)?,
                    end_time: row.get(5)?,
                    priority: row.get(6)?,
                    type_: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    reserved_1: row.get(10)?,
                    reserved_2: row.get(11)?,
                    reserved_3: row.get(12)?,
                    reserved_4: row.get(13)?,
                    reserved_5: row.get(14)?,
                })
            })?
            .collect();

        matters
    }
}

// KVStore 相关操作
impl KVStore {
    pub fn set(conn: &Arc<SafeConnection>, key: &str, value: &str) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        let now = Utc::now();
        conn.execute(
            "INSERT INTO kvstore (key, value, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?3)
            ON CONFLICT(key) DO UPDATE SET
            value = ?2, updated_at = ?3",
            params![key, value, now],
        )?;
        Ok(())
    }

    pub fn get(conn: &Arc<SafeConnection>, key: &str, default: &str) -> Result<String> {
        let conn = conn.conn.read().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM kvstore WHERE key = ?1")?;
        let value = stmt.query_row(params![key], |row| row.get(0)).optional()?;
        Ok(value.unwrap_or(default.to_string()))
    }

    pub fn delete(conn: &Arc<SafeConnection>, key: &str) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute("DELETE FROM kvstore WHERE key = ?1", params![key])?;
        Ok(())
    }
}

// Tag 相关操作
impl Tag {
    pub fn create(conn: &Arc<SafeConnection>, name: &str) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO tags (name, created_at, last_used_at) VALUES (?1, ?2, ?3)",
            params![name, Utc::now(), Utc::now()],
        )?;
        Ok(())
    }

    pub fn get_all(conn: &Arc<SafeConnection>) -> Result<Vec<Tag>> {
        let conn = conn.conn.read().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM tags ORDER BY name")?;
        let tags = stmt
            .query_map([], |row| {
                Ok(Tag {
                    name: row.get(0)?,
                    created_at: row.get(1)?,
                    last_used_at: row.get(2)?,
                })
            })?
            .collect();
        tags
    }

    pub fn update_last_used_at(conn: &Arc<SafeConnection>, name: &str) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute(
            "UPDATE tags SET last_used_at = ?1 WHERE name = ?2",
            params![Utc::now(), name],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Arc<SafeConnection>, name: &str) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute("DELETE FROM tags WHERE name = ?1", params![name])?;
        Ok(())
    }
}

// RepeatTask 相关操作
impl RepeatTask {
    pub fn create(conn: &Arc<SafeConnection>, task: &RepeatTask) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute(
            "INSERT INTO repeat_task (
                id, title, tags, repeat_time, status,
                created_at, updated_at, priority, description
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9
            )",
            params![
                task.id,
                task.title,
                task.tags,
                task.repeat_time,
                task.status,
                task.created_at,
                task.updated_at,
                task.priority,
                task.description
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(conn: &Arc<SafeConnection>, id: &str) -> Result<Option<RepeatTask>> {
        let conn = conn.conn.read().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM repeat_task WHERE id = ?1")?;

        let task = stmt
            .query_row(params![id], |row| {
                Ok(RepeatTask {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    tags: row.get(2)?,
                    repeat_time: row.get(3)?,
                    status: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                    priority: row.get(7)?,
                    description: row.get(8)?,
                })
            })
            .optional()?;

        Ok(task)
    }

    pub fn get_all(conn: &Arc<SafeConnection>) -> Result<Vec<RepeatTask>> {
        let conn = conn.conn.read().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM repeat_task ORDER BY created_at DESC")?;
        let tasks = stmt
            .query_map([], |row| {
                Ok(RepeatTask {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    tags: row.get(2)?,
                    repeat_time: row.get(3)?,
                    status: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                    priority: row.get(7)?,
                    description: row.get(8)?,
                })
            })?
            .collect();
        tasks
    }

    pub fn get_active_tasks(conn: &Arc<SafeConnection>) -> Result<Vec<RepeatTask>> {
        let conn = conn.conn.read().unwrap();
        let mut stmt =
            conn.prepare("SELECT * FROM repeat_task WHERE status = 1 ORDER BY created_at DESC")?;
        let tasks = stmt
            .query_map([], |row| {
                Ok(RepeatTask {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    tags: row.get(2)?,
                    repeat_time: row.get(3)?,
                    status: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                    priority: row.get(7)?,
                    description: row.get(8)?,
                })
            })?
            .collect();
        tasks
    }

    pub fn update(&self, conn: &Arc<SafeConnection>) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute(
            "UPDATE repeat_task SET
                title = ?1,
                tags = ?2,
                repeat_time = ?3,
                status = ?4,
                updated_at = ?5,
                priority = ?6,
                description = ?7
            WHERE id = ?8",
            params![
                self.title,
                self.tags,
                self.repeat_time,
                self.status,
                self.updated_at,
                self.priority,
                self.description,
                self.id
            ],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Arc<SafeConnection>, id: &str) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute("DELETE FROM repeat_task WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_status(conn: &Arc<SafeConnection>, id: &str, new_status: i32) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute(
            "UPDATE repeat_task SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![new_status, Utc::now(), id],
        )?;
        Ok(())
    }
}

impl Todo {
    pub fn create(conn: &Arc<SafeConnection>, todo: &Todo) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute(
            "INSERT INTO todo (id, title, status, created_at, updated_at)
    VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                todo.id,
                todo.title,
                todo.status,
                todo.created_at,
                todo.updated_at
            ],
        )?;
        Ok(())
    }
    pub fn get_by_id(conn: &Arc<SafeConnection>, id: &str) -> Result<Option<Todo>> {
        let conn = conn.conn.read().unwrap();
        let mut stmt = conn.prepare("SELECT FROM todo WHERE id = ?1")?;
        let todo = stmt
            .query_row(params![id], |row| {
                Ok(Todo {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    status: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })
            .optional()?;
        Ok(todo)
    }
    pub fn get_all(conn: &Arc<SafeConnection>) -> Result<Vec<Todo>> {
        let conn = conn.conn.read().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM todo ORDER BY created_at DESC")?;
        let todos = stmt
            .query_map([], |row| {
                Ok(Todo {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    status: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect();
        todos
    }

    pub fn update(&self, conn: &Arc<SafeConnection>) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute(
            "UPDATE todo SET
        title = ?1,
        status = ?2,
        updated_at = ?3
        WHERE id = ?4",
            params![self.title, self.status, self.updated_at, self.id],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Arc<SafeConnection>, id: &str) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute("DELETE FROM todo WHERE id = ?1", params![id])?;
        Ok(())
    }
}

impl NotificationRecord {
    pub fn create(conn: &Arc<SafeConnection>, notification: &NotificationRecord) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute(
            "INSERT INTO notification_records (
                id, title, content, type, status, related_task_id,
                created_at, read_at, expire_at, action_url,
                reserved_1, reserved_2, reserved_3, reserved_4, reserved_5
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15
            )",
            params![
                notification.id,
                notification.title,
                notification.content,
                notification.type_,
                notification.status,
                notification.related_task_id,
                notification.created_at,
                notification.read_at,
                notification.expire_at,
                notification.action_url,
                notification.reserved_1,
                notification.reserved_2,
                notification.reserved_3,
                notification.reserved_4,
                notification.reserved_5
            ],
        )?;
        Ok(())
    }

    pub fn get_unread(conn: &Arc<SafeConnection>) -> Result<Vec<NotificationRecord>> {
        let conn = conn.conn.read().unwrap();
        let mut stmt = conn.prepare(
            "SELECT * FROM notification_records
            WHERE status = 0
            ORDER BY created_at DESC",
        )?;

        let notifications = stmt
            .query_map([], |row| {
                Ok(NotificationRecord {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    type_: row.get(3)?,
                    status: row.get(4)?,
                    related_task_id: row.get(5)?,
                    created_at: row.get(6)?,
                    read_at: row.get(7)?,
                    expire_at: row.get(8)?,
                    action_url: row.get(9)?,
                    reserved_1: row.get(10)?,
                    reserved_2: row.get(11)?,
                    reserved_3: row.get(12)?,
                    reserved_4: row.get(13)?,
                    reserved_5: row.get(14)?,
                })
            })?
            .collect();

        notifications
    }

    pub fn mark_as_read(conn: &Arc<SafeConnection>, id: &str) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute(
            "UPDATE notification_records
            SET status = ?1, read_at = ?2
            WHERE id = ?3",
            params![NotificationStatus::Read as i32, Utc::now(), id],
        )?;
        Ok(())
    }
    pub fn mark_as_read_by_type(conn: &Arc<SafeConnection>, type_: i32) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute(
            "UPDATE notification_records SET status = ?1, read_at = ?2 WHERE type = ?3",
            params![NotificationStatus::Read as i32, Utc::now(), type_],
        )?;
        Ok(())
    }
    pub fn mark_all_as_read(conn: &Arc<SafeConnection>) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute(
            "UPDATE notification_records
            SET status = ?1, read_at = ?2
            WHERE status = ?3",
            params![
                NotificationStatus::Read as i32,
                Utc::now(),
                NotificationStatus::Unread as i32
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(conn: &Arc<SafeConnection>, id: &str) -> Result<Option<NotificationRecord>> {
        let conn = conn.conn.read().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM notification_records WHERE id = ?1")?;

        let notification = stmt
            .query_row(params![id], |row| {
                Ok(NotificationRecord {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    type_: row.get(3)?,
                    status: row.get(4)?,
                    related_task_id: row.get(5)?,
                    created_at: row.get(6)?,
                    read_at: row.get(7)?,
                    expire_at: row.get(8)?,
                    action_url: row.get(9)?,
                    reserved_1: row.get(10)?,
                    reserved_2: row.get(11)?,
                    reserved_3: row.get(12)?,
                    reserved_4: row.get(13)?,
                    reserved_5: row.get(14)?,
                })
            })
            .optional()?;

        Ok(notification)
    }

    pub fn update(&self, conn: &Arc<SafeConnection>) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute(
            "UPDATE notification_records SET
                title = ?1,
                content = ?2,
                type = ?3,
                status = ?4,
                related_task_id = ?5,
                expire_at = ?6,
                action_url = ?7,
                reserved_1 = ?8,
                reserved_2 = ?9,
                reserved_3 = ?10,
                reserved_4 = ?11,
                reserved_5 = ?12
            WHERE id = ?13",
            params![
                self.title,
                self.content,
                self.type_,
                self.status,
                self.related_task_id,
                self.expire_at,
                self.action_url,
                self.reserved_1,
                self.reserved_2,
                self.reserved_3,
                self.reserved_4,
                self.reserved_5,
                self.id
            ],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Arc<SafeConnection>, id: &str) -> Result<()> {
        let conn = conn.conn.write().unwrap();
        conn.execute(
            "DELETE FROM notification_records WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }
}
