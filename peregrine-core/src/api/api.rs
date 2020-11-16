use crate::api::PeregrineApiQueue;
use crate::api::queue::ApiMessage;
use crate::core::{ Focus, StickId, Track };
use crate::request::ChannelIntegration;
use crate::train::{ Carriage };
use crate::{ Commander };
use crate::lock;
use super::PeregrineObjects;
use peregrine_dauphin_queue::{ PgDauphinQueue };

#[derive(Clone)]
pub struct PeregrineApi {
    queue: PeregrineApiQueue
}

impl PeregrineApi {
    pub fn new<C>(commander: C, integration: Box<dyn PeregrineIntegration>) -> anyhow::Result<PeregrineApi> where C: Commander + 'static {
        let queue = PeregrineApiQueue::new();
        let api = PeregrineApi { queue };
        let dauphin_queue = PgDauphinQueue::new();
        let mut core = PeregrineObjects::new(integration,commander,dauphin_queue)?;
        api.queue.run(&mut core);
        lock!(core.integration).set_api(api.clone());
        Ok(api)
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

pub trait PeregrineIntegration {
    fn set_api(&mut self, api: PeregrineApi);
    fn report_error(&mut self, error: &str);
    fn set_carriages(&mut self, carriages: &[Carriage], quick: bool);
    fn channel(&self) -> Box<dyn ChannelIntegration>;
}
