use std::hash::{ Hash, Hasher };
use std::collections::hash_map::{ DefaultHasher };

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
