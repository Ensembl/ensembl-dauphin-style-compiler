use crate::{PeregrineAPI, input::InputEvent};
use crate::run::{ PgPeregrineConfig };
use crate::input::{ InputEventKind };
use crate::input::low::lowlevel::LowLevelInput;
use crate::util::Message;
use web_sys::console;

fn debug(api: &PeregrineAPI, index: u8) {
    console::log_1(&format!("sending debug action {}",index).into());
    api.debug_action(index);
}

fn check_debug(e: &InputEvent, api: &PeregrineAPI) {
    if e.details == InputEventKind::DebugAction && e.start {
        debug(api,*e.amount.get(0).unwrap_or(&0.) as u8);
    }
}

pub fn debug_register(_config: &PgPeregrineConfig, low_level: &mut LowLevelInput, api: &PeregrineAPI) -> Result<(),Message> {
    let api = api.clone();
    low_level.distributor_mut().add(move |e| check_debug(e,&api));
    Ok(())
}
