use crate::core::channel::Channel;
use crate::core::pixelsize::PixelSize;
use crate::core::{ StickId, Viewport };
use crate::request::core::request::{BackendRequest, RequestVariant};
use crate::request::messages::metricreq::MetricReport;
use crate::run::{add_task};
use crate::run::bootstrap::bootstrap;
use crate::{Assets, PgCommanderTaskSpec};
use commander::{CommanderStream, PromiseFuture};
use peregrine_toolkit_async::sync::blocker::{Blocker, Lockout};
use crate::util::message::DataMessage;
use super::pgcore::PeregrineCore;

//#[cfg_attr(debug_assertions,derive(Debug))]
pub enum ApiMessage {
    Ready,
    TransitionComplete,
    SetPosition(f64),
    SetBpPerScreen(f64),
    SetStick(StickId),
    SetMinPxPerCarriage(u32),
    Bootstrap(u64,Channel),
    SetSwitch(Vec<String>),
    ClearSwitch(Vec<String>),
    RadioSwitch(Vec<String>,bool),
    RegenerateTrackConfig,
    Jump(String,PromiseFuture<Option<(StickId,f64,f64)>>),
    ReportMetric(Channel,MetricReport),
    GeneralMetric(String,Vec<(String,String)>,Vec<(String,f64)>),
    SetAssets(Assets),
    PingTrains,
    Sketchy(bool),
    Invalidate
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

    async fn run_message(&mut self, data: &mut PeregrineCore, message: ApiMessage) {
        match message {
            ApiMessage::Ready => {
                data.dauphin_ready();
            },
            ApiMessage::TransitionComplete => {
                let train_set = data.train_set.clone();
                train_set.transition_complete();
            },
            ApiMessage::SetPosition(pos) =>{
                self.viewport = self.viewport.set_position(pos);
            },
            ApiMessage::SetBpPerScreen(scale) => {
                self.viewport = self.viewport.set_bp_per_screen(scale);
            },
            ApiMessage::SetMinPxPerCarriage(px) => {
                self.viewport = self.viewport.set_pixel_size(&PixelSize::new(px))
            }            
            ApiMessage::Jump(location,promise) => {
                data.agent_store.jump_store.jump(&location,promise);
            },
            ApiMessage::SetStick(stick_id) => {
                match data.agent_store.stick_store.get(&stick_id).await.as_ref().map(|x| x.as_ref()) {
                    Ok(stick) => {
                        self.viewport = self.viewport.set_stick(&stick_id,stick.size());
                    },
                    Err(e) => {
                        data.base.messages.send(e.clone());
                    }
                }
            },
            ApiMessage::Bootstrap(identity,channel) => {
                bootstrap(&data.base,&data.agent_store,channel,identity);
            },
            ApiMessage::SetSwitch(path) => {
                data.switches.set_switch(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>());
                self.viewport = self.viewport.set_track_config_list(&data.switches.get_track_config_list());
            },
            ApiMessage::ClearSwitch(path) => {
                data.switches.clear_switch(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>());
                self.viewport = self.viewport.set_track_config_list(&data.switches.get_track_config_list());
            },
            ApiMessage::RadioSwitch(path,yn) => {
                data.switches.radio_switch(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>(),yn);
                self.viewport = self.viewport.set_track_config_list(&data.switches.get_track_config_list());
            },
            ApiMessage::RegenerateTrackConfig => {
                self.viewport = self.viewport.set_track_config_list(&data.switches.get_track_config_list());
            },
            ApiMessage::ReportMetric(channel,metric) => {
                data.base.manager.execute_and_forget(&channel,BackendRequest::new(RequestVariant::Metric(metric)));
            },
            ApiMessage::GeneralMetric(name,tags,values) => {
                data.base.metrics.add_general(&name,&tags,&values);
            },
            ApiMessage::SetAssets(assets) => {
                *data.base.assets.lock().unwrap() = assets;
            },
            ApiMessage::PingTrains => {
                data.train_set.ping();
            },
            ApiMessage::Sketchy(yn) => {
                 data.train_set.set_sketchy(yn);
            },
            ApiMessage::Invalidate => {
                 data.train_set.invalidate();
            }
        }
    }

    fn viewport(&self) -> &Viewport { &self.viewport }
}

#[derive(Clone)]
pub struct PeregrineApiQueue {
    queue: CommanderStream<(ApiMessage,Lockout)>,
    visual_blocker: Blocker
}

impl PeregrineApiQueue {
    pub fn new(visual_blocker: &Blocker) -> PeregrineApiQueue {
        PeregrineApiQueue {
            queue: CommanderStream::new(),
            visual_blocker: visual_blocker.clone()
        }
    }

    fn update_train_set(&mut self, objects: &mut PeregrineCore) {
        let viewport = objects.viewport.clone();
        let train_set = objects.train_set.clone();
        train_set.set(&viewport);
    }

    fn update_viewport(&mut self, data: &mut PeregrineCore, new_viewport: Viewport) {
        if new_viewport != data.viewport {
            data.viewport = new_viewport;
            self.update_train_set(data);
        }
    }

    pub(crate) fn run(&self, data: &mut PeregrineCore) {
        let mut self2 = self.clone();
        let mut data2 = data.clone();
        let carriage_loader = data.carriage_loader.clone();
        add_task::<Result<(),DataMessage>>(&data.base.commander,PgCommanderTaskSpec {
            name: format!("api message runner"),
            prio: 0,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                loop {
                    let mut messages = self2.queue.get_multi(None).await;
                    let mut campaign = ApiQueueCampaign::new(&data2.viewport);
                    let mut lockouts = vec![];
                    for (message,lockout) in messages.drain(..) {
                        campaign.run_message(&mut data2,message).await;
                        lockouts.push(lockout);
                    }
                    carriage_loader.load();
                    self2.update_viewport(&mut data2,campaign.viewport().clone());
                    drop(lockouts);
                }
            }),
            stats: false
        });
    }

    pub fn push(&self, message: ApiMessage) {
        self.queue.add((message,self.visual_blocker.lock()));
    }
}
