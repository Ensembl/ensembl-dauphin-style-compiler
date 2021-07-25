use crate::util::message::{ Message };
pub use url::Url;
pub use web_sys::{ console, WebGlRenderingContext, Element };
use peregrine_data::{ Channel, StickId, Commander };
use super::{config::DebugFlag, progress::Progress};
use commander::CommanderStream;
use peregrine_message::Instigator;
use super::inner::PeregrineInnerAPI;
use super::dom::PeregrineDom;
use crate::integration::pgcommander::PgCommanderWeb;
use crate::run::globalconfig::PeregrineConfig;
use crate::run::config::{ PgConfigKey };
use super::frame::run_animations;

use std::sync::{ Arc, Mutex };

enum DrawMessage {
    Goto(f64,f64),
    SetY(f64),
    SetStick(StickId),
    SetSwitch(Vec<String>),
    ClearSwitch(Vec<String>),
    Bootstrap(Channel),
    SetMessageReporter(Box<dyn FnMut(Message) + 'static + Send>),
    DebugAction(u8),
    SetArtificial(String,bool)
}

// XXX conditional
impl std::fmt::Debug for DrawMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DrawMessage::Goto(centre,scale) => write!(f,"Goto({:?},{:?})",centre,scale),
            DrawMessage::SetY(y) => write!(f,"SetY({:?})",y),
            DrawMessage::SetStick(stick)  => write!(f,"SetStick({:?})",stick),
            DrawMessage::SetSwitch(path) => write!(f,"SetSwitch({:?})",path),
            DrawMessage::ClearSwitch(path)  => write!(f,"ClearSwitch({:?})",path),
            DrawMessage::Bootstrap(channel)  => write!(f,"Channel({:?})",channel),
            DrawMessage::SetMessageReporter(_) => write!(f,"SetMessageReporter(...)"),
            DrawMessage::DebugAction(index)  => write!(f,"DebugAction({:?})",index),
            DrawMessage::SetArtificial(name,start) => write!(f,"SetArtificial({:?},{:?})",name,start)
        }
    }
}

impl DrawMessage {
    fn run(self, draw: &mut PeregrineInnerAPI, mut instigator: Instigator<Message>) -> Result<(),Message> {
        if draw.config().get_bool(&PgConfigKey::DebugFlag(DebugFlag::ShowIncomingMessages))? {
            console::log_1(&format!("message {:?}",self).into());
        }
        match self {
            DrawMessage::Goto(centre,scale) => {
                draw.goto(centre,scale,&mut instigator);
            },
            DrawMessage::SetArtificial(name,down) => {
                draw.set_artificial(&name,down);
            }
            DrawMessage::SetY(y) => {
                draw.set_y(y);
                instigator.done();
            },
            DrawMessage::SetStick(stick) => {
                draw.set_stick(&stick,&mut instigator);
            },
            DrawMessage::SetSwitch(path) => {
                draw.set_switch(&path.iter().map(|x| &x as &str).collect::<Vec<_>>(),&mut instigator);
            },
            DrawMessage::ClearSwitch(path) => {
                draw.clear_switch(&path.iter().map(|x| &x as &str).collect::<Vec<_>>(),&mut instigator);
            },
            DrawMessage::Bootstrap(channel) => {
                draw.bootstrap(channel.clone(),&mut instigator);
            },
            DrawMessage::SetMessageReporter(cb) => {
                draw.set_message_reporter(cb);
                instigator.done();
            },
            DrawMessage::DebugAction(index) => {
                draw.debug_action(index);
                instigator.done();
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct PeregrineAPI {
    queue: CommanderStream<(DrawMessage,Instigator<Message>)>,
    stick: Arc<Mutex<Option<String>>>
}

impl PeregrineAPI {
    pub fn new() -> PeregrineAPI {
        PeregrineAPI {
            queue: CommanderStream::new(),
            stick: Arc::new(Mutex::new(None))
        }
    }

    pub fn bootstrap(&self, channel: &Channel) -> Progress {
        let (progress,insitgator) = Progress::new();
        self.queue.add((DrawMessage::Bootstrap(channel.clone()),insitgator.clone()));
        progress
    }

    pub fn set_y(&self, y: f64) -> Progress {
        let (progress,insitgator) = Progress::new();
        self.queue.add((DrawMessage::SetY(y),insitgator.clone()));
        progress
    }

    pub fn goto(&self, left: f64, right: f64) -> Progress {
        let (progress,insitgator) = Progress::new();
        let (left,right) = (left.min(right),left.max(right));
        self.queue.add((DrawMessage::Goto((left+right)/2.,right-left),insitgator.clone()));
        progress
    }

    pub fn set_switch(&self, path: &[&str]) -> Progress {
        let (progress,insitgator) = Progress::new();
        self.queue.add((DrawMessage::SetSwitch(path.iter().map(|x| x.to_string()).collect()),insitgator.clone()));
        progress
    }

    pub fn clear_switch(&self, path: &[&str]) -> Progress {
        let (progress,insitgator) = Progress::new();
        self.queue.add((DrawMessage::ClearSwitch(path.iter().map(|x| x.to_string()).collect()),insitgator.clone()));
        progress
    }

    pub fn set_stick(&self, stick: &StickId) -> Progress {
        let (progress,insitgator) = Progress::new();
        *self.stick.lock().unwrap() = Some(stick.get_id().to_string()); // XXX not really true yet: have proper ro status via data
        self.queue.add((DrawMessage::SetStick(stick.clone()),insitgator.clone()));
        progress
    }

    pub fn set_message_reporter(&self, callback: Box<dyn FnMut(Message) + 'static + Send>) -> Progress {
        let (progress,insitgator) = Progress::new();
        self.queue.add((DrawMessage::SetMessageReporter(callback),insitgator.clone()));
        progress
    }

    pub fn debug_action(&self, index:u8) {
        let (_,instigator) = Progress::new();
        self.queue.add((DrawMessage::DebugAction(index),instigator.clone()));
    }

    pub fn stick(&self) -> Option<String> { self.stick.lock().unwrap().as_ref().cloned() }

    pub fn set_artificial(&self, name: &str, start: bool) -> Progress {
        let (progress,insitgator) = Progress::new();
        self.queue.add((DrawMessage::SetArtificial(name.to_string(),start),insitgator.clone()));
        progress        
    }

    async fn step(&self, mut draw: PeregrineInnerAPI) -> Result<(),()> {
        loop {
            let (message,instigator) = self.queue.get().await;
            message.run(&mut draw,instigator);
        }
    }

    pub fn run(&self, config: PeregrineConfig, dom: PeregrineDom) -> Result<PgCommanderWeb,Message> {
        let commander = PgCommanderWeb::new()?;
        commander.start();
        let configs = config.build();
        let mut inner = PeregrineInnerAPI::new(&configs,&dom,&commander)?;
        run_animations(&mut inner,&dom)?;
        let self2 = self.clone();
        commander.add("draw-api",15,None,None,Box::pin(async move { self2.step(inner).await }));
        Ok(commander)
    }
}
