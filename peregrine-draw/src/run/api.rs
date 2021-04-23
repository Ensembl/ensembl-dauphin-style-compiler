use crate::util::message::{ Message };
pub use url::Url;
pub use web_sys::{ console, WebGlRenderingContext, Element };
use peregrine_data::{Channel, PeregrineConfig, PgCommanderTaskSpec, StickId, Track};
use super::progress::Progress;
use commander::CommanderStream;
use peregrine_message::Instigator;
use super::inner::PeregrineInnerAPI;
use crate::stage::stage::Position;
use super::dom::PeregrineDom;
use crate::integration::pgcommander::PgCommanderWeb;

use std::sync::{ Arc, Mutex };

enum DrawMessage {
    SetX(f64),
    SetY(f64),
    SetBpPerScreen(f64),
    AddTrack(Track),
    RemoveTrack(Track),
    SetStick(StickId),
    Bootstrap(Channel),
    SetupBlackbox(String),
    SetMessageReporter(Box<dyn FnMut(Message) + 'static + Send>)
}

impl DrawMessage {
    fn run(self, draw: &mut PeregrineInnerAPI, mut instigator: Instigator<Message>) {
        match self {
            DrawMessage::SetBpPerScreen(bp) => {
                draw.set_bp_per_screen(bp,&mut instigator);
            },
            DrawMessage::SetX(x) => {
                draw.set_x(x,&mut instigator);
            },
            DrawMessage::SetY(y) => {
                draw.set_y(y);
                instigator.done();
            },
            DrawMessage::AddTrack(track) => {
                draw.add_track(track,&mut instigator);
            },
            DrawMessage::RemoveTrack(track) => {
                draw.remove_track(track,&mut instigator);
            }
            DrawMessage::SetStick(stick) => {
                draw.set_stick(&stick,&mut instigator);
            },
            DrawMessage::Bootstrap(channel) => {
                draw.bootstrap(channel.clone(),&mut instigator);
            },
            DrawMessage::SetupBlackbox(url) => {
                let e = draw.setup_blackbox(&url);
                if let Err(e) = e { instigator.error(e); }
                instigator.done();
            },
            DrawMessage::SetMessageReporter(cb) => {
                draw.set_message_reporter(cb);
                instigator.done();
            }
        }
    }
}

#[derive(Clone)]
pub struct PeregeineAPI {
    queue: CommanderStream<(DrawMessage,Instigator<Message>)>,
    position: Arc<Mutex<Option<Position>>>
}

impl PeregeineAPI {
    pub fn new() -> PeregeineAPI {
        PeregeineAPI {
            queue: CommanderStream::new(),
            position: Arc::new(Mutex::new(None))
        }
    }

    pub fn bootstrap(&self, channel: &Channel) -> Progress {
        let (progress,insitgator) = Progress::new();
        self.queue.add((DrawMessage::Bootstrap(channel.clone()),insitgator.clone()));
        progress
    }

    pub fn set_x(&self, x: f64) -> Progress {
        let (progress,insitgator) = Progress::new();
        self.queue.add((DrawMessage::SetX(x),insitgator.clone()));
        progress
    }

    pub fn set_y(&self, y: f64) -> Progress {
        let (progress,insitgator) = Progress::new();
        self.queue.add((DrawMessage::SetY(y),insitgator.clone()));
        progress
    }

    pub fn set_bp_per_screen(&self, bp: f64) -> Progress {
        let (progress,insitgator) = Progress::new();
        self.queue.add((DrawMessage::SetBpPerScreen(bp),insitgator.clone()));
        progress
    }

    pub fn add_track(&self, track: Track) -> Progress {
        let (progress,insitgator) = Progress::new();
        self.queue.add((DrawMessage::AddTrack(track),insitgator.clone()));
        progress
    }

    pub fn remove_track(&self, track: Track) -> Progress {
        let (progress,insitgator) = Progress::new();
        self.queue.add((DrawMessage::RemoveTrack(track),insitgator.clone()));
        progress
    }

    pub fn set_stick(&self, stick: &StickId) -> Progress {
        let (progress,insitgator) = Progress::new();
        self.queue.add((DrawMessage::SetStick(stick.clone()),insitgator.clone()));
        progress
    }

    pub fn setup_blackbox(&self, url: &str) -> Progress {
        let (progress,insitgator) = Progress::new();
        self.queue.add((DrawMessage::SetupBlackbox(url.to_string()),insitgator.clone()));
        progress
    }

    pub fn set_message_reporter(&self,callback: Box<dyn FnMut(Message) + 'static + Send>) -> Progress {
        let (progress,insitgator) = Progress::new();
        self.queue.add((DrawMessage::SetMessageReporter(callback),insitgator.clone()));
        progress
    }

    pub fn x(&self) -> Option<f64> { self.position.lock().unwrap().as_ref().map(|p| p.x) }
    pub fn y(&self) -> Option<f64> { self.position.lock().unwrap().as_ref().map(|p| p.y) }
    pub fn bp_per_screen(&self) -> Option<f64> { self.position.lock().unwrap().as_ref().map(|p| p.bp_per_screen) }

    async fn step(&self, mut draw: PeregrineInnerAPI) -> Result<(),()> {
        loop {
            let (message,instigator) = self.queue.get().await;
            message.run(&mut draw,instigator);
        }
    }

    pub fn run(&self, config: PeregrineConfig, dom: PeregrineDom) -> Result<PgCommanderWeb,Message> {
        let inner = PeregrineInnerAPI::new(config,dom)?;
        let commander = inner.commander();
        let self2 = self.clone();
        inner.xxx_set_callbacks(move |p| {
            *self2.position.lock().unwrap() = p;
        });
        let self2 = self.clone();
        commander.add("draw-api",15,None,None,Box::pin(async move { self2.step(inner).await }));
        Ok(commander)
    }
}
