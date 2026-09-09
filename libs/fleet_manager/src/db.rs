use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub id: String,
    pub hostname: String,
    pub os: String,
    pub last_seen: i64,
}

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("fleet.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS devices (
            id TEXT PRIMARY KEY,
            hostname TEXT NOT NULL,
            os TEXT NOT NULL,
            last_seen INTEGER NOT NULL
        )",
        [],
    )?;

    // We can add users table later for web login, but for now we'll just hardcode admin for MVP.
    Ok(conn)
}

pub fn upsert_device(conn: &Connection, device: &Device) -> Result<()> {
    conn.execute(
        "INSERT INTO devices (id, hostname, os, last_seen)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET
         hostname = excluded.hostname,
         os = excluded.os,
         last_seen = excluded.last_seen",
        params![device.id, device.hostname, device.os, device.last_seen],
    )?;
    Ok(())
}

pub fn get_all_devices(conn: &Connection) -> Result<Vec<Device>> {
    let mut stmt = conn.prepare("SELECT id, hostname, os, last_seen FROM devices ORDER BY last_seen DESC")?;
    let device_iter = stmt.query_map([], |row| {
        Ok(Device {
            id: row.get(0)?,
            hostname: row.get(1)?,
            os: row.get(2)?,
            last_seen: row.get(3)?,
        })
    })?;

    let mut devices = Vec::new();
    for device in device_iter {
        devices.push(device?);
    }
    Ok(devices)
}
