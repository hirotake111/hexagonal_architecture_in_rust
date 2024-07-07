use std::env;

use dotenvy::dotenv;
use hexagonal_architecture_in_rust::repository::Postgres;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();
    let database_url =
        env::var("DATABASE_URL").expect("Expected environment variable DATABASE_URL");
    let _repo = Postgres::new(&database_url).await;
    Ok(())
}
