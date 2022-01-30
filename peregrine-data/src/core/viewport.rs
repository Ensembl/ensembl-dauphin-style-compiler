use crate::{DataMessage, core::{ StickId }};
use super::{layout::Layout, pixelsize::PixelSize};
use crate::switch::trackconfiglist::TrackConfigList;

fn unwrap<T>(x: Option<T>) -> Result<T,DataMessage> {
    x.ok_or_else(|| DataMessage::CodeInvariantFailed("unready viewport queried".to_string()))
}

#[derive(Clone,PartialEq)]
#[cfg_attr(debug_assertions,derive(Debug))]
enum LayoutBuilder {
    Pending(Option<(StickId,u64)>,Option<TrackConfigList>),
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
                *self = LayoutBuilder::Finished(Layout::new(&stick.0,stick.1,track_config_list));
            },
            _ => {}
        }
    }

    pub fn set_stick(&mut self, stick_in: &StickId, size: u64) {
        match self {
            LayoutBuilder::Pending(stick,_) => { *stick = Some((stick_in.clone(),size)); },
            LayoutBuilder::Finished(layout) => { layout.set_stick(stick_in,size); }
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

fn limit_value(value: &mut f64, min: f64, max: f64) {
    if *value < min { *value = min; }
    if *value > max { *value = max; }
}

#[derive(Clone,PartialEq)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Viewport {
    layout: LayoutBuilder,
    position: Option<f64>,
    bp_per_screen: Option<f64>,
    pixel_size: Option<PixelSize>
}

impl Viewport {
    pub fn empty() -> Viewport {
        Viewport {
            layout: LayoutBuilder::empty(),
            position: None,
            bp_per_screen: None,
            pixel_size: None
        }
    }

    pub fn ready(&self) -> bool {
        self.layout.layout().is_some() && self.position.is_some() && self.bp_per_screen.is_some() && self.pixel_size.is_some()
    }

    pub fn layout(&self) -> Result<&Layout,DataMessage> { unwrap(self.layout.layout()) }
    pub fn position(&self) -> Result<f64,DataMessage> { unwrap(self.position) }
    pub fn bp_per_screen(&self) -> Result<f64,DataMessage> { unwrap(self.bp_per_screen) }
    pub fn pixel_size(&self) -> Result<&PixelSize,DataMessage> { unwrap(self.pixel_size.as_ref()) }

    fn update_by_limits(&mut self) {
        if let (Ok(size),Some(position),Some(bp_per_screen)) = (
                                                                    self.layout().map(|x| x.size()),
                                                                    self.position.as_mut(),
                                                                    self.bp_per_screen.as_mut()) {
            limit_value(bp_per_screen,1.,size as f64);
            limit_value(position,*bp_per_screen/2.,(size as f64)-*bp_per_screen/2.);
        }
    }

    pub fn new_layout(&self, layout: &Layout) -> Viewport {
        let mut out = self.clone();
        out.layout = LayoutBuilder::filled(layout.clone());
        out.update_by_limits();
        out
    }

    pub fn set_position(&self, position: f64) -> Viewport {
        let mut out = self.clone();
        out.position = Some(position);
        out.update_by_limits();
        out
    }

    pub fn set_bp_per_screen(&self, scale: f64) -> Viewport {
        let mut out = self.clone();
        out.bp_per_screen = Some(scale);
        out.update_by_limits();
        out
    }

    pub fn set_pixel_size(&self, pixel_size: &PixelSize) -> Viewport {
        use web_sys::console;
        //console::log_1(&format!("V pixel_size {:?}",pixel_size).into());
        let mut out = self.clone();
        out.pixel_size = Some(pixel_size.clone());
        out
    }

    pub fn set_stick(&self, stick: &StickId, size: u64) -> Viewport {
        let mut out = self.clone();
        out.layout.set_stick(stick,size);
        out.position = None;
        out.bp_per_screen = None;
        out.update_by_limits();
        out
    }

    pub fn set_track_config_list(&self, track_config_list: &TrackConfigList) -> Viewport {
        let mut out = self.clone();
        out.layout.set_track_config_list(track_config_list);
        out.update_by_limits();
        out
    }
}
