use chatserver;
use postgres::{Connection, TlsMode};
use serde_json;
use std::env;

fn connect() -> Connection {
    // Get postgres env vars
    let pgenv = vec!["PGUSER", "PGPASSWORD", "PGHOST", "PGPORT", "PGDATABASE"];
    let mut pgvars = Vec::new();

    // Put them all in a vec
    for var in pgenv {
        match env::var(var) {
            Ok(val) => pgvars.push(val),
            Err(e) => println!("couldn't interpret {}: {}", var, e),
        }
    }

    // Assemble the url
    let url = format!(
        "postgres://{}:{}@{}:{}/{}",
        pgvars[0], pgvars[1], pgvars[2], pgvars[3], pgvars[4]
    );

    // Connect
    Connection::connect(url, TlsMode::None).unwrap()
}

pub fn init() {
    let db = connect();

    let messages_table = "CREATE TABLE IF NOT EXISTS messages (
        id UUID,
        sender_name VARCHAR NOT NULL,
        msg VARCHAR NOT NULL,
        room VARCHAR NOT NULL)";
    let users_table = "CREATE TABLE IF NOT EXISTS users (
        id UUID NOT NULL,
        image VARCHAR,
        email VARCHAR NOT NULL,
        display_name VARCHAR NOT NULL,
        password VARCHAR NOT NULL,
        state INTEGER NOT NULL,
        created_at TIMESTAMP NOT NULL,
        updated_at TIMESTAMP,
        last_online TIMESTAMP,
        roles VARCHAR)";
    let rooms_table = "CREATE TABLE IF NOT EXISTS rooms (
        id UUID NOT NULL,
        name VARCHAR NOT NULL,
        owner UUID NOT NULL,
        members JSON,
        topic VARCHAR NOT NULL,
        private BOOLEAN NOT NULL,
        hidden BOOLEAN NOT NULL,
        moderators JSON,
        created_at TIMESTAMP NOT NULL,
        updated_at TIMESTAMP,
        last_message_at TIMESTAMP,
        messages JSON,
        banned_users JSON,
        muted_users JSON)";

    db.execute(messages_table, &[]).unwrap();
    db.execute(users_table, &[]).unwrap();
    db.execute(rooms_table, &[]).unwrap();
}

#[derive(Serialize, Deserialize)]
struct Content {
    message: String,
    name: String,
}

pub fn save_message(msg: chatserver::Message) {
    // Prepare the message for saving
    let content: Content = serde_json::from_str(&msg.msg).unwrap();

    // Add it to the messages table
    let db = connect();
    db.execute(
        "INSERT INTO messages (sender_name, msg, room) VALUES ($1, $2, $3)",
        &[&content.name, &content.message, &msg.room],
    ).unwrap();
}
