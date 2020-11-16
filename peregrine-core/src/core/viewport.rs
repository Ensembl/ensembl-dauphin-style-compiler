use crate::core::{ Focus, StickId, Track };
use super::layout::Layout;

#[derive(Clone,PartialEq)]
pub struct Viewport {
    layout: Layout,
    position: f64,
    scale: f64
}

impl Viewport {
    pub fn new(layout: &Layout, position: f64, scale: f64) -> Viewport {
        Viewport {
            layout: layout.clone(),
            position, scale
        }
    }

    pub fn empty() -> Viewport {
        Viewport {
            layout: Layout::empty(),
            position: 0.,
            scale: 0.
        }
    }

    pub fn layout(&self) -> &Layout { &self.layout }
    pub fn position(&self) -> f64 { self.position }
    pub fn scale(&self) -> f64 { self.scale }

    pub fn track_on(&self, track: &Track, yn: bool) -> Viewport {
        let out = self.clone();
        out.layout.track_on(track,yn);
        out
    }

    pub fn new_layout(&self, layout: &Layout) -> Viewport {
        let mut out = self.clone();
        out.layout = layout.clone();
        out
    }

    pub fn set_position(&self, position: f64) -> Viewport {
        let mut out = self.clone();
        out.position = position;
        out
    }

    pub fn set_scale(&self, scale: f64) -> Viewport {
        let mut out = self.clone();
        out.scale = scale;
        out
    }

    pub fn set_stick(&self, stick: &StickId) -> Viewport {
        let mut out = self.clone();
        out.layout.set_stick(stick);
        out
    }

    pub fn set_focus(&self, focus: &Focus) -> Viewport {
        let mut out = self.clone();
        out.layout.set_focus(focus);
        out
    }
}
