//! https://github.com/rust-awesome-app/template-app-base/blob/main/src-tauri/src/error.rs

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Surreal(#[from] surrealdb::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
}
