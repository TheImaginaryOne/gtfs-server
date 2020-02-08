use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use std::env;

pub fn create_connection_pool() -> ConnectionPool {
    let url = &env::var("DATABASE_URL").expect("DATABASE_URL must be defined");
    let manager = ConnectionManager::<DbConnection>::new(url);
    Pool::builder()
        .build(manager)
        .unwrap_or_else(|_| panic!("Could not create pool for database: {}", &url))
}

pub type DbConnection = PgConnection;
pub type ConnectionPool = Pool<ConnectionManager<DbConnection>>;
