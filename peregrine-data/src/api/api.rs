use crate::allotment::core::playingfield::PlayingField;
use crate::core::channel::ChannelIntegration;
use crate::train::drawingcarriage::DrawingCarriage2;
use crate::{DataMessage, TrainExtent, GlobalAllotmentMetadata, GlobalPlayingField};
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

    fn create_carriage(&mut self, carriage: &DrawingCarriage2);
    fn drop_carriage(&mut self, carriage: &DrawingCarriage2);

    fn set_carriages(&mut self, train: &TrainExtent, carriages: &[DrawingCarriage2]) -> Result<(),DataMessage>;
    fn start_transition(&mut self, train: &TrainExtent, max: u64, speed: CarriageSpeed) -> Result<(),DataMessage>;
    fn notify_viewport(&mut self, viewport: &Viewport);
    fn notify_allotment_metadata(&mut self, metadata: &GlobalAllotmentMetadata);
    fn set_playing_field(&mut self, playing_field: PlayingField);
    fn channel(&self) -> Box<dyn ChannelIntegration>;
}
