use std::sync::{Arc, Mutex};
use crate::core::pixelsize::PixelSize;
use crate::core::{ StickId, Viewport };
use crate::request::core::minirequest::MiniRequest;
use crate::request::minirequests::metricreq::MetricReport;
use crate::run::{add_task};
use crate::shapeload::carriagebuilder::CarriageBuilder;
use crate::train::main::datatasks::{load_stick, load_carriage};
use crate::train::main::train::StickData;
use crate::train::model::trainextent::TrainExtent;
use crate::{Assets, PgCommanderTaskSpec, DrawingCarriage, BackendNamespace, SettingMode };
use commander::{CommanderStream, PromiseFuture};
use eachorevery::eoestruct::StructValue;
use peregrine_toolkit::error::{err_web_drop, Error};
use peregrine_toolkit::{log_extra, lock, timer_start, timer_end};
use peregrine_toolkit_async::sync::blocker::{Blocker, Lockout};
use super::pgcore::PeregrineCore;

/* Messages fall into broad categories:
 *
 *  Updating viewport or switches to be transfered to the railway on the next raf:
 *    SetPosition
 *    SetBpPerScreen
 *    SetMinPxPerCarriage
 *    Switch
 *    RadioSwitch
 *    UpdateSwitch
 *    RegenerateTrackConfig
 *    SetStick
 *
 * Feedback from input/graphics to be fed to the railway immediately:
 *    Sketchy
 *    CarriageLoaded
 *    TransitionComplete
 *    Invalidate
 *    PingTrains
 * 
 * Retrieve information immediately from cache/backend:
 *    LoadCarriage
 *    LoadStick
 *    Jump
 * 
 * Metric reports to be sent to backend on a when-possible basis:
 *    ReportMetric
 *    GeneralMetric
 *
 * During startup:
 *    AddBackend
 * 
 * During boot.shutdown:
 *    SetAssets
 *    Ready
 *    Shutdown
 *
 */

 pub(crate) enum ApiMessage {
    AddBackend(String),
    WaitForApplicationReady,
    TransitionComplete,
    SetPosition(Option<f64>,Option<f64>,bool),
    SetStick(StickId),
    SetMinPxPerCarriage(u32),
    Switch(Vec<String>,StructValue),
    UpdateSwitch(Vec<String>,SettingMode),
    RadioSwitch(Vec<String>,bool),
    RegenerateTrackConfig,
    Jump(String,PromiseFuture<Option<(StickId,f64,f64)>>),
    ReportMetric(BackendNamespace,MetricReport),
    GeneralMetric(String,Vec<(String,String)>,Vec<(String,f64)>),
    SetAssets(Assets),
    PingTrains,
    Sketchy(bool),
    CarriageLoaded(DrawingCarriage),
    Invalidate,
    LoadStick(TrainExtent,Arc<Mutex<StickData>>),
    LoadCarriage(CarriageBuilder),
    Shutdown
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

    async fn regenerate_tracks(&mut self, data: &mut PeregrineCore) -> Result<(),Error> {
        self.viewport = self.viewport.set_track_config_list(&data.switches.get_track_config_list().await?);
        Ok(())
    }

    async fn run_message(&mut self, data: &mut PeregrineCore, message: ApiMessage) {
        timer_start!(&format!("api message"));
        match message {
            ApiMessage::WaitForApplicationReady => {
                data.base.channel_registry.booted().await;
            },
            ApiMessage::AddBackend(access) => {
                err_web_drop(data.base.channel_registry.add_backend(&access).await);
            },
            ApiMessage::LoadStick(extent,output) => {
                load_stick(&mut data.base,&data.agent_store.stick_store,&extent,&output);
            },
            ApiMessage::LoadCarriage(builder) => {
                load_carriage(&mut data.base, &data.agent_store.lane_store,&builder);
            }
            ApiMessage::TransitionComplete => {
                data.train_set.transition_complete();
            },
            ApiMessage::SetPosition(centre,size,only_if_unknown) =>{
                if let Some(size) = size {
                    self.viewport = self.viewport.set_bp_per_screen(size,only_if_unknown);
                }
                if let Some(centre) = centre {
                    self.viewport = self.viewport.set_position(centre,only_if_unknown);
                }
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
                        self.viewport = self.viewport.set_stick(stick);
                    },
                    Err(e) => {
                        data.base.messages.send(e.clone());
                    }
                }
            },
            ApiMessage::CarriageLoaded(carriage) => {
                carriage.set_ready();
            },
            ApiMessage::Switch(path,value) => {
                data.switches.switch(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>(),value).await;
                err_web_drop(self.regenerate_tracks(data).await);
            },
            ApiMessage::UpdateSwitch(path,value) => {
                data.switches.update_switch(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>(),value).await;
                err_web_drop(self.regenerate_tracks(data).await);
            },
            ApiMessage::RadioSwitch(path,yn) => {
                data.switches.radio_switch(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>(),yn);
                err_web_drop(self.regenerate_tracks(data).await);
            },
            ApiMessage::RegenerateTrackConfig => {
                err_web_drop(self.regenerate_tracks(data).await);
            },
            ApiMessage::ReportMetric(channel,metric) => {
                data.base.manager.execute_and_forget(&channel,MiniRequest::Metric(metric));
            },
            ApiMessage::GeneralMetric(name,tags,values) => {
                data.base.metrics.add_general(&name,&tags,&values);
            },
            ApiMessage::SetAssets(assets) => {
                lock!(data.base.assets).add(&assets);
            },
            ApiMessage::PingTrains => {
                data.train_set.ping();
            },
            ApiMessage::Sketchy(yn) => {
                 data.train_set.set_sketchy(yn);
            },
            ApiMessage::Invalidate => {
                 data.train_set.invalidate();
            },
            ApiMessage::Shutdown => {
                log_extra!("data module shutdown!");
                data.shutdown().run();
            }
        }
        timer_end!(&format!("api message"));
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

    fn update_viewport(&mut self, data: &mut PeregrineCore, new_viewport: Viewport) {
        if new_viewport != data.viewport {
            data.viewport = new_viewport.clone();
            data.train_set.set(&new_viewport);
        }
    }

    pub(crate) fn visual_blocker(&self) -> &Blocker { &self.visual_blocker }

    pub(crate) fn run(&self, data: &mut PeregrineCore) {
        self.push(ApiMessage::WaitForApplicationReady);
        let mut self2 = self.clone();
        let mut data2 = data.clone();
        add_task::<()>(&data.base.commander,PgCommanderTaskSpec {
            name: format!("api message runner"),
            prio: 0,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                loop {
                    let mut messages = self2.queue.get_multi(None).await;
                    timer_start!("api message start");
                    let mut campaign = ApiQueueCampaign::new(&data2.viewport);
                    let mut lockouts = vec![];
                    timer_end!("api message start");
                    timer_start!("api message set");
                    for (message,lockout) in messages.drain(..) {
                        campaign.run_message(&mut data2,message).await;
                        lockouts.push(lockout);
                    }
                    timer_end!("api message set");
                    timer_start!("update_viewport");
                    self2.update_viewport(&mut data2,campaign.viewport().clone());
                    timer_end!("update_viewport");
                    timer_start!("train ping");
                    data2.train_set.ping();
                    timer_end!("train ping");
                    drop(lockouts);
                    if data2.base.shutdown.poll() { break; }
                }
                log_extra!("data api runner shutdown");
                Ok(())
            }),
            stats: false
        });
    }

    pub(crate) fn push(&self, message: ApiMessage) {
        self.queue.add((message,self.visual_blocker.lock()));
    }

    pub fn carriage_ready(&self, drawing_carriage: &DrawingCarriage) {
        self.push(ApiMessage::CarriageLoaded(drawing_carriage.clone()));
    }

    pub(crate) fn regenerate_track_config(&self) {
        self.push(ApiMessage::RegenerateTrackConfig);
    }

    pub(crate) fn set_assets(&self, assets: &Assets) {
        self.push(ApiMessage::SetAssets(assets.clone()));
    }

    pub(crate) fn load_stick(&self, extent: &TrainExtent, output: &Arc<Mutex<StickData>>) {
        self.push(ApiMessage::LoadStick(extent.clone(),output.clone()));
    }

    pub(crate) fn load_carriage(&self, builder: &CarriageBuilder) {
        self.push(ApiMessage::LoadCarriage(builder.clone()));
    }

    pub fn shutdown(&self) {
        self.push(ApiMessage::Shutdown);
    }

    pub(crate) fn ping(&self) {
        self.push(ApiMessage::PingTrains);
    }
}
