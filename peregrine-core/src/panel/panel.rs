use anyhow::{ anyhow as err };
use varea::{ VareaItem, Discrete, RTreeRange };
use crate::core::focus::Focus;
use crate::core::track::Track;
use crate::core::{ Scale, StickId };
use crate::index::StickStore;
use super::panelprogramstore::PanelProgramStore;
use super::panelrunstore::PanelRun;
use crate::request::Channel;

#[derive(Debug)]
pub struct PanelProgramRegion {
    stick_tags: Option<Vec<String>>,
    scale: Option<(Scale,Scale)>,
    focus: Option<Focus>,
    track: Option<Vec<Track>>
}

impl PanelProgramRegion {
    pub fn new() -> PanelProgramRegion {
        PanelProgramRegion {
            stick_tags: None,
            scale: None,
            focus: None,
            track: None
        }
    }

    pub fn stick_tags(&self) -> Option<&[String]> { self.stick_tags.as_ref().and_then(|x| Some(x.as_ref())) }
    pub fn tracks(&self) -> Option<&[Track]> { self.track.as_ref().and_then(|x| Some(x.as_ref())) }
    pub fn scale(&self) -> Option<(&Scale,&Scale)> { self.scale.as_ref().and_then(|x| Some((&x.0,&x.1))) }

    pub fn set_stick_tags(&mut self, stick_tags: &[String]) { self.stick_tags = Some(stick_tags.to_vec()); }
    pub fn set_scale(&mut self, a: Scale, b: Scale) { self.scale = Some((a,b)); }
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

#[derive(Clone,Debug,Eq,Hash,PartialEq)]
pub struct Panel {
    stick: StickId,
    scale: Scale,
    focus: Focus,
    track: Track,
    index: u64
}

impl Panel {
    pub fn new(stick: StickId, index: u64, scale: Scale, focus: Focus, track: Track) -> Panel {
        Panel { stick, scale, focus, track, index }
    }

    fn map_scale(&self, scale: &Scale) -> u64 {
        self.index >> (scale.get_index() - self.scale.get_index())
    }

    fn to_candidate(&self, ppr: &PanelProgramRegion) -> anyhow::Result<Panel> {
        let max_scale = Scale::new(100);
        let scale = ppr.scale().map(|x| x.1.prev_scale()).unwrap_or(max_scale);
        let index = self.map_scale(&scale);
        Ok(Panel {
            stick: self.stick.clone(),
            scale, index,
            track: self.track.clone(),
            focus: self.focus.clone()
        })
    }

    pub async fn build_panel_run(&self, stick_store: &StickStore, panel_program_store: &PanelProgramStore) -> anyhow::Result<PanelRun> {
        let tags : Vec<String> = stick_store.get(&self.stick).await?
                        .as_ref().as_ref().ok_or_else(|| err!("No such stick"))?
                        .tags().iter().cloned().collect();
        let mut ppr = PanelProgramRegion::new();
        ppr.set_stick_tags(&tags);
        ppr.set_scale(self.scale.clone(),self.scale.next_scale());
        ppr.set_focus(self.focus.clone());
        ppr.set_tracks(&[self.track.clone()]);
        let (channel,prog) = panel_program_store.get(&ppr)
            .ok_or_else(|| err!("no program to render this track!"))?;
        let candidate = self.to_candidate(&ppr)?;
        Ok(PanelRun::new(channel,&prog,&candidate))
    }
}
