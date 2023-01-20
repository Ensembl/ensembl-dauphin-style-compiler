use std::{ hash::{ Hash, Hasher }, fmt };
use std::error::Error;
use std::collections::hash_map::{ DefaultHasher };
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use commander::cdr_identity;
use eachorevery::eoestruct::StructValue;
use lazy_static::lazy_static;
use peregrine_data::{DataMessage, GlobalAllotmentMetadata };
use peregrine_message::{MessageKind, PeregrineMessage};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub enum Endstop {
    MaxZoomIn,
    MaxZoomOut,
    Left,
    Right
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum Message {
    CurrentLocation(String,f64,f64),
    TargetLocation(String,f64,f64),
    AllotmentMetadataReport(GlobalAllotmentMetadata),
    HotspotEvent(f64,f64,Vec<StructValue>,Vec<StructValue>),
    HitEndstop(Vec<Endstop>),
    Ready,
    /**/
    CodeInvariantFailed(String),
    DataError(DataMessage),
    ConfusedWebBrowser(String),
    SerializationError(String),
    WebGLFailure(String),
    Canvas2DFailure(String),
    CannotPackRectangles(String),
    BadBackendConnection(String),
    BadTemplate(String),
    BadAsset(String)
}

impl PeregrineMessage for Message {
    fn kind(&self) -> MessageKind {
        match self {
            Message::CurrentLocation(_,_,_) => MessageKind::Interface,
            Message::TargetLocation(_,_,_) => MessageKind::Interface,
            Message::Ready => MessageKind::Interface,
            Message::AllotmentMetadataReport(_) => MessageKind::Interface,
            Message::HotspotEvent(_,_,_,_) => MessageKind::Interface,
            Message::HitEndstop(_) => MessageKind::Interface,
            _ => MessageKind::Error
        }
    }

    fn knock_on(&self) -> bool {
        match self {
            Message::DataError(d) => d.knock_on(),
            _ => false
        }
    }

    fn code(&self) -> (u64,u64) {
        // allowed 500-999; next is 512
        match self {
            Message::CodeInvariantFailed(s) => (503,calculate_hash(s)),
            Message::DataError(d) => d.code(),
            Message::ConfusedWebBrowser(s) => (504,calculate_hash(s)),
            Message::SerializationError(s) => (505,calculate_hash(s)),
            Message::WebGLFailure(s) => (506,calculate_hash(s)),
            Message::Canvas2DFailure(s) => (507,calculate_hash(s)),
            Message::CannotPackRectangles(s) => (509,calculate_hash(s)),
            Message::BadBackendConnection(s) => (510,calculate_hash(s)),
            Message::BadTemplate(s) => (501,calculate_hash(s)),
            Message::BadAsset(s) => (511,calculate_hash(s)),
            Message::CurrentLocation(_,_,_) => (0,0),
            Message::TargetLocation(_,_,_) => (0,0),
            Message::Ready => (0,0),
            Message::AllotmentMetadataReport(_) => (0,0),
            Message::HotspotEvent(_,_,_,_) => (0,0),
            Message::HitEndstop(_) => (0,0),
        }
    }

    fn to_message_string(&self) -> String {
        match self {
            Message::CodeInvariantFailed(s) => format!("code invariant violated: {}",s),
            Message::DataError(d) => d.to_string(),
            Message::ConfusedWebBrowser(s) => format!("confused web browser: {}",s),
            Message::SerializationError(s) => format!("serialization error: {}",s),
            Message::WebGLFailure(s) => format!("WebGL failure: {}",s),
            Message::Canvas2DFailure(s) => format!("2D canvas failues: {}",s),
            Message::CannotPackRectangles(s) => format!("cannot pack rectangles: {}",s),
            Message::BadBackendConnection(s) => format!("bad backend connection: {}",s),
            Message::BadTemplate(s) => format!("bad template: {}",s),
            Message::BadAsset(s) => format!("bad asset: {}",s),
            Message::CurrentLocation(stick,left,right) => format!("current location: {}:{}-{}",stick,left,right),
            Message::TargetLocation(stick,left,right) => format!("target location: {}:{}-{}",stick,left,right),
            Message::Ready => format!("ready"),
            Message::AllotmentMetadataReport(metadata) => format!("allotment metadata: {:?}",metadata.summarize_json()),
            Message::HotspotEvent(x,y,variety,contents) => format!("click event: {:?} : {:?} at ({},{})",
                variety.iter().map(|x| x.to_json_value().to_string()).collect::<Vec<_>>().join(","),
                contents.iter().map(|x| x.to_json_value().to_string()).collect::<Vec<_>>().join(","),
                x,y),
            Message::HitEndstop(x) => format!("hit endstop: {:?}",x.iter().map(|y| format!("{:?}",y)).collect::<Vec<_>>().join(", ")),
        }
    }
}

impl Message {
    fn cause(&self) -> Option<&Message> {
        None
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.to_message_string())
    }
}

#[cfg(not(debug_assertions))]
impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.to_message_string())
    }    
}

impl Error for Message {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause().map(|x| x as &dyn Error)
    }
}

struct MessageCatcher {
    senders: HashMap<Option<u64>,Box<dyn FnMut(Message) + 'static + Send>>,
    default: Option<u64>
}

impl MessageCatcher {
    fn new() -> MessageCatcher {
        MessageCatcher {
            senders: HashMap::new(),
            default: None
        }
    }

    fn default(&mut self, v: u64) { self.default = Some(v); }

    fn add<F>(&mut self, id: Option<u64>, cb: F) where F: FnMut(Message) + 'static + Send {
        self.senders.insert(id,Box::new(cb));
    }

    fn call(&mut self, id : Option<u64>, message: Message) {
        let id = id.or_else(|| self.default);
        if let Some(sender) = self.senders.get_mut(&id) {
            sender(message);
        }
    }
}

lazy_static! {
    static ref MESSAGE_CATCHER : Arc<Mutex<MessageCatcher>> = Arc::new(Mutex::new(MessageCatcher::new()));
}    

pub(crate) fn message_register_default(id: u64) {
    MESSAGE_CATCHER.lock().unwrap().default(id);
}

pub(crate) fn message_register_callback<F>(id: Option<u64>,cb: F) where F: FnMut(Message) + 'static + Send {
    MESSAGE_CATCHER.lock().unwrap().add(id,cb);
}

pub(crate) fn routed_message(id: Option<u64>, message: Message) {
    MESSAGE_CATCHER.lock().unwrap().call(id,message);    
}

pub(crate) fn message(message: Message) {
    let id = cdr_identity().map(|x| x.0);
    routed_message(id,message);
}
