use crate::allotment::core::allotmentmetadata::AllotmentMetadataReport;
use crate::core::channel::ChannelIntegration;
use crate::{DataMessage, PlayingField};
use crate::train::{ Carriage, Train };
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

    fn create_train(&mut self, train: &Train);
    fn drop_train(&mut self, train: &Train);

    fn create_carriage(&mut self, carriage: &Carriage);
    fn drop_carriage(&mut self, carriage: &Carriage);

    fn set_carriages(&mut self, train: &Train, carriages: &[Carriage]) -> Result<(),DataMessage>;
    fn start_transition(&mut self, train: &Train, max: u64, speed: CarriageSpeed) -> Result<(),DataMessage>;
    fn notify_viewport(&mut self, viewport: &Viewport, future: bool);
    fn notify_allotment_metadata(&mut self, metadata: &AllotmentMetadataReport);
    fn set_playing_field(&mut self, playing_field: PlayingField);
    fn channel(&self) -> Box<dyn ChannelIntegration>;
}
