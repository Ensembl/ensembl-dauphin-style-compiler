use std::num::{ParseFloatError, ParseIntError};
use peregrine_data::{CarriageSpeed, DataMessage, DirectColour};
use crate::{shape::core::spectremanager::SpectreConfigKey, util::message::Message};
use lazy_static::lazy_static;
use peregrine_config::{ Config, ConfigKeyInfo, ConfigValue, ConfigError };
use crate::input::InputEventKind;
use css_color_parser::Color as CssColor;
#[allow(unused)]
use std::fmt;

// XXX factor with similar in peregrine-data
// XXX chromosome ned-stops

#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum CursorCircumstance {
    Default,
    Drag,
    Hold,
    Pinch,
    WheelNegative,
    WheelPositive,
    Hotspot
}

impl CursorCircumstance {
    pub(crate) fn each() -> Vec<CursorCircumstance> {
        vec![
            CursorCircumstance::Default,
            CursorCircumstance::Drag,
            CursorCircumstance::Hold,
            CursorCircumstance::Pinch,
            CursorCircumstance::WheelNegative,
            CursorCircumstance::WheelPositive,
            CursorCircumstance::Hotspot
        ]
    }
}

#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum PgConfigKey {
    FadeOverlap(CarriageSpeed),
    AnimationFadeRate(CarriageSpeed),
    KeyBindings(InputEventKind),
    PullMaxSpeed, // screenfulls/frame
    AutomatedPullMaxSpeed, // screenfulls/frame
    PullAcceleration, // screenfulls/frame/frame
    ZoomMaxSpeed, // factors-of-2/second,
    AutomatedZoomMaxSpeed, // factors-of-2/second,
    ZoomAcceleration, // factors-of-2/second/second,
    ZoomPixelSpeed, // how many pixels is a doubling?
    MouseClickRadius, // px (click vs drag)
    MouseHoldDwell, // ms (click vs hold)
    DoubleClickTime, // ms, how long a gap to not be part of double click
    Cursor(CursorCircumstance), // string, default mouse cursor
    DragCursorDelay, // ms, switch to drag cursor (ie assume not click)
    WheelSensitivity,
    WheelTimeout, // ms, how long between wheel events to assume not a wheel
    AuxBufferSize, // 4-byte units
    PinchMinSep, // px min num pix to try to scale with super zoom-out
    // min factor where we try to calculate centre-of-zoom rather htan default, to avoid divide-by-zero.
    // smaller than 1/(px-of-a-giant-screen), but much bigger than precision of floats
    PinchMinScale,
    Spectre(SpectreConfigKey), // various visual properties of spectres
    ReportUpdateFrequency, // ms between position reports
    AnimationBoing, // boing factor during moves
    AnimationVelocityMin, // velocity at which animation should be considered complete
    AnimationForceMin, // acceleration at which animation should be considered complete
    AnimationBrakeMul, // reduction of friction when undriven
    UserDragLethargy, // lethargy when directly dragged by user
    InstructedDragLethargy, // lethargy for user-expected but not user-driven moves
    SelfDragLethargy, // lethargy for unexpected moves
    WindowLethargy, // lethargy for window moves
    MinBpPerScreen,
    EndstopSound, // bell or not?
    MinHoldDragSize, // min bp-per-screen ratio for valid hold-drag
    TargetReportTime, // time between unforced target (intention) reports
    GotoRho, // Wijk and Nuij's rho parameter for goto animations: higher means perfer more zoom
    GotoV, // Wijk and Nuij's V parameter for goto animations: overall animation speed
    GotoMaxS, // Maximum value of Wijk and Nuij's S parameter before bailing and using a fade
    Verbosity, // Message verbosity
}

#[cfg(not(debug_assertions))]
impl fmt::Debug for PgConfigKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"*elided*")
    }
}

#[derive(Clone)]
pub enum PgConfigValue {
    Float(f64),
    String(String),
    StaticStr(&'static str),
    Boolean(bool),
    Size(usize),
    DirectColour(DirectColour)
}

lazy_static! {
    static ref CONFIG_CONFIG : Vec<ConfigKeyInfo<'static,PgConfigKey,PgConfigValue>> = {
        vec![
            ConfigKeyInfo { key: PgConfigKey::Verbosity, name: "verbosity", default: &PgConfigValue::StaticStr("") },
            ConfigKeyInfo { key: PgConfigKey::AnimationFadeRate(CarriageSpeed::Quick), name: "animate.fade.fast", default: &PgConfigValue::Float(200.) },
            ConfigKeyInfo { key: PgConfigKey::AnimationFadeRate(CarriageSpeed::SlowCrossFade), name: "animate.fade.slow-cross", default: &PgConfigValue::Float(500.) },
            ConfigKeyInfo { key: PgConfigKey::AnimationFadeRate(CarriageSpeed::Slow), name: "animate.fade.slow", default: &PgConfigValue::Float(400.) },
            ConfigKeyInfo { key: PgConfigKey::FadeOverlap(CarriageSpeed::Quick), name: "animate.overlap.fast", default: &PgConfigValue::Float(-0.75) },
            ConfigKeyInfo { key: PgConfigKey::FadeOverlap(CarriageSpeed::SlowCrossFade), name: "animate.overlap.slow-cross", default: &PgConfigValue::Float(0.) },
            ConfigKeyInfo { key: PgConfigKey::FadeOverlap(CarriageSpeed::Slow), name: "animate.overlap.slow", default: &PgConfigValue::Float(3.) },
            ConfigKeyInfo { key: PgConfigKey::AnimationBoing, name: "animate.boing", default: &PgConfigValue::Float(1.05) },
            ConfigKeyInfo { key: PgConfigKey::AnimationVelocityMin, name: "animate.animation-velocity-min", default: &PgConfigValue::Float(0.0005) },
            ConfigKeyInfo { key: PgConfigKey::AnimationForceMin, name: "animate.animation-force-min", default: &PgConfigValue::Float(0.00001) },
            ConfigKeyInfo { key: PgConfigKey::AnimationBrakeMul, name: "animate.animation-brake-mul", default: &PgConfigValue::Float(0.2) },
            ConfigKeyInfo { key: PgConfigKey::ZoomPixelSpeed, name: "animate.zoom-pixel-speed", default: &PgConfigValue::Float(500.) },
            ConfigKeyInfo { key: PgConfigKey::UserDragLethargy, name: "animate.lethargy.user-dragn", default: &PgConfigValue::Float(500.) },
            ConfigKeyInfo { key: PgConfigKey::InstructedDragLethargy, name: "animate.lethargy.instruct-drag", default: &PgConfigValue::Float(5000.) },
            ConfigKeyInfo { key: PgConfigKey::SelfDragLethargy, name: "animate.lethargy.self-drag", default: &PgConfigValue::Float(25000.) },
            ConfigKeyInfo { key: PgConfigKey::WindowLethargy, name: "animate.lethargy.window", default: &PgConfigValue::Float(500.) },
            ConfigKeyInfo { key: PgConfigKey::MinBpPerScreen, name: "display.min-bp-per-screen", default: &PgConfigValue::Float(30.) },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::DebugAction), name: "keys.debug-action", default: &PgConfigValue::StaticStr("") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PixelsLeft), name: "keys.pixels-left", default: &PgConfigValue::StaticStr("Alt-a[200]") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PixelsRight), name: "keys.pixels-right", default: &PgConfigValue::StaticStr("MirrorRunningDrag Alt-d[200]") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PixelsIn), name: "keys.pixels-in", default: &PgConfigValue::StaticStr("Alt-w[200]") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PixelsOut), name: "keys.pixels-out", default: &PgConfigValue::StaticStr("Wheel Alt-s[200]") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PullLeft), name: "keys.pull-left", default: &PgConfigValue::StaticStr("") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PullRight), name: "keys.pull-right", default: &PgConfigValue::StaticStr("") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PullIn), name: "keys.pull-in", default: &PgConfigValue::StaticStr("") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::PullOut), name: "keys.pull-out", default: &PgConfigValue::StaticStr("") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::SetPosition), name: "keys.pixels-scale", default: &PgConfigValue::StaticStr("RunningPinch") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::AnimatePosition), name: "keys.animate-position", default: &PgConfigValue::StaticStr("Court") },
            ConfigKeyInfo { key: PgConfigKey::KeyBindings(InputEventKind::ZMenu), name: "keys.zmenu", default: &PgConfigValue::StaticStr("Click") },
            ConfigKeyInfo { key: PgConfigKey::DoubleClickTime, name: "mouse.doubleclick-time", default: &PgConfigValue::Float(500.) },
            ConfigKeyInfo { key: PgConfigKey::MouseClickRadius, name: "mouse.click-radius", default: &PgConfigValue::Float(4.) },
            ConfigKeyInfo { key: PgConfigKey::MouseHoldDwell, name: "mouse.hold-dwell", default: &PgConfigValue::Float(1000.) },
            ConfigKeyInfo { key: PgConfigKey::WheelTimeout, name: "mouse.wheel-timeout", default: &PgConfigValue::Float(500.) },
            ConfigKeyInfo { key: PgConfigKey::Cursor(CursorCircumstance::Default), name: "mouse.cursor.default", default: &PgConfigValue::StaticStr("default") },
            ConfigKeyInfo { key: PgConfigKey::Cursor(CursorCircumstance::WheelPositive), name: "mouse.cursor.wheel.positive", default: &PgConfigValue::StaticStr("zoom-in col-resize") },
            ConfigKeyInfo { key: PgConfigKey::Cursor(CursorCircumstance::WheelNegative), name: "mouse.cursor.wheel.negative", default: &PgConfigValue::StaticStr("zoom-in col-resize") },
            ConfigKeyInfo { key: PgConfigKey::Cursor(CursorCircumstance::Drag), name: "mouse.cursor.drag", default: &PgConfigValue::StaticStr("grabbing") },
            ConfigKeyInfo { key: PgConfigKey::Cursor(CursorCircumstance::Hold), name: "mouse.cursor.hold", default: &PgConfigValue::StaticStr("crosshair") },
            ConfigKeyInfo { key: PgConfigKey::Cursor(CursorCircumstance::Pinch), name: "mouse.cursor.pinch", default: &PgConfigValue::StaticStr("nsew-resize") },
            ConfigKeyInfo { key: PgConfigKey::Cursor(CursorCircumstance::Hotspot), name: "mouse.cursor.hotspot", default: &PgConfigValue::StaticStr("pointer") },
            ConfigKeyInfo { key: PgConfigKey::DragCursorDelay, name: "mouse.drag-cursor-delay", default: &PgConfigValue::Float(100.) },
            ConfigKeyInfo { key: PgConfigKey::PullMaxSpeed, name: "pull.max-speed", default: &PgConfigValue::Float(1./40.) },
            ConfigKeyInfo { key: PgConfigKey::AutomatedPullMaxSpeed, name: "pull.max-speed.automated", default: &PgConfigValue::Float(1.) },
            ConfigKeyInfo { key: PgConfigKey::PullAcceleration, name: "pull.acceleration", default: &PgConfigValue::Float(1./4000.) },
            ConfigKeyInfo { key: PgConfigKey::ZoomMaxSpeed, name: "zoom.max-speed", default: &PgConfigValue::Float(1./10.) },
            ConfigKeyInfo { key: PgConfigKey::AutomatedZoomMaxSpeed, name: "zoom.max-speed.auto", default: &PgConfigValue::Float(4.) },
            ConfigKeyInfo { key: PgConfigKey::ZoomAcceleration, name: "zoom.acceleration", default: &PgConfigValue::Float(1./300.) },
            ConfigKeyInfo { key: PgConfigKey::GotoRho, name: "zoom.goto.rho", default: &PgConfigValue::Float(1.41) },
            ConfigKeyInfo { key: PgConfigKey::GotoV, name: "zoom.goto.v", default: &PgConfigValue::Float(0.003) },
            ConfigKeyInfo { key: PgConfigKey::GotoMaxS, name: "zoom.goto.s.max", default: &PgConfigValue::Float(5.) },
            ConfigKeyInfo { key: PgConfigKey::WheelSensitivity, name: "wheel.sensitivity", default: &PgConfigValue::Float(2.) },
            ConfigKeyInfo { key: PgConfigKey::PinchMinSep, name: "touch.pinch-min-sep", default: &PgConfigValue::Float(16.) },
            ConfigKeyInfo { key: PgConfigKey::PinchMinScale, name: "touch.pinch-min-scale", default: &PgConfigValue::Float(1./1000000.) },
            ConfigKeyInfo { key: PgConfigKey::AuxBufferSize, name: "perf.aux-buffer-size", default: &PgConfigValue::Size(256*1024) },
            ConfigKeyInfo { key: PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsWidth), name: "spectre.ants.width", default: &PgConfigValue::Float(1.) },
            ConfigKeyInfo { key: PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsLength), name: "spectre.ants.length", default: &PgConfigValue::Float(8.) },
            ConfigKeyInfo { key: PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsProp), name: "spectre.ants.prop", default: &PgConfigValue::Float(0.5) },
            ConfigKeyInfo { key: PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsColour), name: "spectre.ants.colour", default: &PgConfigValue::DirectColour(DirectColour(255,0,0,255)) },
            ConfigKeyInfo { key: PgConfigKey::Spectre(SpectreConfigKey::StainColour), name: "spectre.stain.colour", default: &PgConfigValue::DirectColour(DirectColour(50,50,50,100)) },
            ConfigKeyInfo { key: PgConfigKey::ReportUpdateFrequency, name: "report.update-frequency", default: &PgConfigValue::Float(250.) },
            ConfigKeyInfo { key: PgConfigKey::EndstopSound, name: "report.sound.endstop", default: &PgConfigValue::StaticStr("bell") },
            ConfigKeyInfo { key: PgConfigKey::MinHoldDragSize, name: "animate.min-hold-drag-size", default: &PgConfigValue::Float(0.01) },
            ConfigKeyInfo { key: PgConfigKey::TargetReportTime, name: "report.target-update-time", default: &PgConfigValue::Float(5000.) },
        ]};
}

fn string_to_float(value_str: &str) -> Result<f64,String> {
    value_str.parse().map_err(|e: ParseFloatError| e.to_string())
}

fn string_to_usize(value_str: &str) -> Result<usize,String> {
    value_str.parse().map_err(|e: ParseIntError| e.to_string())
}

fn string_to_colour(value_str: &str) -> Result<DirectColour,String> {
    let x = value_str.parse::<CssColor>().map_err(|op| format!("converting colour: {}",op.to_string()))?;
    Ok(DirectColour(x.r,x.g,x.b,(x.a*255.0) as u8))
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

    fn try_as_colour(&self) -> Option<DirectColour> {
        match self {
            PgConfigValue::DirectColour(x) => Some(x.clone()),
            _ => None
        }
    }

    fn as_colour(&self) -> Result<DirectColour,Message> {
        if let Some(v) = self.try_as_colour() { return Ok(v); }
        Err(Message::DataError(DataMessage::CodeInvariantFailed(format!("cannot get value as colour"))))
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
            PgConfigValue::Size(_) => PgConfigValue::Size(string_to_usize(value_str)?),
            PgConfigValue::DirectColour(_) => PgConfigValue::DirectColour(string_to_colour(value_str)?)
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
    pub fn try_get_colour(&self, key: &PgConfigKey) -> Option<DirectColour> { self.0.try_get(key).and_then(|x| x.try_as_colour()) }
    pub fn get_colour(&self, key: &PgConfigKey) -> Result<DirectColour,Message> { self.get(key)?.as_colour() }

}
