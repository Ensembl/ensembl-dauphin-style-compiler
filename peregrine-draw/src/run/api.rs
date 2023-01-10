use crate::util::message::{ Message };
use eachorevery::eoestruct::StructValue;
use peregrine_toolkit::console::{set_printer, Severity};
use peregrine_toolkit::{log_extra, log_important};
use peregrine_toolkit_async::sync::blocker::Blocker;
use peregrine_toolkit_async::sync::needed::Needed;
pub use url::Url;
use wasm_bindgen::JsValue;
pub use web_sys::{ console, WebGlRenderingContext, Element };
use peregrine_data::{ StickId, Commander };
use super::buildconfig::{ GIT_TAG, GIT_BUILD_DATE };
use super::mousemove::run_mouse_move;
use commander::CommanderStream;
use super::inner::PeregrineInnerAPI;
use crate::integration::pgcommander::PgCommanderWeb;
use crate::run::globalconfig::PeregrineConfig;
use super::frame::run_animations;
use crate::domcss::dom::PeregrineDom;

use std::sync::{ Arc, Mutex };

#[derive(Clone)]
struct DrawMessageQueue {
    queue: CommanderStream<Option<DrawMessage>>,
    syncer: Blocker
}

impl DrawMessageQueue {
    fn new() -> DrawMessageQueue {
        let syncer = Blocker::new();
        syncer.set_freewheel(true);
        DrawMessageQueue {
            queue: CommanderStream::new(),
            syncer
        }
    }

    fn syncer(&self) -> &Blocker { &self.syncer }

    fn add(&self, message: Option<DrawMessage>) {
        self.queue.add(message);
    }

    async fn get(&self) -> Option<DrawMessage> {
        let message = self.queue.get().await;
        self.syncer.wait().await;
        self.syncer.set_freewheel(true);
        message
    }

    fn sync(&self) {
        self.syncer.set_freewheel(false);
    }
}

enum DrawMessage {
    Goto(f64,f64),
    SetY(f64),
    SetStick(StickId),
    Switch(Vec<String>,StructValue),
    RadioSwitch(Vec<String>,bool),
    SetMessageReporter(Box<dyn FnMut(&Message) + 'static + Send>),
    DebugAction(u8),
    SetArtificial(String,bool),
    Jump(String),
    Sync(),
    AddJsapiChannel(String,JsValue)
}

impl std::fmt::Debug for DrawMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DrawMessage::Goto(centre,scale) => write!(f,"Goto({:?},{:?})",centre,scale),
            DrawMessage::SetY(y) => write!(f,"SetY({:?})",y),
            DrawMessage::SetStick(stick)  => write!(f,"SetStick({:?})",stick),
            DrawMessage::Switch(path,value) => write!(f,"Switch({:?},{:?})",path,value.to_json_value().to_string()),
            DrawMessage::RadioSwitch(path,yn)  => write!(f,"RadioSwitch({:?},{:?})",path,yn),
            DrawMessage::SetMessageReporter(_) => write!(f,"SetMessageReporter(...)"),
            DrawMessage::DebugAction(index)  => write!(f,"DebugAction({:?})",index),
            DrawMessage::SetArtificial(name,start) => write!(f,"SetArtificial({:?},{:?})",name,start),
            DrawMessage::Jump(location) => write!(f,"Jump({})",location),
            DrawMessage::Sync() => write!(f,"Sync"),
            DrawMessage::AddJsapiChannel(name,channel) => write!(f,"AddJsapiChannel({},...)",name)
        }
    }
}

#[cfg(debug_assertions)]
fn dev_warning() {
    use peregrine_toolkit::warn;

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
        warn!("{}",&line.trim());  
    }
}

impl DrawMessage {
    fn run(self, draw: &mut PeregrineInnerAPI, blocker: &Blocker) -> Result<(),Message> {
        log_extra!("message {:?}",self);
        match self {
            DrawMessage::Goto(centre,scale) => {
                draw.goto(centre,scale);
            },
            DrawMessage::SetArtificial(name,down) => {
                draw.set_artificial(&name,down);
            },
            DrawMessage::SetY(y) => {
                draw.set_y(y);
            },
            DrawMessage::SetStick(stick) => {
                draw.set_stick(&stick);
            },
            DrawMessage::Switch(path,value) => {
                draw.switch(&path.iter().map(|x| &x as &str).collect::<Vec<_>>(),value);
            },
            DrawMessage::RadioSwitch(path,yn) => {
                draw.radio_switch(&path.iter().map(|x| &x as &str).collect::<Vec<_>>(),yn);
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
            DrawMessage::AddJsapiChannel(name,payload) => {
                draw.add_jsapi_channel(&name,payload);
            }
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

    pub fn set_y(&self, y: f64) {
        self.queue.add(Some(DrawMessage::SetY(y)));
    }

    pub fn goto(&self, left: f64, right: f64) {
        let (left,right) = (left.min(right),left.max(right));
        self.queue.add(Some(DrawMessage::Goto((left+right)/2.,right-left)));
    }

    pub fn jump(&self, location: &str) {
        self.queue.add(Some(DrawMessage::Jump(location.to_string())));
    }

    pub fn wait(&self) {
        self.queue.add(Some(DrawMessage::Sync()));
    }

    pub fn switch(&self, path: &[&str], value: StructValue) {
        self.queue.add(Some(DrawMessage::Switch(path.iter().map(|x| x.to_string()).collect(),value)));
    }

    pub fn radio_switch(&self, path: &[&str], yn: bool) {
        self.queue.add(Some(DrawMessage::RadioSwitch(path.iter().map(|x| x.to_string()).collect(),yn)));
    }

    pub fn set_stick(&self, stick: &StickId) {
        *self.stick.lock().unwrap() = Some(stick.get_id().to_string()); // XXX not really true yet: have proper ro status via data
        self.queue.add(Some(DrawMessage::SetStick(stick.clone())));
    }

    pub fn set_message_reporter(&self, callback: Box<dyn FnMut(&Message) + 'static + Send>) {
        self.queue.add(Some(DrawMessage::SetMessageReporter(callback)));
    }

    pub fn debug_action(&self, index:u8) {
        self.queue.add(Some(DrawMessage::DebugAction(index)));
    }

    pub fn shutdown(&self) {
        self.queue.add(None);
    }

    pub fn stick(&self) -> Option<String> { self.stick.lock().unwrap().as_ref().cloned() }

    pub fn add_jsapi_channel(&self, name: &str, payload: JsValue) {
        self.queue.add(Some(DrawMessage::AddJsapiChannel(name.to_string(),payload)))
    }

    pub fn set_artificial(&self, name: &str, start: bool) {
        self.queue.add(Some(DrawMessage::SetArtificial(name.to_string(),start)));
    }

    async fn step(&self, mut draw: PeregrineInnerAPI) -> Result<(),Message> {
        log_important!("version {} {} {}.",GIT_TAG,GIT_BUILD_DATE,env!("BUILD_TIME"));
        #[cfg(debug_assertions)]
        dev_warning();
        while let Some(message) = self.queue.get().await{
            message.run(&mut draw,&self.queue.syncer())?;
        }
        log_extra!("draw-api quit");
        Ok(())
    }

    pub fn run(&self, config: PeregrineConfig, el: &Element, backend: &str) -> Result<PgCommanderWeb,Message> {
        let commander = PgCommanderWeb::new()?;
        commander.start();
        let redraw_needed = Needed::new();
        let dom = PeregrineDom::new(&commander,el,&redraw_needed)?;
        set_printer(|severity,message| {
            match severity {
                Severity::Error => { console::error_1(&message.into()); },
                Severity::Warning => { console::warn_1(&message.into()); },
                Severity::Notice => { console::log_1(&message.into()); },
            }
        });
        let configs = config.build();
        let mut inner = PeregrineInnerAPI::new(&configs,&dom,&commander,backend,self.queue.syncer(),&redraw_needed)?;
        run_animations(&mut inner,&dom)?;
        run_mouse_move(&mut inner,&dom)?;
        let self2 = self.clone();
        commander.add("draw-api",0,None,None,Box::pin(async move { self2.step(inner).await }));
        let queue = self.queue.clone();
        dom.shutdown().add(move || queue.add(None));
        Ok(commander)
    }
}
