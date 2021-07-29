use crate::StickId;
use crate::{DataMessage, request::ChannelIntegration};
use crate::train::{ Carriage };
use crate::core::Viewport;

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum CarriageSpeed {
    Quick, /* same stick, same switches */
    SlowCrossFade, /* same stick, different switches */
    Slow /* different stick */
}

pub trait PeregrineIntegration {
    fn set_carriages(&mut self, carriages: &[Carriage], index: u32) -> Result<(),DataMessage>;
    fn start_transition(&mut self, index: u32, max: u64, speed: CarriageSpeed) -> Result<(),DataMessage>;
    fn notify_viewport(&mut self, viewport: &Viewport, future: bool);
    fn channel(&self) -> Box<dyn ChannelIntegration>;
    fn busy(&mut self, yn: bool);
}
