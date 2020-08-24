use std::collections::HashSet;
use varea::{ VareaItem, Discrete, RTreeRange };
use crate::core::focus::Focus;
use crate::core::stick::StickId;
use crate::core::track::Track;
use super::scale::Scale;

pub struct PanelSliceId {
    stick: StickId,
    scale: Scale,
    index: u64,
    focus: Focus,
    track: Track
}

impl PanelSliceId {
    pub fn new(stick: StickId, scale: Scale, index: u64, focus: Focus, track: Track) -> PanelSliceId {
        PanelSliceId {
            stick, scale, index, focus, track
        }
    }

    pub fn to_range(&self) -> PanelSliceRange {
        let mut out = PanelSliceRange::new();
        out.set_stick(self.stick.clone());
        out.set_scale(self.scale.clone(),self.scale.next_scale());
        out.set_index(self.index,self.index+1);
        out.set_focus(self.focus.clone());
        out.set_tracks(vec![self.track.clone()].iter().cloned().collect::<HashSet<_>>());
        out
    }
}

pub struct PanelSliceRange {
    stick: Option<StickId>,
    scale: Option<(Scale,Scale)>,
    index: Option<(u64,u64)>,
    focus: Option<Focus>,
    track: Option<HashSet<Track>>
}

impl PanelSliceRange {
    pub fn new() -> PanelSliceRange {
        PanelSliceRange {
            stick: None,
            scale: None,
            index: None,
            focus: None,
            track: None
        }
    }

    pub fn set_stick(&mut self, stick: StickId) { self.stick = Some(stick); }
    pub fn set_scale(&mut self, a: Scale, b: Scale) { self.scale = Some((a,b)); }
    pub fn set_index(&mut self, a: u64, b: u64) { self.index = Some((a,b)); }
    pub fn set_focus(&mut self, f: Focus) { self.focus = Some(f); }
    pub fn set_tracks(&mut self, t: HashSet<Track>) {  self.track = Some(t); }

    pub fn to_varea_item(&self) -> VareaItem {
        let mut item = VareaItem::new();
        if let Some(stick) = &self.stick {
            item.add("stick",Discrete::new(&[stick.clone()]));
        }
        if let Some(scale) = &self.scale {
            item.add("scale",RTreeRange::new(scale.0.get_index(),scale.1.get_index()));
        }
        if let Some(index) = &self.index {
            item.add("index",RTreeRange::new(index.0,index.1));
        }
        if let Some(focus) = &self.focus {
            item.add("focus",Discrete::new(&[focus.clone()]));
        }
        if let Some(track) = &self.track {
            let tracks = track.iter().cloned().collect::<Vec<_>>();
            item.add("track",Discrete::new(&tracks));
        }
        item
    }
}
