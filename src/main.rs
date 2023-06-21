use rocket::{get,launch, routes};
use tokio_postgres::{ NoTls, Error};
use rocket::response::Debug;
use std::env;

async fn init_db() -> Result<(), Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls)
        .await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    client.batch_execute(
        "
        CREATE TABLE IF NOT EXISTS messages (
            id SERIAL PRIMARY KEY,
            message VARCHAR UNIQUE NOT NULL
            )
    ",
    ).await?;
    client.execute(
        "INSERT INTO messages (message) VALUES ($1) ON CONFLICT DO NOTHING",
        &[&"Hello, Docker!"],
    ).await?;
    Ok(())
}

#[get("/<id>")]
async fn index(id: i32) -> Result<String, Debug<tokio_postgres::Error>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls)
        .await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    let result = client.query_opt("SELECT message FROM messages WHERE id = $1", &[&id]).await?;
    match result {
      Some(row) => {
          let message: &str = row.get(0);
          Ok(message.to_string())
      }
      None => {
          Ok("Not found".to_string())
      }
    }
}

#[launch]
fn rocket() -> _ {
    init_db();
    rocket::build().mount("/", routes![index])
}
