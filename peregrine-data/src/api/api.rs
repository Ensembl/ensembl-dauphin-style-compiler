use crate::InstanceInformation;
use crate::globals::allotmentmetadata::GlobalAllotmentMetadata;
use crate::globals::playingfield::PlayingField;
use crate::train::drawing::drawingcarriage::DrawingCarriage;
use crate::DataMessage;
use crate::core::Viewport;
use crate::core::Assets;
use peregrine_toolkit::identitynumber;

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum CarriageSpeed {
    Quick, /* same stick, same switches */
    SlowCrossFade, /* same stick, different switches */
    Slow /* different stick */
}

#[cfg_attr(any(debug_assertions,debug_trains),derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash)]
pub struct TrainIdentity(u64);

identitynumber!(TRAINID);
pub(crate) fn new_train_identity() -> TrainIdentity {
    TrainIdentity(TRAINID.next())
}

pub trait PeregrineIntegration {
    fn report_instance_information(&self, info: &InstanceInformation);
    fn set_assets(&mut self, assets: &Assets);

    fn create_train(&mut self, train: &TrainIdentity);
    fn drop_train(&mut self, train: &TrainIdentity);

    fn create_carriage(&mut self, carriage: &DrawingCarriage);
    fn drop_carriage(&mut self, carriage: &DrawingCarriage);

    fn set_carriages(&mut self, train: &TrainIdentity, carriages: &[DrawingCarriage]) -> Result<(),DataMessage>;
    fn start_transition(&mut self, train: &TrainIdentity, max: u64, speed: CarriageSpeed) -> Result<(),DataMessage>;
    fn notify_viewport(&mut self, viewport: &Viewport);
    fn notify_allotment_metadata(&mut self, metadata: &GlobalAllotmentMetadata);
    fn set_playing_field(&mut self, playing_field: PlayingField);
}
