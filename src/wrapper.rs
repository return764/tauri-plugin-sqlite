use std::fs::create_dir_all;
use std::str::FromStr;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sqlx::{Column, Executor, Pool, Row, Sqlite};
use sqlx::migrate::{MigrateDatabase, Migrator};
use sqlx::query::Query;
use sqlx::sqlite::{SqliteArguments, SqliteConnectOptions};
use serde_json::Value as JsonValue;
use tauri::{AppHandle, Manager, Runtime};
use crate::{decode, Error};

#[derive(Debug, Deserialize, Serialize)]
pub struct ConnectOptions {
    pub db_url: String,
    extensions: Option<Vec<String>>
}

impl ConnectOptions {
    pub fn from_url(db_url: String) -> Self {
        Self {
            db_url,
            extensions: None
        }
    }
}

pub trait DbPool {
    async fn connect<R: Runtime>(
        _app: &AppHandle<R>,
        options: &ConnectOptions
    ) -> Result<Box<Self>, Error>;

    async fn migrate(
        &self,
        migrator: &Migrator,
    ) -> Result<(), Error>;

    async fn execute(
        &self,
        query: String,
        values: Vec<JsonValue>,
    ) -> Result<(u64, i64), Error>;

    async fn select(
        &self,
        query: String,
        values: Vec<JsonValue>,
    ) -> Result<Vec<IndexMap<String, JsonValue>>, Error>;
}

impl DbPool for Pool<Sqlite> {
    async fn connect<R: Runtime>(app: &AppHandle<R>, options: &ConnectOptions) -> Result<Box<Pool<Sqlite>>, Error> {
        let app_path = app
            .path()
            .app_config_dir()
            .expect("No App config path was found!");
        let conn_url = &options.db_url;

        create_dir_all(&app_path).expect("Couldn't create app config dir");

        let conn_url = &path_mapper(app_path, conn_url);

        if !Sqlite::database_exists(conn_url).await.unwrap_or(false) {
            Sqlite::create_database(conn_url).await?;
        }

        let mut connect_options = SqliteConnectOptions::from_str(conn_url.as_str())?;
        if let Some(extensions) = &options.extensions {
            for extension in extensions {
                connect_options = connect_options.extension(extension.clone());
            }
        }

        Ok(Box::new(Pool::connect_with(connect_options).await?))
    }

    async fn migrate(&self, migrator: &Migrator) -> Result<(), Error> {
        Ok(migrator.run(self).await?)
    }

    async fn execute(&self, query: String, values: Vec<JsonValue>) -> Result<(u64, i64), Error> {
        let mut query = sqlx::query(&query);
        query = bind_query(query, values);
        let result = Executor::execute(self, query).await?;

        Ok((
            result.rows_affected(),
            result.last_insert_rowid()
        ))
    }

    async fn select(&self, query: String, values: Vec<JsonValue>) -> Result<Vec<IndexMap<String, JsonValue>>, Error> {
        let mut query = sqlx::query(&query);
        query = bind_query(query, values);
        let rows = self.fetch_all(query).await?;
        let mut values = Vec::new();
        for row in rows {
            let mut value = IndexMap::default();
            for (i, column) in row.columns().iter().enumerate() {
                let v = row.try_get_raw(i)?;

                let v = decode::to_json(v)?;

                value.insert(column.name().to_string(), v);
            }

            values.push(value);
        }
        Ok(values)
    }
}

fn bind_query<'a>(mut query: Query<'a, Sqlite, SqliteArguments<'a>>, values: Vec<JsonValue>) -> Query<'a, Sqlite, SqliteArguments<'a>> {
    for value in values {
        if value.is_null() {
            query = query.bind(None::<JsonValue>);
        } else if value.is_string() {
            query = query.bind(value.as_str().unwrap().to_owned())
        } else if let Some(number) = value.as_number() {
            query = query.bind(number.as_f64().unwrap_or_default())
        } else {
            query = query.bind(value);
        }
    }
    query
}

fn path_mapper(mut app_path: std::path::PathBuf, connection_string: &str) -> String {
    app_path.push(
        connection_string
            .split_once(':')
            .expect("Couldn't parse the connection string for DB!")
            .1,
    );

    format!(
        "sqlite:{}",
        app_path
            .to_str()
            .expect("Problem creating fully qualified path to Database file!")
    )
}
