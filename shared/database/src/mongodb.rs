use anyhow::Result;
use mongodb::{Client, Database};

pub type MongoClient = Client;
pub type MongoDatabase = Database;

pub async fn create_mongo_client(database_url: &str) -> Result<MongoClient> {
    let client = Client::with_uri_str(database_url).await?;
    
    // Test connection
    client
        .database("admin")
        .run_command(mongodb::bson::doc! {"ping": 1}, None)
        .await?;
    
    tracing::info!("Connected to MongoDB database");
    Ok(client)
}

pub fn get_database(client: &MongoClient, database_name: &str) -> MongoDatabase {
    client.database(database_name)
}

pub async fn health_check(client: &MongoClient) -> Result<()> {
    client
        .database("admin")
        .run_command(mongodb::bson::doc! {"ping": 1}, None)
        .await?;
    Ok(())
}