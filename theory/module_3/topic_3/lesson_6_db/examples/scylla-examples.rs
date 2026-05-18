// use scylla::{Session, SessionBuilder, IntoTypedRows};
// use serde::{Deserialize, Serialize};

// #[derive(Debug, Clone, Serialize, Deserialize)]
// struct Event {
//     user_id: i64,
//     event_time: i64,
//     event_type: String,
//     data: String,
// }

// async fn create_scylla_session(uri: &str) -> anyhow::Result<Session> {
//     Ok(SessionBuilder::new().known_node(uri).build().await?)
// }

// async fn create_events_table(session: &Session) -> anyhow::Result<()> {
//     session
//         .query(
//             "CREATE TABLE IF NOT EXISTS events(
//                 user_id bigint,
//                 event_time bigint,
//                 event_type text,
//                 data text,
//                 PRIMARY KEY (user_id, event_time)
//             ) WITH CLUSTERING ORDER BY (event_time DESC)",
//             &[],
//         )
//         .await?;
//     Ok(())
// }

// async fn insert_event(session: &Session, event: Event) -> anyhow::Result<()> {
//     session
//         .query(
//             "INSERT INTO events (user_id, event_time, event_type, data) VALUES (?, ?, ?, ?)",
//             (&event.user_id, &event.event_time, &event.event_type, &event.data),
//         )
//         .await?;
//     Ok(())
// }

// async fn get_user_events(session: &Session, user_id: i64) -> anyhow::Result<Vec<Event>> {
//     let rs = session
//         .query(
//             "SELECT user_id, event_time, event_type, data FROM events WHERE user_id = ?",
//             (user_id,),
//         )
//         .await?;
    
//     let mut events = Vec::new();
//     if let Some(rows) = rs.rows {
//         for row in rows.into_typed::<(i64, i64, String, String)>() {
//             let (user_id, event_time, event_type, data) = row?;
//             events.push(Event { user_id, event_time, event_type, data });
//         }
//     }
//     Ok(events)
// }

// // Prepared statements для производительности
// async fn prepare_insert_event(session: &Session) -> anyhow::Result<scylla::prepared_statement::PreparedStatement> {
//     let prepared = session
//         .prepare("INSERT INTO events (user_id, event_time, event_type, data) VALUES (?, ?, ?, ?)")
//         .await?;
//     Ok(prepared)
// }

fn main() {
    todo!()
}