use varea::{ VareaItem, Discrete, RTreeRange };
use crate::core::focus::Focus;
use crate::core::track::Track;
use crate::core::Scale;

pub struct PanelSliceId {
    stick_tag: String,
    scale: Scale,
    index: u64,
    focus: Focus,
    track: Track
}

impl PanelSliceId {
    pub fn new(stick_tag: &str, scale: Scale, index: u64, focus: Focus, track: Track) -> PanelSliceId {
        PanelSliceId {
            stick_tag: stick_tag.to_string(), scale, index, focus, track
        }
    }

    pub fn to_range(&self) -> PanelSliceRange {
        let mut out = PanelSliceRange::new();
        out.set_stick_tags(&[self.stick_tag.clone()]);
        out.set_scale(self.scale.clone(),self.scale.next_scale());
        out.set_index(self.index,self.index+1);
        out.set_focus(self.focus.clone());
        out.set_tracks(&[self.track.clone()]);
        out
    }
}

#[derive(Debug)]
pub struct PanelSliceRange {
    stick_tags: Option<Vec<String>>,
    scale: Option<(Scale,Scale)>,
    index: Option<(u64,u64)>,
    focus: Option<Focus>,
    track: Option<Vec<Track>>
}

impl PanelSliceRange {
    pub fn new() -> PanelSliceRange {
        PanelSliceRange {
            stick_tags: None,
            scale: None,
            index: None,
            focus: None,
            track: None
        }
    }

    pub fn stick_tags(&self) -> Option<&[String]> { self.stick_tags.as_ref().and_then(|x| Some(x.as_ref())) }
    pub fn tracks(&self) -> Option<&[Track]> { self.track.as_ref().and_then(|x| Some(x.as_ref())) }

    pub fn set_stick_tags(&mut self, stick_tags: &[String]) { self.stick_tags = Some(stick_tags.to_vec()); }
    pub fn set_scale(&mut self, a: Scale, b: Scale) { self.scale = Some((a,b)); }
    pub fn set_index(&mut self, a: u64, b: u64) { self.index = Some((a,b)); }
    pub fn set_focus(&mut self, f: Focus) { self.focus = Some(f); }
    pub fn set_tracks(&mut self, t: &[Track]) {  self.track = Some(t.to_vec()); }

    pub fn to_varea_item(&self) -> VareaItem {
        let mut item = VareaItem::new();
        if let Some(stick_tags) = &self.stick_tags {
            item.add("stick",Discrete::new(stick_tags));
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
