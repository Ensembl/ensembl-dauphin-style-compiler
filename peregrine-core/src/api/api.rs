use crate::api::PeregrineApiQueue;
use crate::api::queue::ApiMessage;
use crate::core::{ Focus, StickId, Track };
use crate::request::ChannelIntegration;
use crate::train::{ Carriage };
use crate::lock;
use super::PeregrineObjects;
use crate::request::channel::Channel;

#[derive(Clone)]
pub struct PeregrineApi {
    queue: PeregrineApiQueue
}

impl PeregrineApi {
    pub fn new() -> anyhow::Result<PeregrineApi> {
        let queue = PeregrineApiQueue::new();
        let api = PeregrineApi { queue };
        Ok(api)
    }

    pub fn ready(&self, mut objects: PeregrineObjects) {
        lock!(objects.integration).set_api(self.clone());
        self.queue.run(&mut objects);
        self.queue.push(ApiMessage::Ready);
    }

    pub fn bootstrap(&self, channel: Channel) {
        self.queue.push(ApiMessage::Bootstrap(channel));
    }

    pub fn transition_complete(&self) {
        self.queue.push(ApiMessage::TransitionComplete);
    }

    pub fn add_track(&self, track: Track) {
        self.queue.push(ApiMessage::AddTrack(track));
    }

    pub fn remove_track(&self, track: Track) {
        self.queue.push(ApiMessage::RemoveTrack(track));
    }

    pub fn set_position(&self, pos: f64) {
        self.queue.push(ApiMessage::SetPosition(pos));
    }

    pub fn set_scale(&self, scale: f64) {
        self.queue.push(ApiMessage::SetScale(scale));
    }

    pub fn set_focus(&self, focus: &Focus) {
        self.queue.push(ApiMessage::SetFocus(focus.clone()));
    }

    pub fn set_stick(&self, stick: &StickId) {
        self.queue.push(ApiMessage::SetStick(stick.clone()));
    }
}

#[derive(Debug,Clone)]
pub enum CarriageSpeed {
    Quick, /* same stick */
    Slow /* different stick */
}

pub trait PeregrineIntegration {
    fn set_api(&mut self, api: PeregrineApi);
    fn report_error(&mut self, error: &str);
    fn set_carriages(&mut self, carriages: &[Carriage], index: u32);
    fn start_transition(&mut self, index: u32, max: u64, speed: CarriageSpeed);
    fn channel(&self) -> Box<dyn ChannelIntegration>;
}
