use actix_web::{App, get, HttpResponse, HttpServer, middleware::Logger, Responder, web};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;

use handlers::create;
use handlers::get_all;

mod config;
mod db;

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
        let all = db::db::get_all(&client).await?;
        Ok(HttpResponse::Ok().json(all))
    }

    pub async fn create(
        db_pool: web::Data<Pool>,
        create: web::Json<Create>,
    ) -> Result<HttpResponse, Error> {
        let client: Client = db_pool.get().await.map_err(MyError::PoolError)?;
        db::db::create(&client, create.name.to_string()).await?;

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
    let config = config::config::Config::from_env().unwrap();
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


