use crate::api::PeregrineCore;
use crate::core::{ Focus, StickId, Viewport };
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
    SetPosition(f64),
    SetBpPerScreen(f64),
    SetFocus(Focus),
    SetStick(StickId),
    Bootstrap(Channel),
    SetSwitch(Vec<String>),
    ClearSwitch(Vec<String>),
    RegeneraateTrackConfig
}

struct ApiQueueCampaign {
    viewport: Viewport
}

impl ApiQueueCampaign {
    fn new(viewport: &Viewport) -> ApiQueueCampaign {
        ApiQueueCampaign {
            viewport: viewport.clone()
        }
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
            ApiMessage::SetPosition(pos) =>{
                self.viewport = self.viewport.set_position(pos);
            },
            ApiMessage::SetBpPerScreen(scale) => {
                self.viewport = self.viewport.set_bp_per_screen(scale);
            },
            ApiMessage::SetFocus(focus) => {
                // XXX currently unimplemented
            },
            ApiMessage::SetStick(stick) => {
                self.viewport = self.viewport.set_stick(&stick);
            },
            ApiMessage::Bootstrap(channel) => {
                bootstrap(&data.base,&data.agent_store,channel,instigator);
            },
            ApiMessage::SetSwitch(path) => {
                data.switches.set_switch(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>());
                self.viewport = self.viewport.set_track_config_list(&data.switches.get_track_config_list());
            },
            ApiMessage::ClearSwitch(path) => {
                data.switches.clear_switch(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>());
                self.viewport = self.viewport.set_track_config_list(&data.switches.get_track_config_list());
            },
            ApiMessage::RegeneraateTrackConfig => {
                self.viewport = self.viewport.set_track_config_list(&data.switches.get_track_config_list());
            }
        }
    }

    fn viewport(&self) -> &Viewport { &self.viewport }
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

    fn bootstrap(&mut self, data: &mut PeregrineCore, channel: Channel, instigator: Instigator<DataMessage>) {
        bootstrap(&data.base,&data.agent_store,channel,instigator)
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
            ApiMessage::SetPosition(pos) =>{
                self.update_viewport(data,data.viewport.set_position(pos),instigator);
            },
            ApiMessage::SetBpPerScreen(scale) => {
                self.update_viewport(data,data.viewport.set_bp_per_screen(scale),instigator);
            },
            ApiMessage::SetFocus(focus) => {
                // XXX currently unimplemented
            },
            ApiMessage::SetStick(stick) => {
                self.update_viewport(data,data.viewport.set_stick(&stick),instigator);
            },
            ApiMessage::Bootstrap(channel) => {
                self.bootstrap(data,channel,instigator);
            },
            ApiMessage::SetSwitch(path) => {
                data.switches.set_switch(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>());
                self.update_viewport(data,data.viewport.set_track_config_list(&data.switches.get_track_config_list()),instigator);
            },
            ApiMessage::ClearSwitch(path) => {
                data.switches.clear_switch(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>());
                self.update_viewport(data,data.viewport.set_track_config_list(&data.switches.get_track_config_list()),instigator);
            },
            ApiMessage::RegeneraateTrackConfig => {
                self.update_viewport(data,data.viewport.set_track_config_list(&data.switches.get_track_config_list()),instigator);
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
                    let mut messages = self2.queue.get_multi().await;
                    let mut campagin = ApiQueueCampaign::new(&data2.viewport);
                    for (message,instigator) in messages.drain(..) {
                        campagin.run_message(&mut data2,message,instigator);
                    }
                    self2.update_viewport(&mut data2,campagin.viewport().clone(),Instigator::new());
                }
            }),
            stats: false
        });
    }

    pub fn push(&self, message: ApiMessage, instigator: Instigator<DataMessage>) {
        self.queue.add((message,instigator));
    }
}
