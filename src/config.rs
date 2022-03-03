use config::{self};
use anyhow::Result;
use std::sync::{RwLock, RwLockReadGuard};

use crate::errors::MyError;

pub struct Cfg(RwLock<config::Config>);

impl Cfg {
    pub fn new() -> Result<Self> {
        let settings = config::Config::builder()
            .add_source((config::File::with_name("config")))
            .add_source(config::Environment::with_prefix("WEDNESDAY"))
            .build()?;
        Ok(Self(RwLock::new(settings)))
    }

    pub fn read(&self) -> Result<RwLockReadGuard<config::Config>> {
        Ok(self.0.read().map_err(|_e| MyError::ConfigLockError)?)
    }

    // pub fn write(&self) -> Result<RwLockWriteGuard<config::Config>> {
    //     Ok(self.0.write().map_err(|_e| MyError::ConfigLockError)?)
    // }

    pub fn bot_name(&self) -> Result<String> {
        Ok(self.read()?.get_string("bot_name")?)
    }

    pub fn token(&self) -> Result<String> {
        Ok(self.read()?.get_string("token")?)
    }

    pub fn sentry_url(&self) -> Result<String> {
        Ok(self.read()?.get_string("sentry_url")?)
    }

    pub fn coin_market_api_key(&self) -> Result<String> {
        Ok(self.read()?.get_string("coin_market_api_key")?)
    }

    pub fn db(&self) -> Result<String> {
        Ok(self.read()?.get_string("db")?)
    }

    pub fn cache(&self) -> Result<String> {
        Ok(self.read()?.get_string("cache")?)
    }
}
