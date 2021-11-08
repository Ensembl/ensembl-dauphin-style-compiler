use crate::Scale;
use crate::allotment::allotmentmetadata::AllotmentMetadataReport;
use crate::core::channel::ChannelIntegration;
use crate::{DataMessage};
use crate::train::{ Carriage };
use crate::core::Viewport;
use crate::core::Assets;

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum CarriageSpeed {
    Quick, /* same stick, same switches */
    SlowCrossFade, /* same stick, different switches */
    Slow /* different stick */
}

#[derive(PartialEq,Eq,Clone,Debug)]
pub struct PlayingField {
    height: i64,
    squeeze: (i64,i64),
}

impl PlayingField {
    pub fn empty() -> PlayingField {
        PlayingField {
            height: 0,
            squeeze: (0,0)
        }
    }

    pub fn new_height(height: i64) -> PlayingField {
        PlayingField {
            height,
            squeeze: (0,0)
        }
    }

    pub fn new_squeeze(left: i64, right: i64) -> PlayingField {
        PlayingField {
            height: 0,
            squeeze: (left,right)
        }
    }

    pub fn height(&self) -> i64 { self.height }
    pub fn squeeze(&self) -> (i64,i64) { self.squeeze }

    pub fn union(&mut self, other: &PlayingField) {
        self.height = self.height.max(other.height);
        self.squeeze.0 = self.squeeze.0.max(other.squeeze.0);
        self.squeeze.1 = self.squeeze.1.max(other.squeeze.1);
    }
}

pub trait PeregrineIntegration {
    fn set_assets(&mut self, assets: Assets);
    fn set_carriages(&mut self, carriages: &[Carriage], scale: Scale, index: u32) -> Result<(),DataMessage>;
    fn start_transition(&mut self, index: u32, max: u64, speed: CarriageSpeed) -> Result<(),DataMessage>;
    fn notify_viewport(&mut self, viewport: &Viewport, future: bool);
    fn notify_allotment_metadata(&mut self, metadata: &AllotmentMetadataReport);
    fn set_playing_field(&mut self, playing_field: PlayingField);
    fn channel(&self) -> Box<dyn ChannelIntegration>;
}
