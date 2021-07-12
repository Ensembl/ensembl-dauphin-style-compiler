use crate::{DataMessage, core::{ StickId }};
use super::layout::Layout;
use crate::switch::trackconfiglist::TrackConfigList;

fn unwrap<T>(x: Option<T>) -> Result<T,DataMessage> {
    x.ok_or_else(|| DataMessage::CodeInvariantFailed("unready viewport queried".to_string()))
}

#[derive(Clone,Debug,PartialEq)]
enum LayoutBuilder {
    Pending(Option<StickId>,Option<TrackConfigList>),
    Finished(Layout)
}

impl LayoutBuilder {
    fn empty() -> LayoutBuilder {
        LayoutBuilder::Pending(None,None)
    }

    fn filled(layout: Layout) -> LayoutBuilder {
        LayoutBuilder::Finished(layout)
    }

    fn try_upgrade(&mut self) {
        match self {
            LayoutBuilder::Pending(Some(stick),Some(track_config_list)) => {
                *self = LayoutBuilder::Finished(Layout::new(stick,track_config_list));
            },
            _ => {}
        }
    }

    pub fn set_stick(&mut self, stick_in: &StickId) {
        match self {
            LayoutBuilder::Pending(stick,_) => { *stick = Some(stick_in.clone()); },
            LayoutBuilder::Finished(layout) => { layout.set_stick(stick_in); }
        }
        self.try_upgrade();
    }

    pub fn set_track_config_list(&mut self, track_config_list_in: &TrackConfigList) {
        match self {
            LayoutBuilder::Pending(_,track_config_list) => { *track_config_list = Some(track_config_list_in.clone()); },
            LayoutBuilder::Finished(layout) => { layout.set_track_config_list(track_config_list_in); }
        }
        self.try_upgrade();
    }

    pub fn layout(&self) -> Option<&Layout> {
        match self {
            LayoutBuilder::Pending(_,_) => None,
            LayoutBuilder::Finished(layout) => Some(layout)
        }
    }
}

#[derive(Clone,Debug,PartialEq)]
pub struct Viewport {
    layout: LayoutBuilder,
    position: Option<f64>,
    bp_per_screen: Option<f64>
}

impl Viewport {
    pub fn new(layout: &Layout, position: f64, bp_per_screen: f64) -> Viewport {
        Viewport {
            layout: LayoutBuilder::Finished(layout.clone()),
            position: Some(position),
            bp_per_screen: Some(bp_per_screen)
        }
    }

    pub fn empty() -> Viewport {
        Viewport {
            layout: LayoutBuilder::empty(),
            position: None,
            bp_per_screen: None
        }
    }

    pub fn ready(&self) -> bool {
        self.layout.layout().is_some() && self.position.is_some() && self.bp_per_screen.is_some()
    }

    pub fn layout(&self) -> Result<&Layout,DataMessage> { unwrap(self.layout.layout()) }
    pub fn position(&self) -> Result<f64,DataMessage> { unwrap(self.position) }
    pub fn bp_per_screen(&self) -> Result<f64,DataMessage> { unwrap(self.bp_per_screen) }

    pub fn new_layout(&self, layout: &Layout) -> Viewport {
        let mut out = self.clone();
        out.layout = LayoutBuilder::filled(layout.clone());
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
        out.layout.set_stick(stick);
        out
    }

    pub fn set_track_config_list(&self, track_config_list: &TrackConfigList) -> Viewport {
        let mut out = self.clone();
        out.layout.set_track_config_list(track_config_list);
        out
    }
}
