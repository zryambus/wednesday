use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Config lock error")]
    ConfigLockError,
}
