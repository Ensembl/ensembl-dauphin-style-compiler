use crate::util::message::{ Message };
pub use url::Url;
pub use web_sys::{ console, WebGlRenderingContext, Element };
use peregrine_data::{ Channel, StickId, Commander };
use super::progress::Progress;
use commander::CommanderStream;
use peregrine_message::Instigator;
use super::inner::PeregrineInnerAPI;
use crate::stage::stage::Position;
use super::dom::PeregrineDom;
use crate::integration::pgcommander::PgCommanderWeb;
use crate::run::globalconfig::PeregrineConfig;
use crate::input::Input;
use crate::run::inner::Target;

use std::sync::{ Arc, Mutex };

enum DrawMessage {
    SetX(f64),
    SetY(f64),
    SetBpPerScreen(f64),
    SetStick(StickId),
    SetSwitch(Vec<String>),
    ClearSwitch(Vec<String>),
    Bootstrap(Channel),
    SetupBlackbox(String),
    SetMessageReporter(Box<dyn FnMut(Message) + 'static + Send>),
    DebugAction(u8)
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
            DrawMessage::SetupBlackbox(url) => {
                let e = draw.setup_blackbox(&url);
                if let Err(e) = e { instigator.error(e); }
                instigator.done();
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
    }
}

#[derive(Clone)]
pub struct PeregrineAPI {
    queue: CommanderStream<(DrawMessage,Instigator<Message>)>,
    input: Arc<Mutex<Option<Input>>>,
    stick: Arc<Mutex<Option<String>>>,
    position: Arc<Mutex<Option<Position>>>,
    target: Arc<Mutex<Target>>
}

impl PeregrineAPI {
    pub fn new() -> PeregrineAPI {
        PeregrineAPI {
            queue: CommanderStream::new(),
            position: Arc::new(Mutex::new(None)),
            stick: Arc::new(Mutex::new(None)),
            input: Arc::new(Mutex::new(None)),
            target: Arc::new(Mutex::new(Target::new()))
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

    pub fn setup_blackbox(&self, url: &str) -> Progress {
        let (progress,insitgator) = Progress::new();
        self.queue.add((DrawMessage::SetupBlackbox(url.to_string()),insitgator.clone()));
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

    pub fn x(&self) -> Option<f64> { self.position.lock().unwrap().as_ref().map(|p| p.x) }
    pub fn y(&self) -> Option<f64> { self.position.lock().unwrap().as_ref().map(|p| p.y) }
    pub fn stick(&self) -> Option<String> { self.stick.lock().unwrap().as_ref().cloned() }
    pub fn bp_per_screen(&self) -> Option<f64> { self.position.lock().unwrap().as_ref().map(|p| p.bp_per_screen) }
    pub fn size(&self) -> Option<(u32,u32)> { self.target.lock().unwrap().size().cloned() }

    async fn step(&self, mut draw: PeregrineInnerAPI) -> Result<(),()> {
        loop {
            let (message,instigator) = self.queue.get().await;
            message.run(&mut draw,instigator);
        }
    }

    pub fn run(&self, config: PeregrineConfig, dom: PeregrineDom) -> Result<PgCommanderWeb,Message> {
        let commander = PgCommanderWeb::new(&dom)?;
        commander.start();
        let configs = config.build();
        *self.input.lock().unwrap() = Some(Input::new(&dom,&configs.draw,&self,&commander)?);
        let inner = PeregrineInnerAPI::new(configs,dom,&commander)?;
        let self2 = self.clone();
        inner.xxx_set_callbacks(move |p| {
            *self2.position.lock().unwrap() = p;
        });
        let self2 = self.clone();
        inner.add_target_callback(move |p| {
            *self2.target.lock().unwrap() = p.clone();
        });
        let self2 = self.clone();
        commander.add("draw-api",15,None,None,Box::pin(async move { self2.step(inner).await }));
        Ok(commander)
    }
}
