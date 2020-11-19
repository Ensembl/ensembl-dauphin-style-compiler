use crate::api::PeregrineObjects;
use crate::core::{ Focus, StickId, Track, Viewport };
use crate::PgCommanderTaskSpec;
use commander::CommanderStream;
use crate::request::channel::Channel;
use crate::request::bootstrap::bootstrap;

pub enum ApiMessage {
    Ready,
    TransitionComplete,
    AddTrack(Track),
    RemoveTrack(Track),
    SetPosition(f64),
    SetScale(f64),
    SetFocus(Focus),
    SetStick(StickId),
    Bootstrap(Channel)
}

#[derive(Clone)]
pub struct PeregrineApiQueue {
    queue: CommanderStream<ApiMessage>
}

impl PeregrineApiQueue {
    pub fn new() -> PeregrineApiQueue {
        PeregrineApiQueue {
            queue: CommanderStream::new(),
        }
    }

    fn update_train_set(&mut self, objects: &mut PeregrineObjects) {
        let viewport = objects.viewport.clone();
        let train_set = objects.train_set.clone();
        train_set.set(objects,&viewport);
    }

    fn update_viewport(&mut self, data: &mut PeregrineObjects, new_viewport: Viewport) {
        if new_viewport != data.viewport {
            data.viewport = new_viewport;
            self.update_train_set(data);
        }
    }

    fn try_bootstrap(&mut self, data: &mut PeregrineObjects, channel: Channel) -> anyhow::Result<()> {
        bootstrap(&data.manager,&data.program_loader,&data.commander,&data.dauphin,channel,&data.booted)
    }

    fn bootstrap(&mut self, data: &mut PeregrineObjects, channel: Channel) {
        match self.try_bootstrap(data,channel) {
            Ok(()) => {}
            Err(e) => {
                data.integration.lock().unwrap().report_error(&format!("Cannot bootstrap, nothing at far end"));
            }
        }
    }

    fn run_message(&mut self, data: &mut PeregrineObjects, message: ApiMessage) {
        match message {
            ApiMessage::Ready => {
                data.dauphin_ready();
            },
            ApiMessage::TransitionComplete => {
                let train_set = data.train_set.clone();
                train_set.transition_complete(data);
            },
            ApiMessage::AddTrack(track) => {
                self.update_viewport(data,data.viewport.track_on(&track,true));
            },
            ApiMessage::RemoveTrack(track) => {
                self.update_viewport(data,data.viewport.track_on(&track,false));
            },
            ApiMessage::SetPosition(pos) =>{
                self.update_viewport(data,data.viewport.set_position(pos));
            },
            ApiMessage::SetScale(scale) => {
                self.update_viewport(data,data.viewport.set_scale(scale));
            },
            ApiMessage::SetFocus(focus) => {
                self.update_viewport(data,data.viewport.set_focus(&focus));
            },
            ApiMessage::SetStick(stick) => {
                self.update_viewport(data,data.viewport.set_stick(&stick));
            },
            ApiMessage::Bootstrap(channel) => {
                self.bootstrap(data,channel);
            }
        }
    }

    pub fn run(&self, data: &mut PeregrineObjects) {
        let mut self2 = self.clone();
        let mut data2 = data.clone();
        data.commander.add_task(PgCommanderTaskSpec {
            name: format!("api message runner"),
            prio: 5,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                loop {
                    let message = self2.queue.get().await;
                    self2.run_message(&mut data2,message);
                }
            })
        });
    }

    pub fn push(&self, message: ApiMessage) {
        self.queue.add(message);
    }
}
