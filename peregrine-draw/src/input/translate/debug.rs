use crate::{PeregrineAPI, input::InputEvent};
use crate::run::{ PgPeregrineConfig,  PgConfigKey };
use crate::input::{ InputEventKind };
use crate::input::low::lowlevel::LowLevelInput;
use crate::util::Message;
use web_sys::console;

fn debug(api: &PeregrineAPI) {
    console::log_1(&format!("position report. stick={:?} x={:?} bp_per_screen={:?}",
        api.stick(), api.x(), api.bp_per_screen()
    ).into());
}

fn check_debug(e: &InputEvent, api: &PeregrineAPI) {
    if e.details == InputEventKind::PositionReport && e.start {
        debug(api);
    }
}

pub fn debug_register(config: &PgPeregrineConfig, low_level: &mut LowLevelInput, api: &PeregrineAPI) -> Result<(),Message> {
    let api = api.clone();
    low_level.distributor_mut().add(move |e| check_debug(e,&api));
    Ok(())
}
