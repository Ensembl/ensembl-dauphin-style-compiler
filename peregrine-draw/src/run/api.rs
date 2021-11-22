use crate::util::message::{ Message };
use peregrine_toolkit::sync::blocker::Blocker;
pub use url::Url;
pub use web_sys::{ console, WebGlRenderingContext, Element };
use peregrine_data::{ Channel, StickId, Commander };
use super::mousemove::run_mouse_move;
use super::{config::DebugFlag };
use commander::CommanderStream;
use super::inner::PeregrineInnerAPI;
use super::dom::PeregrineDom;
use crate::integration::pgcommander::PgCommanderWeb;
use crate::run::globalconfig::PeregrineConfig;
use crate::run::config::{ PgConfigKey };
use super::frame::run_animations;
use super::PgPeregrineConfig;

use std::sync::{ Arc, Mutex };

#[derive(Clone)]
struct DrawMessageQueue {
    queue: CommanderStream<DrawMessage>,
    blocker: Blocker
}

impl DrawMessageQueue {
    fn new() -> DrawMessageQueue {
        let blocker = Blocker::new();
        blocker.set_freewheel(true);
        DrawMessageQueue {
            queue: CommanderStream::new(),
            blocker
        }
    }

    fn blocker(&self) -> &Blocker { &self.blocker }

    fn add(&self, message: DrawMessage) {
        self.queue.add(message);
    }

    async fn get(&self) -> DrawMessage {
        let message = self.queue.get().await;
        self.blocker.wait().await;
        self.blocker.set_freewheel(true);
        message
    }

    fn sync(&self) {
        self.blocker.set_freewheel(false);
    }
}

enum DrawMessage {
    Goto(f64,f64),
    SetY(f64),
    SetStick(StickId),
    SetSwitch(Vec<String>),
    ClearSwitch(Vec<String>),
    RadioSwitch(Vec<String>,bool),
    Bootstrap(Channel),
    SetMessageReporter(Box<dyn FnMut(&Message) + 'static + Send>),
    DebugAction(u8),
    SetArtificial(String,bool),
    Jump(String),
    Sync()
}

impl std::fmt::Debug for DrawMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DrawMessage::Goto(centre,scale) => write!(f,"Goto({:?},{:?})",centre,scale),
            DrawMessage::SetY(y) => write!(f,"SetY({:?})",y),
            DrawMessage::SetStick(stick)  => write!(f,"SetStick({:?})",stick),
            DrawMessage::SetSwitch(path) => write!(f,"SetSwitch({:?})",path),
            DrawMessage::ClearSwitch(path)  => write!(f,"ClearSwitch({:?})",path),
            DrawMessage::RadioSwitch(path,yn)  => write!(f,"RadioSwitch({:?},{:?})",path,yn),
            DrawMessage::Bootstrap(channel)  => write!(f,"Channel({:?})",channel),
            DrawMessage::SetMessageReporter(_) => write!(f,"SetMessageReporter(...)"),
            DrawMessage::DebugAction(index)  => write!(f,"DebugAction({:?})",index),
            DrawMessage::SetArtificial(name,start) => write!(f,"SetArtificial({:?},{:?})",name,start),
            DrawMessage::Jump(location) => write!(f,"Jump({})",location),
            DrawMessage::Sync() => write!(f,"Sync")
        }
    }
}

#[cfg(force_show_incoming)]
fn show_incoming(config: &PgPeregrineConfig) -> Result<bool,Message> { Ok(true) }

#[cfg(debug_assertions)]
fn dev_warning() {
    let message = r#"
                    ******************************
                    * This is a dev build. Expect it to be 
                    * cranky and slow with real data.
                    *
                    * Do not submit performance bugs
                    * against this build.
                    *
                    * Do not expect this to work for 
                    * very large chromosomes
                    ******************************"#;
    for line in message.split("\n") {
        console::warn_1(&line.trim().into());  
    }
}


#[cfg(not(force_show_incoming))]
fn show_incoming(config: &PgPeregrineConfig) -> Result<bool,Message> { config.get_bool(&PgConfigKey::DebugFlag(DebugFlag::ShowIncomingMessages)) }

impl DrawMessage {
    fn run(self, draw: &mut PeregrineInnerAPI, blocker: &Blocker) -> Result<(),Message> {
        if show_incoming(draw.config())? {
            console::log_1(&format!("message {:?}",self).into());
        }
        match self {
            DrawMessage::Goto(centre,scale) => {
                draw.goto(centre,scale);
            },
            DrawMessage::SetArtificial(name,down) => {
                draw.set_artificial(&name,down);
            }
            DrawMessage::SetY(y) => {
                draw.set_y(y);
            },
            DrawMessage::SetStick(stick) => {
                draw.set_stick(&stick);
            },
            DrawMessage::SetSwitch(path) => {
                draw.set_switch(&path.iter().map(|x| &x as &str).collect::<Vec<_>>());
            },
            DrawMessage::ClearSwitch(path) => {
                draw.clear_switch(&path.iter().map(|x| &x as &str).collect::<Vec<_>>());
            },
            DrawMessage::RadioSwitch(path,yn) => {
                draw.radio_switch(&path.iter().map(|x| &x as &str).collect::<Vec<_>>(),yn);
            },
            DrawMessage::Bootstrap(channel) => {
                draw.bootstrap(channel.clone());
            },
            DrawMessage::SetMessageReporter(cb) => {
                draw.set_message_reporter(cb);
            },
            DrawMessage::DebugAction(index) => {
                draw.debug_action(index);
            },
            DrawMessage::Jump(location) => {
                draw.jump(&location);
            },
            DrawMessage::Sync() => {
                blocker.set_freewheel(false);
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct PeregrineAPI {
    queue: DrawMessageQueue,
    stick: Arc<Mutex<Option<String>>>
}

impl PeregrineAPI {
    pub fn new() -> PeregrineAPI {
        PeregrineAPI {
            queue: DrawMessageQueue::new(),
            stick: Arc::new(Mutex::new(None))
        }
    }

    pub fn bootstrap(&self, channel: &Channel) {
        self.queue.add(DrawMessage::Bootstrap(channel.clone()));
    }

    pub fn set_y(&self, y: f64) {
        self.queue.add(DrawMessage::SetY(y));
    }

    pub fn goto(&self, left: f64, right: f64) {
        let (left,right) = (left.min(right),left.max(right));
        self.queue.add(DrawMessage::Goto((left+right)/2.,right-left));
    }

    pub fn jump(&self, location: &str) {
        self.queue.add(DrawMessage::Jump(location.to_string()));
    }

    pub fn wait(&self) {
        self.queue.add(DrawMessage::Sync());
    }

    pub fn set_switch(&self, path: &[&str]) {
        self.queue.add(DrawMessage::SetSwitch(path.iter().map(|x| x.to_string()).collect()));
    }

    pub fn clear_switch(&self, path: &[&str]) {
        self.queue.add(DrawMessage::ClearSwitch(path.iter().map(|x| x.to_string()).collect()));
    }

    pub fn radio_switch(&self, path: &[&str], yn: bool) {
        self.queue.add(DrawMessage::RadioSwitch(path.iter().map(|x| x.to_string()).collect(),yn));
    }

    pub fn set_stick(&self, stick: &StickId) {
        *self.stick.lock().unwrap() = Some(stick.get_id().to_string()); // XXX not really true yet: have proper ro status via data
        self.queue.add(DrawMessage::SetStick(stick.clone()));
    }

    pub fn set_message_reporter(&self, callback: Box<dyn FnMut(&Message) + 'static + Send>) {
        self.queue.add(DrawMessage::SetMessageReporter(callback));
    }

    pub fn debug_action(&self, index:u8) {
        self.queue.add(DrawMessage::DebugAction(index));
    }

    pub fn stick(&self) -> Option<String> { self.stick.lock().unwrap().as_ref().cloned() }

    pub fn set_artificial(&self, name: &str, start: bool) {
        self.queue.add(DrawMessage::SetArtificial(name.to_string(),start));
    }

    async fn step(&self, mut draw: PeregrineInnerAPI) -> Result<(),Message> {
        let git_tag = env!("GIT_TAG");
        let git_tag = if git_tag != "" { format!("tag {}",git_tag) } else { format!("no tag") };
        if show_incoming(draw.config())? {
            console::log_1(&format!("compilation: git {:?} ({}) build time {:?} build host {:?}",
                env!("GIT_HASH"),git_tag,env!("BUILD_TIME"),env!("BUILD_HOST")).into());
        }
        #[cfg(debug_assertions)]
        dev_warning();
        loop {
            let message = self.queue.get().await;
            message.run(&mut draw,&self.queue.blocker())?;
        }
    }

    pub fn run(&self, config: PeregrineConfig, dom: PeregrineDom) -> Result<PgCommanderWeb,Message> {
        let commander = PgCommanderWeb::new()?;
        commander.start();
        let configs = config.build();
        let mut inner = PeregrineInnerAPI::new(&configs,&dom,&commander,self.queue.blocker())?;
        run_animations(&mut inner,&dom)?;
        run_mouse_move(&mut inner,&dom)?;
        let self2 = self.clone();
        commander.add("draw-api",0,None,None,Box::pin(async move { self2.step(inner).await }));
        Ok(commander)
    }
}
