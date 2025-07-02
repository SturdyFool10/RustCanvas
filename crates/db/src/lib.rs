#[allow(dead_code)]
use std::error::Error;
use std::path::Path;

/// Represents a user in the database.
pub struct User {
    /// The username of the user.
    pub username: String,
    /// The hashed password of the user.
    pub password_hash: String,
    /// The security key associated with the user, may be None.
    pub security_key: Option<String>,
    /// The salt used for hashing the user's password.
    pub salt: String,
    /// The permissions level of the user, represented as a 16-bit unsigned integer.
    pub permissions: u16,
    ///lockout time of user, -1 if not locked out
    pub lockout_time: i64,
}

pub struct DrawnObject {
    //id to tell us what type of object it is
    pub id: u32,
    //object arguments
    pub num_args: Vec<f64>,
    //object string args
    pub str_args: Vec<String>,
    //object color args
    pub color_args: Vec<(u8, u8, u8)>,
    //object boolean args
    pub bool_args: Vec<bool>,
}
#[allow(dead_code)]
pub struct DatabaseConnection {
    conn: rusqlite::Connection,
}
impl DatabaseConnection {
    pub fn new(path: &Path) -> Result<Self, Box<dyn Error>> {
        let conn = rusqlite::Connection::open(path)?;
        let sql = include_str!("sql/init.sql");
        conn.execute_batch(sql)?;
        Ok(Self { conn })
    }
}
