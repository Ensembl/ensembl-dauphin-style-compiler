use std::num::{ParseFloatError, ParseIntError};
use peregrine_data::{ DataMessage };
use crate::util::message::Message;
use lazy_static::lazy_static;
use peregrine_config::{ Config, ConfigKeyInfo, ConfigValue, ConfigError };
use crate::input::InputEventKind;

// XXX factor with similar in peregrine-data
// XXX chromosome ned-stops

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum CursorCircumstance {
    Default,
    Drag,
    Hold,
    Pinch,
    WheelNegative,
    WheelPositive
}

impl CursorCircumstance {
    pub(crate) fn each() -> Vec<CursorCircumstance> {
        vec![
            CursorCircumstance::Default,
            CursorCircumstance::Drag,
            CursorCircumstance::Hold,
            CursorCircumstance::Pinch,
            CursorCircumstance::WheelNegative,
            CursorCircumstance::WheelPositive
        ]
    }
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum DebugFlag {
    ShowIncomingMessages
}

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
    Cursor(CursorCircumstance), // string, default mouse cursor
    DragCursorDelay, // ms, switch to drag cursor (ie assume not click)
    WheelTimeout, // ms, how long between wheel events to assume not a wheel
    DebugFlag(DebugFlag),
    AuxBufferSize, // 4-byte units
    PinchMinSep, // px min num pix to try to scale with super zoom-out
    // min factor where we try to calculate centre-of-zoom rather htan default, to avoid divide-by-zero.
    // smaller than 1/(px-of-a-giant-screen), but much bigger than precision of floats
    PinchMinScale,
}

#[derive(Clone)]
pub enum PgConfigValue {
    Float(f64),
    String(String),
    StaticStr(&'static str),
    Boolean(bool),
    Size(usize)
}

lazy_static! {
    static ref CONFIG_CONFIG : Vec<ConfigKeyInfo<'static,PgConfigKey,PgConfigValue>> = {
        vec![
            ConfigKeyInfo { key: PgConfigKey::AnimationFadeRate(true), name: "animate.fade.fast", default: &PgConfigValue::Float(200.) },
            ConfigKeyInfo { key: PgConfigKey::AnimationFadeRate(false), name: "animate.fade.slow", default: &PgConfigValue::Float(1000.) },    
            ConfigKeyInfo { key: PgConfigKey::FadeOverlap(true), name: "animate.overlap.fast", default: &PgConfigValue::Float(-0.75) },
            ConfigKeyInfo { key: PgConfigKey::FadeOverlap(false), name: "animate.overlap.slow", default: &PgConfigValue::Float(3.) },
            ConfigKeyInfo { key: PgConfigKey::ZoomPixelSpeed, name: "animate.zoom-pixel-peed", default: &PgConfigValue::Float(500.) },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::DebugAction), name: "keys.debug-action", default: &PgConfigValue::StaticStr("Click[1] Drag[2] Hold[3] HoldDrag[4] SwitchToHold[6] DoubleClick[7] Pinch[10] SwitchToPinch[11] 1[1] 2[2] 3[3] 4[4] 5[5] 6[6] 7[7] 8[8] 9[9] 0[0]") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PixelsLeft), name: "keys.pixels-left", default: &PgConfigValue::StaticStr("Shift-A[100]") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PixelsRight), name: "keys.pixels-right", default: &PgConfigValue::StaticStr("MirrorRunningDrag Shift-D[100]") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PixelsIn), name: "keys.pixels-in", default: &PgConfigValue::StaticStr("Shift-W[500]") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PixelsOut), name: "keys.pixels-out", default: &PgConfigValue::StaticStr("Shift-S[500] MirrorWheel") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PullLeft), name: "keys.pull-left", default: &PgConfigValue::StaticStr("a ArrowLeft") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PullRight), name: "keys.pull-right", default: &PgConfigValue::StaticStr("d ArrowRight") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PullIn), name: "keys.pull-in", default: &PgConfigValue::StaticStr("w ArrowUp") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PullOut), name: "keys.pull-out", default: &PgConfigValue::StaticStr("s ArrowDown") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::SetPosition), name: "keys.pixels-scale", default: &PgConfigValue::StaticStr("RunningPinch") },
            ConfigKeyInfo { key: PgConfigKey::DoubleClickTime, name: "mouse.doubleclick-time", default: &PgConfigValue::Float(500.) },
            ConfigKeyInfo { key: PgConfigKey::MouseClickRadius, name: "mouse.click-radius", default: &PgConfigValue::Float(4.) },
            ConfigKeyInfo { key: PgConfigKey::MouseHoldDwell, name: "mouse.hold-dwell", default: &PgConfigValue::Float(1500.) },
            ConfigKeyInfo { key: PgConfigKey::WheelTimeout, name: "mouse.wheel-timeout", default: &PgConfigValue::Float(500.) },
            ConfigKeyInfo { key: PgConfigKey::Cursor(CursorCircumstance::Default), name: "mouse.cursor.default", default: &PgConfigValue::StaticStr("default") },
            ConfigKeyInfo { key: PgConfigKey::Cursor(CursorCircumstance::WheelPositive), name: "mouse.cursor.wheel.positive", default: &PgConfigValue::StaticStr("zoom-in col-resize") },
            ConfigKeyInfo { key: PgConfigKey::Cursor(CursorCircumstance::WheelNegative), name: "mouse.cursor.wheel.negative", default: &PgConfigValue::StaticStr("zoom-in col-resize") },
            ConfigKeyInfo { key: PgConfigKey::Cursor(CursorCircumstance::Drag), name: "mouse.cursor.hold", default: &PgConfigValue::StaticStr("grabbing") },
            ConfigKeyInfo { key: PgConfigKey::Cursor(CursorCircumstance::Hold), name: "mouse.cursor.drag", default: &PgConfigValue::StaticStr("crosshair") },
            ConfigKeyInfo { key: PgConfigKey::Cursor(CursorCircumstance::Pinch), name: "mouse.cursor.pinch", default: &PgConfigValue::StaticStr("nsew-resize") },
            ConfigKeyInfo { key: PgConfigKey::DragCursorDelay, name: "mouse.drag-cursor-delay", default: &PgConfigValue::Float(100.) },
            ConfigKeyInfo { key: PgConfigKey::PullMaxSpeed, name: "pull.max-speed", default: &PgConfigValue::Float(1./40.) },
            ConfigKeyInfo { key: PgConfigKey::PullAcceleration, name: "pull.acceleration", default: &PgConfigValue::Float(1./40000.) },
            ConfigKeyInfo { key: PgConfigKey::ZoomMaxSpeed, name: "zoom.max-speed", default: &PgConfigValue::Float(1./10.) },
            ConfigKeyInfo { key: PgConfigKey::ZoomAcceleration, name: "zoom.acceleration", default: &PgConfigValue::Float(1./10000.) },
            ConfigKeyInfo { key: PgConfigKey::PinchMinSep, name: "touch.pinch-min-sep", default: &PgConfigValue::Float(16.) },
            ConfigKeyInfo { key: PgConfigKey::PinchMinScale, name: "touch.pinch-min-scale", default: &PgConfigValue::Float(1./1000000.) },
            ConfigKeyInfo { key: PgConfigKey::DebugFlag(DebugFlag::ShowIncomingMessages), name: "debug.show-incoming-messages", default: &PgConfigValue::Boolean(false) },
            ConfigKeyInfo { key: PgConfigKey::AuxBufferSize, name: "perf.aux-buffer-size", default: &PgConfigValue::Size(65536) }
        ]};
}

fn string_to_float(value_str: &str) -> Result<f64,String> {
    value_str.parse().map_err(|e: ParseFloatError| e.to_string())
}

fn string_to_usize(value_str: &str) -> Result<usize,String> {
    value_str.parse().map_err(|e: ParseIntError| e.to_string())
}

// XXX macroise
impl PgConfigValue {
    fn as_f64(&self) -> Result<f64,Message> {
        match self {
            PgConfigValue::Float(x) => Ok(*x),
            PgConfigValue::Boolean(true) => Ok(1.),
            PgConfigValue::Boolean(false) => Ok(0.),
            _ => Err(Message::DataError(DataMessage::CodeInvariantFailed(format!("cannot get value as f64"))))
        }
    }

    fn try_as_str(&self) -> Option<&str> {
        match self {
            PgConfigValue::String(x) => Some(x),
            PgConfigValue::StaticStr(x) => Some(x),
            PgConfigValue::Boolean(true) => Some("true"),
            PgConfigValue::Boolean(false) => Some("false"),
            _ => None
        }
    }

    fn as_str(&self) -> Result<&str,Message> {
        if let Some(v) = self.try_as_str() { return Ok(v); }
        Err(Message::DataError(DataMessage::CodeInvariantFailed(format!("cannot get value as str"))))
    }

    fn try_as_bool(&self) -> Option<bool> {
        match self {
            PgConfigValue::String(x) => Some(truthy(x)),
            PgConfigValue::StaticStr(x) => Some(truthy(x)),
            PgConfigValue::Boolean(x) => Some(*x),
            _ => None
        }
    }

    fn as_bool(&self) -> Result<bool,Message> {
        if let Some(v) = self.try_as_bool() { return Ok(v); }
        Err(Message::DataError(DataMessage::CodeInvariantFailed(format!("cannot get value as bool"))))
    }

    fn try_as_size(&self) -> Option<usize> {
        match self {
            PgConfigValue::Size(x) => Some(*x),
            _ => None
        }
    }

    fn as_size(&self) -> Result<usize,Message> {
        if let Some(v) = self.try_as_size() { return Ok(v); }
        Err(Message::DataError(DataMessage::CodeInvariantFailed(format!("cannot get value as size"))))
    }
}

fn truthy(value: &str) -> bool {
    let value = value.to_lowercase();
    value == "1" || value == "true" || value == "yes"
}

impl ConfigValue for PgConfigValue {
    fn parse(&self, value_str: &str) -> Result<PgConfigValue,String> {
        Ok(match self {
            PgConfigValue::Float(_) => PgConfigValue::Float(string_to_float(value_str)?),
            PgConfigValue::String(_) => PgConfigValue::String(value_str.to_string()),
            PgConfigValue::StaticStr(_) => PgConfigValue::String(value_str.to_string()),
            PgConfigValue::Boolean(_) => PgConfigValue::Boolean(truthy(value_str)),
            PgConfigValue::Size(_) => PgConfigValue::Size(string_to_usize(value_str)?)
        })
    }
}


fn map_error<R>(e: Result<R,ConfigError>) -> Result<R,Message> {
    e.map_err(|e| Message::DataError(DataMessage::ConfigError(e)))
}
pub struct PgPeregrineConfig(Config<'static,PgConfigKey,PgConfigValue>);

impl PgPeregrineConfig {
    pub fn new() -> PgPeregrineConfig {
        PgPeregrineConfig(Config::new(&CONFIG_CONFIG))
    }

    pub fn set(&mut self, key_str: &str, value: &str) -> Result<(),Message> {
        map_error(self.0.set(key_str,value))
    }

    fn get(&self, key: &PgConfigKey) -> Result<&PgConfigValue,Message> { map_error(self.0.get(key)) }
    pub fn get_f64(&self, key: &PgConfigKey) -> Result<f64,Message> { self.get(key)?.as_f64() }
    pub fn try_get_str(&self, key: &PgConfigKey) -> Option<&str> { self.0.try_get(key).and_then(|x| x.try_as_str()) }
    pub fn get_str(&self, key: &PgConfigKey) -> Result<&str,Message> { self.get(key)?.as_str() }
    pub fn try_get_bool(&self, key: &PgConfigKey) -> Option<bool> { self.0.try_get(key).and_then(|x| x.try_as_bool()) }
    pub fn get_bool(&self, key: &PgConfigKey) -> Result<bool,Message> { self.get(key)?.as_bool() }
    pub fn try_get_size(&self, key: &PgConfigKey) -> Option<usize> { self.0.try_get(key).and_then(|x| x.try_as_size()) }
    pub fn get_size(&self, key: &PgConfigKey) -> Result<usize,Message> { self.get(key)?.as_size() }
}