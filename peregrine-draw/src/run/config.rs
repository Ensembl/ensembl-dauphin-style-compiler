use std::num::ParseFloatError;
use peregrine_data::{ DataMessage };
use crate::util::message::Message;
use lazy_static::lazy_static;
use peregrine_config::{ Config, ConfigKeyInfo, ConfigValue, ConfigError };
use crate::input::InputEventKind;

// XXX factor with similar in peregrine-data
// XXX chromosome ned-stops

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum PgConfigKey {
    FadeOverlap(bool),
    AnimationFadeRate(bool),
    KeyBindings(InputEventKind),
    PullMaxSpeed, // screenfulls/frame
    PullAcceleration, // screenfulls/frame/frame
    ZoomMaxSpeed, // factors-of-2/second,
    ZoomAcceleration, // factors-of-2/second/second,
    ZoomPixelSpeed, // how many pixels is a doubling?
    MouseClickRadius, // px (click vs drag)
    MouseHoldDwell, // ms (click vs hold)
    DoubleClickTime, // ms, how long a gap to not be part of double click
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
            ConfigKeyInfo { key: PgConfigKey::AnimationFadeRate(true), name: "animate.fade.fast", default: &PgConfigValue::Float(200.) },
            ConfigKeyInfo { key: PgConfigKey::AnimationFadeRate(false), name: "animate.fade.slow", default: &PgConfigValue::Float(1000.) },    
            ConfigKeyInfo { key: PgConfigKey::FadeOverlap(true), name: "animate.overlap.fast", default: &PgConfigValue::Float(-0.75) },
            ConfigKeyInfo { key: PgConfigKey::FadeOverlap(false), name: "animate.overlap.slow", default: &PgConfigValue::Float(3.) },
            ConfigKeyInfo { key: PgConfigKey::ZoomPixelSpeed, name: "animate.zoom-pixel-peed", default: &PgConfigValue::Float(500.) },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::DebugAction), name: "keys.debug-action", default: &PgConfigValue::StaticStr("Click[1] Drag[2] Hold[3] HoldDrag[4] RunningHold[5] SwitchToHold[6] DoubleClick[7] 1[1] 2[2] 3[3] 4[4] 5[5] 6[6] 7[7] 8[8] 9[9] 0[0]") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PixelsLeft), name: "keys.pixels-left", default: &PgConfigValue::StaticStr("Shift-A[100]") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PixelsRight), name: "keys.pixels-right", default: &PgConfigValue::StaticStr("MirrorRunningDrag Shift-D[100]") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PixelsIn), name: "keys.pixels-in", default: &PgConfigValue::StaticStr("Shift-W[500]") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PixelsOut), name: "keys.pixels-out", default: &PgConfigValue::StaticStr("Shift-S[500] Wheel") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PullLeft), name: "keys.pull-left", default: &PgConfigValue::StaticStr("a ArrowLeft") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PullRight), name: "keys.pull-right", default: &PgConfigValue::StaticStr("d ArrowRight") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PullIn), name: "keys.pull-in", default: &PgConfigValue::StaticStr("w ArrowUp") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PullOut), name: "keys.pull-out", default: &PgConfigValue::StaticStr("s ArrowDown") },
            ConfigKeyInfo { key: PgConfigKey::DoubleClickTime, name: "mouse.doubleclick-time", default: &PgConfigValue::Float(500.) },
            ConfigKeyInfo { key: PgConfigKey::MouseClickRadius, name: "mouse.click-radius", default: &PgConfigValue::Float(4.) },
            ConfigKeyInfo { key: PgConfigKey::MouseHoldDwell, name: "mouse.hold-dwell", default: &PgConfigValue::Float(1250.) },
            ConfigKeyInfo { key: PgConfigKey::PullMaxSpeed, name: "pull.max-speed", default: &PgConfigValue::Float(1./40.) },
            ConfigKeyInfo { key: PgConfigKey::PullAcceleration, name: "pull.acceleration", default: &PgConfigValue::Float(1./40000.) },
            ConfigKeyInfo { key: PgConfigKey::ZoomMaxSpeed, name: "zoom.max-speed", default: &PgConfigValue::Float(1./10.) },
            ConfigKeyInfo { key: PgConfigKey::ZoomAcceleration, name: "zoom.acceleration", default: &PgConfigValue::Float(1./10000.) },
        ]};
}

fn string_to_float(value_str: &str) -> Result<f64,String> {
    value_str.parse().map_err(|e: ParseFloatError| e.to_string())
}

// XXX macroise
impl PgConfigValue {
    fn as_f64(&self) -> Result<f64,Message> {
        match self {
            PgConfigValue::Float(x) => Ok(*x),
            _ => Err(Message::DataError(DataMessage::CodeInvariantFailed(format!("cannot get value as f64"))))
        }
    }

    fn try_as_str(&self) -> Option<&str> {
        match self {
            PgConfigValue::String(x) => Some(x),
            PgConfigValue::StaticStr(x) => Some(x),
            _ => None
        }
    }

    fn as_str(&self) -> Result<&str,Message> {
        if let Some(v) = self.try_as_str() { return Ok(v); }
        Err(Message::DataError(DataMessage::CodeInvariantFailed(format!("cannot get value as str"))))
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
    pub fn try_get_str(&self, key: &PgConfigKey) -> Option<&str> { self.0.try_get(key).and_then(|x| x.try_as_str()) }
    pub fn get_str(&self, key: &PgConfigKey) -> Result<&str,Message> { self.get(key)?.as_str() }
}
