use crate::api::PeregrineCore;
use crate::core::{ Focus, StickId, Track, Viewport };
use crate::run::add_task;
use crate::PgCommanderTaskSpec;
use commander::CommanderStream;
use crate::request::channel::Channel;
use crate::request::bootstrap::bootstrap;
use crate::util::message::DataMessage;
use peregrine_message::Instigator;

#[derive(Debug)]
pub enum ApiMessage {
    Ready,
    TransitionComplete,
    AddTrack(Track),
    RemoveTrack(Track),
    SetPosition(f64),
    SetBpPerScreen(f64),
    SetFocus(Focus),
    SetStick(StickId),
    Bootstrap(Channel)
}

#[derive(Clone)]
pub struct PeregrineApiQueue {
    queue: CommanderStream<(ApiMessage,Instigator<DataMessage>)>
}

impl PeregrineApiQueue {
    pub fn new() -> PeregrineApiQueue {
        PeregrineApiQueue {
            queue: CommanderStream::new(),
        }
    }

    fn update_train_set(&mut self, objects: &mut PeregrineCore,instigator: Instigator<DataMessage>) {
        let viewport = objects.viewport.clone();
        let train_set = objects.train_set.clone();
        train_set.set(objects,&viewport,instigator);
    }

    fn update_viewport(&mut self, data: &mut PeregrineCore, new_viewport: Viewport, instigator: Instigator<DataMessage>) {
        if new_viewport != data.viewport {
            data.viewport = new_viewport;
            self.update_train_set(data,instigator);
        }
    }

    // TODO investigate bootstrap call chain
    fn bootstrap(&mut self, data: &mut PeregrineCore, channel: Channel) {
        bootstrap(&data.base,&data.agent_store,channel)
    }

    fn run_message(&mut self, data: &mut PeregrineCore, message: ApiMessage, instigator: Instigator<DataMessage>) {
        match message {
            ApiMessage::Ready => {
                data.dauphin_ready();
            },
            ApiMessage::TransitionComplete => {
                let train_set = data.train_set.clone();
                train_set.transition_complete(data);
            },
            ApiMessage::AddTrack(track) => {
                self.update_viewport(data,data.viewport.track_on(&track,true),instigator);
            },
            ApiMessage::RemoveTrack(track) => {
                self.update_viewport(data,data.viewport.track_on(&track,false),instigator);
            },
            ApiMessage::SetPosition(pos) =>{
                self.update_viewport(data,data.viewport.set_position(pos),instigator);
            },
            ApiMessage::SetBpPerScreen(scale) => {
                self.update_viewport(data,data.viewport.set_bp_per_screen(scale),instigator);
            },
            ApiMessage::SetFocus(focus) => {
                self.update_viewport(data,data.viewport.set_focus(&focus),instigator);
            },
            ApiMessage::SetStick(stick) => {
                self.update_viewport(data,data.viewport.set_stick(&stick),instigator);
            },
            ApiMessage::Bootstrap(channel) => {
                self.bootstrap(data,channel);
            }
        }
    }

    pub fn run(&self, data: &mut PeregrineCore) {
        let mut self2 = self.clone();
        let mut data2 = data.clone();
        add_task::<Result<(),DataMessage>>(&data.base.commander,PgCommanderTaskSpec {
            name: format!("api message runner"),
            prio: 5,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                loop {
                    let (message,instigator) = self2.queue.get().await;
                    self2.run_message(&mut data2,message,instigator);
                }
            })
        });
    }

    pub fn push(&self, message: ApiMessage, instigator: Instigator<DataMessage>) {
        self.queue.add((message,instigator));
    }
}
