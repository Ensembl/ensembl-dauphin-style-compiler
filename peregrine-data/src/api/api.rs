use crate::allotment::globals::allotmentmetadata::GlobalAllotmentMetadata;
use crate::allotment::globals::playingfield::PlayingField;
use crate::core::channel::ChannelIntegration;
use crate::train::drawingcarriage::DrawingCarriage;
use crate::DataMessage;
use crate::core::Viewport;
use crate::core::Assets;
use lazy_static::lazy_static;
use identitynumber::identitynumber;

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum CarriageSpeed {
    Quick, /* same stick, same switches */
    SlowCrossFade, /* same stick, different switches */
    Slow /* different stick */
}

#[derive(Clone,PartialEq,Eq,Hash)]
pub struct TrainIdentity(u64);

identitynumber!(TRAINID);
pub(crate) fn new_train_identity() -> TrainIdentity {
    TrainIdentity(TRAINID.next())
}

pub trait PeregrineIntegration {
    fn set_assets(&mut self, assets: Assets);

    fn create_train(&mut self, train: &TrainIdentity);
    fn drop_train(&mut self, train: &TrainIdentity);

    fn create_carriage(&mut self, carriage: &DrawingCarriage);
    fn drop_carriage(&mut self, carriage: &DrawingCarriage);

    fn set_carriages(&mut self, train: &TrainIdentity, carriages: &[DrawingCarriage]) -> Result<(),DataMessage>;
    fn start_transition(&mut self, train: &TrainIdentity, max: u64, speed: CarriageSpeed) -> Result<(),DataMessage>;
    fn notify_viewport(&mut self, viewport: &Viewport);
    fn notify_allotment_metadata(&mut self, metadata: &GlobalAllotmentMetadata);
    fn set_playing_field(&mut self, playing_field: PlayingField);
    fn channel(&self) -> Box<dyn ChannelIntegration>;
}
