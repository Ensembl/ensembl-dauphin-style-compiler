use crate::Scale;
use crate::allotment::allotmentmetadata::AllotmentMetadataReport;
use crate::{DataMessage, request::ChannelIntegration};
use crate::train::{ Carriage };
use crate::core::Viewport;
use crate::core::Assets;

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum CarriageSpeed {
    Quick, /* same stick, same switches */
    SlowCrossFade, /* same stick, different switches */
    Slow /* different stick */
}

pub trait PeregrineIntegration {
    fn set_assets(&mut self, assets: Assets);
    fn set_carriages(&mut self, carriages: &[Carriage], scale: Scale, index: u32) -> Result<(),DataMessage>;
    fn start_transition(&mut self, index: u32, max: u64, speed: CarriageSpeed) -> Result<(),DataMessage>;
    fn notify_viewport(&mut self, viewport: &Viewport, future: bool);
    fn notify_allotment_metadata(&mut self, metadata: &AllotmentMetadataReport);
    fn set_height(&mut self, height: i64);
    fn channel(&self) -> Box<dyn ChannelIntegration>;
}
