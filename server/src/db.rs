use rusqlite::{params, Connection, OptionalExtension, Result};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct PlayerDbRecord {
    pub id: u64,
    pub github_id: Option<String>,
    pub guest_id: Option<String>,
    pub username: String,
    pub credits: u32,
    pub minerals: u32,
    pub score: u32,
    pub ship_model: String,
    pub save_data: Option<String>,
}

pub type DbPool = Arc<Mutex<Connection>>;

pub fn init_db(db_path: &str) -> Result<DbPool> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS players (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            github_id TEXT UNIQUE,
            guest_id TEXT UNIQUE,
            username TEXT NOT NULL,
            credits INTEGER NOT NULL DEFAULT 1000,
            minerals INTEGER NOT NULL DEFAULT 0,
            score INTEGER NOT NULL DEFAULT 0,
            ship_model TEXT NOT NULL DEFAULT 'Phoenix',
            save_data TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    // Add column if migrating existing database
    let _ = conn.execute("ALTER TABLE players ADD COLUMN save_data TEXT", []);
    Ok(Arc::new(Mutex::new(conn)))
}

pub fn get_or_create_github_player(
    pool: &DbPool,
    github_id: &str,
    username: &str,
) -> Result<PlayerDbRecord> {
    let conn = pool.lock().unwrap();

    let mut stmt = conn.prepare(
        "SELECT id, github_id, guest_id, username, credits, minerals, score, ship_model, save_data 
         FROM players WHERE github_id = ?1",
    )?;

    let existing = stmt
        .query_row(params![github_id], |row| {
            Ok(PlayerDbRecord {
                id: row.get::<_, i64>(0)? as u64,
                github_id: row.get(1)?,
                guest_id: row.get(2)?,
                username: row.get(3)?,
                credits: row.get::<_, i64>(4)? as u32,
                minerals: row.get::<_, i64>(5)? as u32,
                score: row.get::<_, i64>(6)? as u32,
                ship_model: row.get(7)?,
                save_data: row.get(8)?,
            })
        })
        .optional()?;

    if let Some(mut record) = existing {
        if !username.is_empty() && record.username != username {
            let _ = conn.execute(
                "UPDATE players SET username = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![username, record.id as i64],
            );
            record.username = username.to_string();
        }
        return Ok(record);
    }

    conn.execute(
        "INSERT INTO players (github_id, username, credits, minerals, score, ship_model)
         VALUES (?1, ?2, 1000, 0, 0, 'Phoenix')",
        params![github_id, username],
    )?;

    let new_id = conn.last_insert_rowid() as u64;

    Ok(PlayerDbRecord {
        id: new_id,
        github_id: Some(github_id.to_string()),
        guest_id: None,
        username: username.to_string(),
        credits: 1000,
        minerals: 0,
        score: 0,
        ship_model: "Phoenix".to_string(),
        save_data: None,
    })
}

pub fn get_or_create_guest_player(
    pool: &DbPool,
    guest_id: &str,
    username: &str,
) -> Result<PlayerDbRecord> {
    let conn = pool.lock().unwrap();

    let mut stmt = conn.prepare(
        "SELECT id, github_id, guest_id, username, credits, minerals, score, ship_model, save_data 
         FROM players WHERE guest_id = ?1",
    )?;

    let existing = stmt
        .query_row(params![guest_id], |row| {
            Ok(PlayerDbRecord {
                id: row.get::<_, i64>(0)? as u64,
                github_id: row.get(1)?,
                guest_id: row.get(2)?,
                username: row.get(3)?,
                credits: row.get::<_, i64>(4)? as u32,
                minerals: row.get::<_, i64>(5)? as u32,
                score: row.get::<_, i64>(6)? as u32,
                ship_model: row.get(7)?,
                save_data: row.get(8)?,
            })
        })
        .optional()?;

    if let Some(mut record) = existing {
        if !username.is_empty() && record.username != username {
            let _ = conn.execute(
                "UPDATE players SET username = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![username, record.id as i64],
            );
            record.username = username.to_string();
        }
        return Ok(record);
    }

    conn.execute(
        "INSERT INTO players (guest_id, username, credits, minerals, score, ship_model)
         VALUES (?1, ?2, 1000, 0, 0, 'Phoenix')",
        params![guest_id, username],
    )?;

    let new_id = conn.last_insert_rowid() as u64;

    Ok(PlayerDbRecord {
        id: new_id,
        github_id: None,
        guest_id: Some(guest_id.to_string()),
        username: username.to_string(),
        credits: 1000,
        minerals: 0,
        score: 0,
        ship_model: "Phoenix".to_string(),
        save_data: None,
    })
}

pub fn get_player_by_id(pool: &DbPool, player_id: u64) -> Result<Option<PlayerDbRecord>> {
    let conn = pool.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, github_id, guest_id, username, credits, minerals, score, ship_model, save_data 
         FROM players WHERE id = ?1",
    )?;

    let res = stmt
        .query_row(params![player_id as i64], |row| {
            Ok(PlayerDbRecord {
                id: row.get::<_, i64>(0)? as u64,
                github_id: row.get(1)?,
                guest_id: row.get(2)?,
                username: row.get(3)?,
                credits: row.get::<_, i64>(4)? as u32,
                minerals: row.get::<_, i64>(5)? as u32,
                score: row.get::<_, i64>(6)? as u32,
                ship_model: row.get(7)?,
                save_data: row.get(8)?,
            })
        })
        .optional()?;

    Ok(res)
}

pub fn save_player_progression(
    pool: &DbPool,
    player_id: u64,
    save_data: &astrobrawl_shared::PlayerSaveData,
) -> Result<()> {
    let conn = pool.lock().unwrap();
    let json = serde_json::to_string(save_data).unwrap_or_default();
    let minerals = save_data.cargo.total_value_credits();
    conn.execute(
        "UPDATE players 
         SET credits = ?1, minerals = ?2, score = ?3, save_data = ?4, updated_at = CURRENT_TIMESTAMP 
         WHERE id = ?5",
        params![
            save_data.credits as i64,
            minerals as i64,
            save_data.score as i64,
            json,
            player_id as i64
        ],
    )?;
    Ok(())
}

pub fn save_player_stats(
    pool: &DbPool,
    player_id: u64,
    credits: u32,
    minerals: u32,
    score: u32,
) -> Result<()> {
    let conn = pool.lock().unwrap();
    conn.execute(
        "UPDATE players SET credits = ?1, minerals = ?2, score = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?4",
        params![credits as i64, minerals as i64, score as i64, player_id as i64],
    )?;
    Ok(())
}
