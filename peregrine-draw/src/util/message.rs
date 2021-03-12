use std::hash::{ Hash, Hasher };
use std::collections::hash_map::{ DefaultHasher };
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use commander::cdr_identity;
use lazy_static::lazy_static;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
pub enum MessageLevel {
    Notice,
    Warn,
    Error
}

pub enum MessageCategory {
    BadIntegration,
    BadInternals,
    ServerUnavilable,
}

pub enum MessageSuggestion {
    WarnOnConsole,
    NotifyUser,
    LogToBackendService,
    LogToFrontendService
}

pub enum Message {
    DroppedWithoutTidying(String)
}

impl Message {
    fn level(&self) -> MessageLevel {
        match self {
            Message::DroppedWithoutTidying(_) => MessageLevel::Warn
        }
    }

    fn category(&self) -> MessageCategory {
        match self {
            Message::DroppedWithoutTidying(_) => MessageCategory::BadInternals
        }
    }

    fn fatal(&self) -> bool {
        match self {
            Message::DroppedWithoutTidying(_) => false
        }
    }

    fn suggestions(&self) ->&[MessageSuggestion] {
        match self {
            Message::DroppedWithoutTidying(_) => &[MessageSuggestion::WarnOnConsole,MessageSuggestion::LogToBackendService]
        }
    }

    fn code(&self) -> (u64,u64) {
        match self {
            Message::DroppedWithoutTidying(s) => (0,calculate_hash(s))
        }
    }
}

impl ToString for Message {
    fn to_string(&self) -> String {
        match self {
            Message::DroppedWithoutTidying(s) => format!("dropped object without tidying: {}",s)
        }
    }
}

struct MessageCatcher {
    senders: HashMap<Option<u64>,Box<dyn FnMut(Message) + 'static + Send>>
}

impl MessageCatcher {
    fn new() -> MessageCatcher {
        MessageCatcher {
            senders: HashMap::new()
        }
    }

    fn add<F>(&mut self, id: Option<u64>, cb: F) where F: FnMut(Message) + 'static + Send {
        self.senders.insert(id,Box::new(cb));
    }

    fn call(&mut self, id : Option<u64>, message: Message) {
        if let Some(sender) = self.senders.get_mut(&id) {
            sender(message);
        }
    }
}

lazy_static! {
    static ref message_catcher : Arc<Mutex<MessageCatcher>> = Arc::new(Mutex::new(MessageCatcher::new()));
}    

pub(crate) fn message_register_callback<F>(id: Option<u64>,cb: F) where F: FnMut(Message) + 'static + Send {
    message_catcher.lock().unwrap().add(id,cb);
}

pub(crate) fn message(message: Message) {
    let id = cdr_identity().map(|x| x.0);
    message_catcher.lock().unwrap().call(id,message);
}
