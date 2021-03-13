use anyhow::{ anyhow as err };
use varea::{ VareaItem, Discrete, RTreeRange };
use crate::core::focus::Focus;
use crate::core::track::Track;
use crate::core::{ Scale, StickId };
use crate::index::StickStore;
use super::panelprogramstore::PanelProgramStore;
use super::programregion::ProgramRegion;
use super::panelrunstore::PanelRun;
use crate::util::message::{ DataMessage };
use serde_cbor::Value as CborValue;

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

    pub fn stick_id(&self) -> &StickId { &self.stick }
    pub fn track(&self) -> &Track { &self.track }
    pub fn focus(&self) -> &Focus { &self.focus }
    pub fn index(&self) -> u64 { self.index }
    pub fn scale(&self) -> &Scale { &self.scale }

    pub fn serialize(&self) -> Result<CborValue,DataMessage> {
        Ok(CborValue::Array(vec![
            self.stick.serialize()?,self.scale.serialize()?,self.focus().serialize()?,
            self.track.serialize()?,CborValue::Integer(self.index as i128)
        ]))
    }

    pub fn min_value(&self) -> u64 {
        self.scale.bp_in_carriage() * self.index
    }

    pub fn max_value(&self) -> u64 {
        self.scale.bp_in_carriage() * (self.index+1)
    }

    fn map_scale(&self, scale: &Scale) -> u64 {
        self.index >> (scale.get_index() - self.scale.get_index())
    }

    fn to_candidate(&self, ppr: &ProgramRegion) ->Panel {
        let scale = ppr.scale_up(&self.scale);
        let index = self.map_scale(&scale);
        Panel {
            stick: self.stick.clone(),
            scale, index,
            track: self.track.clone(),
            focus: self.focus.clone()
        }
    }

    pub async fn build_panel_run(&self, stick_store: &StickStore, panel_program_store: &PanelProgramStore) -> Result<PanelRun,DataMessage> {
        let tags : Vec<String> = stick_store.get(&self.stick).await
                        .as_ref().as_ref().ok_or_else(|| err!("No such stick")).map_err(|e| DataMessage::XXXTmp(e.to_string()))?
                        .tags().iter().cloned().collect();
        let mut ppr = ProgramRegion::new();
        ppr.set_stick_tags(&tags);
        ppr.set_scale(self.scale.clone(),self.scale.next_scale());
        ppr.set_focus(self.focus.clone());
        ppr.set_tracks(&[self.track.clone()]);
        let (channel,prog,ppr) = panel_program_store.get(&ppr)
            .ok_or_else(|| DataMessage::NoPanelProgram(self.clone()))?;
        let candidate = self.to_candidate(&ppr);
        Ok(PanelRun::new(channel,&prog,&candidate))
    }
}
