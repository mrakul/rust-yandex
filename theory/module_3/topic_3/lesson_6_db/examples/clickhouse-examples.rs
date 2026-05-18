// use clickhouse::{Client, Row};
// use serde::{Deserialize, Serialize};

// #[derive(Debug, Row, Serialize, Deserialize)]
// struct Event {
//     user_id: u64,
//     event_time: chrono::DateTime<chrono::Utc>,
//     event_type: String,
//     data: String,
// }

// fn create_clickhouse_client() -> Client {
//     Client::default()
//         .with_url("http://localhost:8123")
//         .with_database("default")
//         .with_user("default")
//         .with_password("") // dev only
// }

// async fn create_events_table(client: &Client) -> clickhouse::error::Result<()> {
//     client
//         .query(
//             r#"
//             CREATE TABLE IF NOT EXISTS events (
//               user_id UInt64,
//               event_time DateTime,
//               event_type String,
//               data String
//             ) ENGINE = MergeTree
//             PARTITION BY toYYYYMM(event_time)
//             ORDER BY (user_id, event_time)
//             "#,
//         )
//         .execute()
//         .await?;
//     Ok(())
// }

// async fn insert_events_batch(client: &Client, events: Vec<Event>) -> clickhouse::error::Result<()> {
//     let mut insert = client.insert("events")?;
//     for ev in events {
//         insert.write(&ev).await?;
//     }
//     insert.end().await?;
//     Ok(())
// }

// async fn get_user_events_count(client: &Client, user_id: u64) -> clickhouse::error::Result<u64> {
//     let count: u64 = client
//         .query("SELECT count() FROM events WHERE user_id = ?")
//         .bind(user_id)
//         .fetch_one()
//         .await?;
//     Ok(count)
// }

// async fn get_daily_stats(client: &Client, date: chrono::NaiveDate) -> clickhouse::error::Result<Vec<(String, u64)>> {
//     #[derive(Row, Deserialize)]
//     struct StatRow {
//         event_type: String,
//         cnt: u64,
//     }
    
//     let stats: Vec<StatRow> = client
//         .query(
//             "SELECT event_type, count() as cnt 
//              FROM events 
//              WHERE toDate(event_time) = ? 
//              GROUP BY event_type 
//              ORDER BY cnt DESC"
//         )
//         .bind(date)
//         .fetch_all()
//         .await?;
    
//     Ok(stats.into_iter().map(|s| (s.event_type, s.cnt)).collect())
// }

fn main() {
    todo!()
}