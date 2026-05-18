// use mongodb::{Client, Collection};
// use mongodb::bson::{doc, Document};
// use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize)]
// struct User {
//     #[serde(rename = "_id")]
//     id: Option<mongodb::bson::oid::ObjectId>,
//     email: String,
//     name: String,
//     created_at: chrono::DateTime<chrono::Utc>,
// }

// async fn create_mongo_client(connection_string: &str) -> Result<Client, mongodb::error::Error> {
//     let client = Client::with_uri_str(connection_string).await?;
//     Ok(client)
// }

// async fn insert_user(collection: &Collection<User>, email: String, name: String) -> Result<mongodb::bson::oid::ObjectId, mongodb::error::Error> {
//     let user = User {
//         id: None,
//         email,
//         name,
//         created_at: chrono::Utc::now(),
//     };
    
//     let result = collection.insert_one(user).await?;
//     Ok(result.inserted_id.as_object_id().unwrap())
// }

// async fn find_user_by_email(collection: &Collection<User>, email: &str) -> Result<Option<User>, mongodb::error::Error> {
//     let filter = doc! { "email": email };
//     let user = collection.find_one(filter).await?;
//     Ok(user)
// }

// async fn update_user_name(collection: &Collection<User>, email: &str, new_name: String) -> Result<bool, mongodb::error::Error> {
//     let filter = doc! { "email": email };
//     let update = doc! { "$set": { "name": new_name } };
//     let result = collection.update_one(filter, update).await?;
//     Ok(result.modified_count > 0)
// }

// // Создание индекса
// async fn create_email_index(collection: &Collection<User>) -> Result<(), mongodb::error::Error> {
//     let index_model = mongodb::IndexModel::builder()
//         .keys(doc! { "email": 1 })
//         .options(mongodb::options::IndexOptions::builder()
//             .unique(true)
//             .build())
//         .build();
    
//     collection.create_index(index_model).await?;
//     Ok(())
// }

fn main() {
    todo!()
}