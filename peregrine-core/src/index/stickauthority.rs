use blackbox::blackbox_log;
use crate::lock;
use crate::request::{ Channel, RequestManager };
use crate::request::program::ProgramLoader;
use crate::request::stickauthority::get_stick_authority_program;
use crate::run::{ PgDauphin, PgDauphinTaskSpec };
use std::sync::{ Arc, Mutex };

pub struct StickAuthority {
    channel: Channel,
    program_name: String
}

impl StickAuthority {
    pub fn new(channel: &Channel, program_name: &str) -> StickAuthority {
        blackbox_log!("stickauthority","");
        StickAuthority {
            channel: channel.clone(),
            program_name: program_name.to_string()
        }
    }
}
