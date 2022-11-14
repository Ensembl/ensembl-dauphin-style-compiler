use std::{ hash::{ Hash, Hasher }, fmt };
use std::collections::hash_map::{ DefaultHasher };
use std::error::Error;
use peregrine_message::{ MessageKind, PeregrineMessage };
use peregrine_config::ConfigError;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum DataMessage {
    XXXTransitional(peregrine_toolkit::error::Error),
    CodeInvariantFailed(String),
    ConfigError(ConfigError),
    LengthMismatch(String),
}

impl PeregrineMessage for DataMessage {
    fn kind(&self) -> MessageKind {
        match self {
            _ => MessageKind::Error
        }
    }

    fn code(&self) -> (u64,u64) {
        // Next code is 33; 0 is reserved; 499 is last.
        match self {
            DataMessage::XXXTransitional(s) => (32,calculate_hash(&s.message)),
            DataMessage::CodeInvariantFailed(s) => (15,calculate_hash(s)),
            DataMessage::ConfigError(e) => (17,calculate_hash(e)),
            DataMessage::LengthMismatch(e) => (28,calculate_hash(e)),
        }
    }

    fn knock_on(&self) -> bool {
        match self {
            _ => false
        }
    }

    #[cfg(debug_assertions)]
    fn to_message_string(&self) -> String {
        match self {
            DataMessage::XXXTransitional(e) => format!("{:?}",e),
            DataMessage::CodeInvariantFailed(f) => format!("Code invariant failed: {}",f),
            DataMessage::LengthMismatch(e) => format!("length mismatch: {}",e),
            DataMessage::ConfigError(e) => match e {
                ConfigError::UnknownConfigKey(k) => format!("unknown config key '{}",k),
                ConfigError::BadConfigValue(k,r) => format!("bad config value for key '{}': {}",k,r),
                ConfigError::UninitialisedKey(k) => format!("uninitialised config key {}",k),    
            },
        }
    }

    #[cfg(not(debug_assertions))]
    fn to_message_string(&self) -> String {
        match self {
            DataMessage::XXXTransitional(e) => format!("{:?}",e),
            DataMessage::CodeInvariantFailed(f) => format!("Code invariant failed: {}",f),
            DataMessage::LengthMismatch(e) => format!("length mismatch: {}",e),
            DataMessage::TunnelError(e) => e.lock().unwrap().to_message_string(),
            DataMessage::ConfigError(e) => match e {
                ConfigError::UnknownConfigKey(k) => format!("unknown config key '{}",k),
                ConfigError::BadConfigValue(k,r) => format!("bad config value for key '{}': {}",k,r),
                ConfigError::UninitialisedKey(k) => format!("uninitialised config key {}",k),    
            },
        }
    }

    fn cause_message(&self) -> Option<&(dyn PeregrineMessage + 'static)> {
        self.cause().map(|x| x as &dyn PeregrineMessage)
    }
}

impl DataMessage {
    fn cause(&self) -> Option<&DataMessage> { None }
}

impl fmt::Display for DataMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.to_message_string())
    }
}

impl Error for DataMessage {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause().map(|x| x as &dyn Error)
    }
}

#[cfg(not(debug_assertions))]
impl fmt::Debug for DataMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.to_message_string())
    }    
}