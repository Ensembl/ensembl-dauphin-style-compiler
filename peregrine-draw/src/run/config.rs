use std::num::ParseFloatError;
use peregrine_data::{ DataMessage };
use crate::util::message::Message;
use lazy_static::lazy_static;
use peregrine_config::{ Config, ConfigKeyInfo, ConfigValue, ConfigError };
use crate::input::InputEventKind;

// XXX factor with similar in peregrine-data
// XXX chromosome ned-stops

#[derive(Clone,PartialEq,Eq,Hash)]
pub enum PgConfigKey {
    KeyBindings(InputEventKind),
    PullMaxSpeed, // screenfulls/frame
    PullAccelleration, // screenfulls/frame/frame
    ZoomMaxSpeed, // factors-of-2/second,
    ZoomAccelleration, // factors-of-2/second/second,
    FadeOverlapProp,
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
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PullIn), name: "keys.pull-in", default: &PgConfigValue::StaticStr("w W") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PullOut), name: "keys.pull-out", default: &PgConfigValue::StaticStr("s S") },
            ConfigKeyInfo { key: PgConfigKey::PullMaxSpeed, name: "pull.max-speed", default: &PgConfigValue::Float(1./60.) }, // 1 screen/second
            ConfigKeyInfo { key: PgConfigKey::PullAccelleration, name: "pull.accelleration", default: &PgConfigValue::Float(1./72000.) }, // reach 1 screen/second^2 in 20s 1200frames ie 1/60 screen/frame in 1200 frames
            ConfigKeyInfo { key: PgConfigKey::ZoomMaxSpeed, name: "zoom.max-speed", default: &PgConfigValue::Float(2.) },
            ConfigKeyInfo { key: PgConfigKey::ZoomAccelleration, name: "zoom.accelleration", default: &PgConfigValue::Float(1./30000.) }, // reach 2 factors/second in 10s, ie in 600 frames
            ConfigKeyInfo { key: PgConfigKey::FadeOverlapProp, name: "transition.fade-overlap", default: &PgConfigValue::Float(0.1) },
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


fn map_error<R>(e: Result<R,ConfigError>) -> Result<R,Message> {
    e.map_err(|e| Message::DataError(DataMessage::ConfigError(e)))
}
pub struct PgPeregrineConfig<'a>(Config<'a,PgConfigKey,PgConfigValue>);

impl<'a> PgPeregrineConfig<'a> {
    pub fn new() -> PgPeregrineConfig<'a> {
        PgPeregrineConfig(Config::new(&CONFIG_CONFIG))
    }

    pub fn set(&mut self, key_str: &str, value: &str) -> Result<(),Message> {
        map_error(self.0.set(key_str,value))
    }

    fn get(&self, key: &PgConfigKey) -> Result<&PgConfigValue,Message> { map_error(self.0.get(key)) }

    pub fn get_f64(&self, key: &PgConfigKey) -> Result<f64,Message> { self.get(key)?.as_f64() }
    pub fn get_str(&self, key: &PgConfigKey) -> Result<&str,Message> { self.get(key)?.as_str() }
}
