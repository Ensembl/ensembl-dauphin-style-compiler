use crate::{DataMessage, core::{ Focus, StickId, Track }};
use super::layout::Layout;

fn unwrap<T>(x: Option<T>) -> Result<T,DataMessage> {
    x.ok_or_else(|| DataMessage::CodeInvariantFailed("unready viewport queried".to_string()))
}

#[derive(Clone,PartialEq)]
pub struct Viewport {
    layout: Layout,
    position: Option<f64>,
    bp_per_screen: Option<f64>
}

impl Viewport {
    pub fn new(layout: &Layout, position: f64, bp_per_screen: f64) -> Viewport {
        Viewport {
            layout: layout.clone(),
            position: Some(position),
            bp_per_screen: Some(bp_per_screen)
        }
    }

    pub fn empty() -> Viewport {
        Viewport {
            layout: Layout::empty(),
            position: None,
            bp_per_screen: None
        }
    }

    pub fn ready(&self) -> bool { self.layout.ready() && self.position.is_some() && self.bp_per_screen.is_some() }

    pub fn layout(&self) -> &Layout { &self.layout }
    pub fn position(&self) -> Result<f64,DataMessage> { unwrap(self.position) }
    pub fn bp_per_screen(&self) -> Result<f64,DataMessage> { unwrap(self.bp_per_screen) }

    pub fn track_on(&self, track: &Track, yn: bool) -> Viewport {
        let mut out = self.clone();
        out.layout = out.layout.track_on(track,yn);
        out
    }

    pub fn new_layout(&self, layout: &Layout) -> Viewport {
        let mut out = self.clone();
        out.layout = layout.clone();
        out
    }

    pub fn set_position(&self, position: f64) -> Viewport {
        let mut out = self.clone();
        out.position = Some(position);
        out
    }

    pub fn set_bp_per_screen(&self, scale: f64) -> Viewport {
        let mut out = self.clone();
        out.bp_per_screen = Some(scale);
        out
    }

    pub fn set_stick(&self, stick: &StickId) -> Viewport {
        let mut out = self.clone();
        out.layout = out.layout.set_stick(stick);
        out
    }

    pub fn set_focus(&self, focus: &Focus) -> Viewport {
        let mut out = self.clone();
        out.layout = out.layout.set_focus(focus);
        out
    }
}
