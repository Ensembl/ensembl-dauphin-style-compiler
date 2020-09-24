use crate::api::PeregrineApiQueue;
use crate::api::queue::ApiMessage;
use crate::core::{ Focus, StickId, Track };
use crate::train::{ Carriage };

pub struct PeregrineApi {
    queue: PeregrineApiQueue
}

impl PeregrineApi {
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
    fn set_api(&self, api: PeregrineApi);
    fn report_error(&self, error: &str);
    fn set_carriages(&self, carriages: &[Carriage], quick: bool);
}
