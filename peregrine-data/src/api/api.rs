use crate::api::PeregrineApiQueue;
use crate::api::queue::ApiMessage;
use crate::core::{ Focus, StickId, Track };
use crate::request::ChannelIntegration;
use crate::train::{ Carriage };
use super::PeregrineCore;
use crate::request::channel::Channel;

#[derive(Debug,Clone)]
pub enum CarriageSpeed {
    Quick, /* same stick */
    Slow /* different stick */
}

pub trait PeregrineIntegration {
    fn report_error(&mut self, error: &str);
    fn set_carriages(&mut self, carriages: &[Carriage], index: u32);
    fn start_transition(&mut self, index: u32, max: u64, speed: CarriageSpeed);
    fn channel(&self) -> Box<dyn ChannelIntegration>;
}
