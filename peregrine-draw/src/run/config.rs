use std::num::ParseFloatError;
use peregrine_data::{ DataMessage };
use crate::util::message::Message;
use lazy_static::lazy_static;
use peregrine_config::{ Config, ConfigKeyInfo, ConfigValue };
use crate::input::InputEventKind;

#[derive(Clone,PartialEq,Eq,Hash)]
pub enum PgConfigKey {
    KeyBindings(InputEventKind)
}

#[derive(Clone)]
pub enum PgConfigValue {
    Float(f64),
    String(String),
    StaticStr(&'static str)
}

lazy_static! {
    static ref CONFIG_CONFIG : Vec<ConfigKeyInfo<'static,PgConfigKey,PgConfigValue>> = {
        vec![
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PullLeft), name: "keys.pull-left", default: &PgConfigValue::StaticStr("a A") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PullRight), name: "keys.pull-right", default: &PgConfigValue::StaticStr("d D") },
        ]};
}

fn string_to_float(value_str: &str) -> Result<f64,String> {
    value_str.parse().map_err(|e: ParseFloatError| e.to_string())
}

impl PgConfigValue {
    fn as_f64(&self) -> Result<f64,Message> {
        match self {
            PgConfigValue::Float(x) => Ok(*x),
            _ => Err(Message::DataError(DataMessage::CodeInvariantFailed(format!("cannot get value as f64"))))
        }
    }

    fn as_str(&self) -> Result<&str,Message> {
        match self {
            PgConfigValue::String(x) => Ok(x),
            PgConfigValue::StaticStr(x) => Ok(x),
            _ => Err(Message::DataError(DataMessage::CodeInvariantFailed(format!("cannot get value as str"))))
        }
    }
}

impl ConfigValue for PgConfigValue {
    fn parse(&self, value_str: &str) -> Result<PgConfigValue,String> {
        Ok(match self {
            PgConfigValue::Float(_) => PgConfigValue::Float(string_to_float(value_str)?),
            PgConfigValue::String(_) => PgConfigValue::String(value_str.to_string()),
            PgConfigValue::StaticStr(_) => PgConfigValue::String(value_str.to_string())
        })
    }
}

pub struct PgPeregrineConfig<'a>(Config<'a,PgConfigKey,PgConfigValue>);

impl<'a> PgPeregrineConfig<'a> {
    pub fn new() -> PgPeregrineConfig<'a> {
        PgPeregrineConfig(Config::new(&CONFIG_CONFIG))
    }

    pub fn set(&mut self, key_str: &str, value: &str) -> Result<(),DataMessage> {
        self.0.set(key_str,value).map_err(|e| DataMessage::ConfigError(e))
    }

    fn get(&self, key: &PgConfigKey) -> &PgConfigValue { self.0.get(key) }

    pub fn get_f64(&self, key: &PgConfigKey) -> Result<f64,Message> { self.get(key).as_f64() }
    pub fn get_str(&self, key: &PgConfigKey) -> Result<&str,Message> { self.get(key).as_str() }
}
