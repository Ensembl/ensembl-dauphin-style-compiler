use crate::allotment::core::allotmentmetadata::AllotmentMetadataReport;
use crate::core::channel::ChannelIntegration;
use crate::train::drawingcarriage::DrawingCarriage;
use crate::{DataMessage, PlayingField, TrainExtent};
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

    fn create_train(&mut self, train: &TrainExtent);
    fn drop_train(&mut self, train: &TrainExtent);

    fn create_carriage(&mut self, carriage: &DrawingCarriage);
    fn drop_carriage(&mut self, carriage: &DrawingCarriage);

    fn set_carriages(&mut self, train: &TrainExtent, carriages: &[DrawingCarriage]) -> Result<(),DataMessage>;
    fn start_transition(&mut self, train: &TrainExtent, max: u64, speed: CarriageSpeed) -> Result<(),DataMessage>;
    fn notify_viewport(&mut self, viewport: &Viewport, future: bool);
    fn notify_allotment_metadata(&mut self, metadata: &AllotmentMetadataReport);
    fn set_playing_field(&mut self, playing_field: PlayingField);
    fn channel(&self) -> Box<dyn ChannelIntegration>;
}
