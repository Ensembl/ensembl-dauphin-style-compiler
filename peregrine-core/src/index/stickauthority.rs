use blackbox::blackbox_log;
use crate::lock;
use crate::request::{ Channel, RequestManager };
use crate::request::program::ProgramLoader;
use crate::run::{ PgDauphin, PgDauphinTaskSpec };
use std::sync::{ Arc, Mutex };

pub struct StickAuthority {
    channel: Channel,
    startup_program_name: String,
    resolution_program_name: String
}

impl StickAuthority {
    pub fn new(channel: &Channel, startup_program_name: &str, resolution_program_name: &str) -> StickAuthority {
        blackbox_log!("stickauthority","");
        StickAuthority {
            channel: channel.clone(),
            startup_program_name: startup_program_name.to_string(),
            resolution_program_name: resolution_program_name.to_string(),
        }
    }

    pub fn channel(&self) -> &Channel { &self.channel }
    pub fn startup_program(&self) -> &str { &self.startup_program_name }
    pub fn lookup_program(&self) -> &str { &self.resolution_program_name }
}
