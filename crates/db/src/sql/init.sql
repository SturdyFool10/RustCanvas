-- Table for the `User` struct
CREATE TABLE IF NOT EXISTS Users (
    username TEXT NOT NULL PRIMARY KEY,
    password_hash TEXT NOT NULL,
    security_key TEXT, -- Nullable
    salt TEXT NOT NULL,
    permissions UNSIGNED SMALLINT NOT NULL, -- 16-bit unsigned integer
    lockout_time BIGINT NOT NULL -- -1 if not locked out
);

-- Table for the `DrawnObject` struct
CREATE TABLE IF NOT EXISTS DrawnObjects (
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- Auto-incremented primary key
    type UNSIGNED INTEGER NOT NULL, -- 32-bit unsigned integer to represent the type
    num_args TEXT NOT NULL, -- Stored as a serialized JSON array of floats
    str_args TEXT NOT NULL, -- Stored as a serialized JSON array of strings
    color_args TEXT NOT NULL, -- Stored as a serialized JSON array of unsigned 32-bit integers
    bool_args TEXT NOT NULL -- Stored as a serialized JSON array of booleans
);
