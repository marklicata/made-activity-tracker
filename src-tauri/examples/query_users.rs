use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let db_path = "../../test.db";
    println!("Opening database: {}", db_path);

    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT id, github_id, login, name, tracked, tracked_at FROM users ORDER BY id"
    )?;

    let users = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, bool>(4)?,
            row.get::<_, Option<String>>(5)?,
        ))
    })?;

    println!("\nID | GitHub ID | Login | Name | Tracked | Tracked At");
    println!("---|-----------|-------|------|---------|------------");

    for user in users {
        let (id, github_id, login, name, tracked, tracked_at) = user?;
        println!(
            "{} | {} | {} | {} | {} | {}",
            id,
            github_id,
            login,
            name.unwrap_or_else(|| "NULL".to_string()),
            tracked,
            tracked_at.unwrap_or_else(|| "NULL".to_string())
        );
    }

    Ok(())
}
