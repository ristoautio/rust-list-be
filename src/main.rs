use actix_web::{App, get, HttpResponse, HttpServer, middleware::Logger, Responder, web};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;

use handlers::create;
use handlers::get_all;

mod config {
    pub use ::config::ConfigError;
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Config {
        pub server_addr: String,
        pub pg: deadpool_postgres::Config,
    }

    impl Config {
        pub fn from_env() -> Result<Self, ConfigError> {
            let mut cfg = ::config::Config::new();
            cfg.merge(::config::Environment::new())?;
            cfg.try_into()
        }
    }
}

mod models {
    use serde::{Deserialize, Serialize};
    use tokio_pg_mapper_derive::PostgresMapper;

    #[derive(Deserialize, PostgresMapper, Serialize)]
    #[pg_mapper(table = "list")]
    pub struct List {
        pub id: i32,
        pub name: String,
    }
}


mod errors {
    use actix_web::{HttpResponse, ResponseError};
    use deadpool_postgres::PoolError;
    use derive_more::{Display, From};
    use tokio_pg_mapper::Error as PGMError;
    use tokio_postgres::error::Error as PGError;

    #[derive(Display, From, Debug)]
    pub enum MyError {
        NotFound,
        PGError(PGError),
        PGMError(PGMError),
        PoolError(PoolError),
    }

    impl std::error::Error for MyError {}

    impl ResponseError for MyError {
        fn error_response(&self) -> HttpResponse {
            match *self {
                MyError::NotFound => HttpResponse::NotFound().finish(),
                MyError::PoolError(ref err) => {
                    HttpResponse::InternalServerError().body(err.to_string())
                }
                _ => HttpResponse::InternalServerError().finish(),
            }
        }
    }
}


mod db {
    use deadpool_postgres::Client;
    use tokio_pg_mapper::FromTokioPostgresRow;

    use crate::{errors::MyError, models::List};

    pub async fn get_all(client: &Client) -> Result<Vec<List>, MyError> {
        let _stmt = "select * from list";
        let stmt = client.prepare(&_stmt).await.unwrap();

        let result = client.query(&stmt, &[])
            .await?
            .iter()
            .map(|row| List::from_row_ref(row).unwrap())
            .collect::<Vec<List>>();
        std::result::Result::Ok(result)
    }

    pub async fn create(client: &Client, name: String) -> Result<bool, MyError> {
        let _stmt = "INSERT INTO list (id, name) VALUES (nextval('list_id_seq'), $1)";
        let stmt = client.prepare(&_stmt).await.unwrap();

        client.execute(&stmt, &[&name],).await?;
        std::result::Result::Ok(true)
    }
}


#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[derive(Serialize, Deserialize)]
struct MyObj {
    name: String,
    foo: String,
}

#[derive(Deserialize)]
pub struct Create {
    name: String,
}

mod handlers {
    use actix_web::{Error, HttpResponse, web};
    use deadpool_postgres::{Client, Pool};

    use crate::{Create, db, errors::MyError, MyObj};

    pub async fn get_all(
        db_pool: web::Data<Pool>,
    ) -> Result<HttpResponse, Error> {
        let client: Client = db_pool.get().await.map_err(MyError::PoolError)?;
        let all = db::get_all(&client).await?;
        Ok(HttpResponse::Ok().json(all))
    }

    pub async fn create(
        db_pool: web::Data<Pool>,
        create: web::Json<Create>,
    ) -> Result<HttpResponse, Error> {
        let client: Client = db_pool.get().await.map_err(MyError::PoolError)?;
        db::create(&client, create.name.to_string()).await?;

        Ok(HttpResponse::Ok().json(MyObj {
            name: create.name.to_string(),
            foo: "".to_string()
        }))
    }

}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let config = crate::config::Config::from_env().unwrap();
    let pool = config.pg.create_pool(NoTls).unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .data(pool.clone())
            .service(health)
            .service(web::resource("/lists")
                .route(web::get().to(get_all))
                .route(web::post().to(create)))
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}


