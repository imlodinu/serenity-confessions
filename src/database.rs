use sea_orm::{self, ConnectOptions};

static mut URL: Option<String> = None;

pub fn initialise(secret_store: &shuttle_secrets::SecretStore) {
    unsafe {
        URL = Some(secret_store.get("DATABASE_URL").unwrap());
    }
}

pub async fn connect() -> Option<sea_orm::DatabaseConnection> {
    unsafe {
        let mut opt = ConnectOptions::new(URL.clone().unwrap());
        opt.sqlx_logging_level(tracing::log::LevelFilter::Debug)
            .sqlx_logging(true);
        sea_orm::Database::connect(opt).await.ok()
    }
}
