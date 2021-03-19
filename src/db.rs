mod models {
    use serde::{Deserialize, Serialize};
    use tokio_pg_mapper_derive::PostgresMapper;

    #[derive(Deserialize, PostgresMapper, Serialize)]
    #[pg_mapper(table = "list")]
    pub struct List {
        pub id: i32,
        pub name: String,
        pub deleted: bool,
    }
}

pub mod db {
    use deadpool_postgres::Client;
    use tokio_pg_mapper::FromTokioPostgresRow;

    use crate::{errors::MyError};
    use crate::db::models::List;

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

    pub async fn get_items(client: &Client, list_id: i32) -> Result<Vec<List>, MyError> {
        let _stmt = "select * from list_item where list_id = $1 order by deleted asc";
        let stmt = client.prepare(&_stmt).await.unwrap();

        let result = client.query(&stmt, &[&list_id])
            .await?
            .iter()
            .map(|row| List::from_row_ref(row).unwrap())
            .collect::<Vec<List>>();
        std::result::Result::Ok(result)
    }

    pub async fn add_item(client: &Client, list_id: i32, name: String) -> Result<bool, MyError> {
        let _stmt = "INSERT INTO list_item (id, list_id, name) VALUES (nextval('list_item_seq'), $1, $2)";
        let stmt = client.prepare(&_stmt).await.unwrap();

        client.execute(&stmt, &[&list_id, &name],).await?;
        std::result::Result::Ok(true)
    }

    pub async fn remove_item(client: &Client, list_id: i32, id: i32) -> Result<(), MyError> {
        let _stmt = "UPDATE list_item SET deleted=true where list_id = $1 and id = $2";
        let stmt = client.prepare(&_stmt).await.unwrap();

        client.execute(&stmt, &[&list_id, &id],).await?;
        std::result::Result::Ok(())
    }

    pub async fn init(client: &Client) -> Result<(), MyError>{
        client.batch_execute("
            CREATE TABLE IF NOT EXISTS list (
                id int primary key not null,
                deleted boolean not null default false,
                name varchar
            );

            CREATE TABLE IF NOT EXISTS list_item (
                id int primary key not null,
                name varchar not null,
                deleted boolean not null default false,
                list_id int not null references list(id)
            );

            CREATE SEQUENCE IF NOT EXISTS list_id_seq START 101;
            CREATE SEQUENCE IF NOT EXISTS list_item_seq START 101;
        ").await?;

        Ok(())
    }
}
