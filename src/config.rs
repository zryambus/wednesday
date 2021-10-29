use config;
use anyhow::Result;
use std::sync::{RwLock, RwLockReadGuard};

use crate::errors::MyError;

pub struct Cfg(RwLock<config::Config>);

impl Cfg {
    pub fn new() -> Result<Self> {
        let mut settings = config::Config::default();
        settings
            .merge(config::File::with_name("config"))?
            .merge(config::Environment::with_prefix("WEDNESDAY"))?;
        Ok(Self(RwLock::new(settings)))
    }

    pub fn read(&self) -> Result<RwLockReadGuard<config::Config>> {
        Ok(self.0.read().map_err(|_e| MyError::ConfigLockError)?)
    }

    // pub fn write(&self) -> Result<RwLockWriteGuard<config::Config>> {
    //     Ok(self.0.write().map_err(|_e| MyError::ConfigLockError)?)
    // }
}
