use std::{num::ParseFloatError};
use crate::util::message::DataMessage;
use lazy_static::lazy_static;
use peregrine_config::{Config, ConfigError, ConfigKeyInfo, ConfigValue};

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum ConfigKey {
}

#[derive(Clone)]
pub enum PgdConfigValue {
    Float(f64)
}

// XXX move
lazy_static! {
    static ref CONFIG_CONFIG : Vec<ConfigKeyInfo<'static,ConfigKey,PgdConfigValue>> = vec![
    ];
}

fn string_to_float(value_str: &str) -> Result<f64,String> {
    value_str.parse().map_err(|e: ParseFloatError| e.to_string())
}

impl PgdConfigValue {
    fn as_f64(&self) -> Result<f64,DataMessage> {
        match self {
            PgdConfigValue::Float(x) => Ok(*x),
            _ => Err(DataMessage::CodeInvariantFailed(format!("cannot get value as f64")))
        }
    }
}

impl ConfigValue for PgdConfigValue {
    fn parse(&self, value_str: &str) -> Result<PgdConfigValue,String> {
        Ok(match self {
            PgdConfigValue::Float(_) => PgdConfigValue::Float(string_to_float(value_str)?)
        })
    }
}

fn map_error<R>(e: Result<R,ConfigError>) -> Result<R,DataMessage> {
    e.map_err(|e| DataMessage::ConfigError(e))
}

pub struct PgdPeregrineConfig<'a>(Config<'a,ConfigKey,PgdConfigValue>);

impl<'a> PgdPeregrineConfig<'a> {
    pub fn new() -> PgdPeregrineConfig<'a> {
        PgdPeregrineConfig(Config::new(&CONFIG_CONFIG))
    }

    pub fn set(&mut self, key_str: &str, value: &str) -> Result<(),DataMessage> {
        map_error(self.0.set(key_str,value))
    }

    fn get(&self, key: &ConfigKey) -> Result<&PgdConfigValue,DataMessage> {
        map_error(self.0.get(key))
    }

    pub fn get_f64(&self, key: &ConfigKey) -> Result<f64,DataMessage> { self.get(key)?.as_f64() }
}
