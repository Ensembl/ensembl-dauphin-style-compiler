use crate::PeregrineInnerAPI;
use crate::{input::InputEvent};
use crate::run::{ PgPeregrineConfig };
use crate::input::{ InputEventKind };
use crate::input::low::lowlevel::LowLevelInput;
use crate::util::Message;
use web_sys::console;

fn debug(inner: &mut PeregrineInnerAPI, index: u8) {
    console::log_1(&format!("sending debug action {}",index).into());
    inner.debug_action(index);
}

fn check_debug(e: &InputEvent, inner: &mut PeregrineInnerAPI) {
    if e.details == InputEventKind::DebugAction && e.start {
        debug(inner,*e.amount.get(0).unwrap_or(&0.) as u8);
    }
}

pub fn debug_register(_config: &PgPeregrineConfig, low_level: &mut LowLevelInput, inner: &PeregrineInnerAPI) -> Result<(),Message> {
    let mut inner = inner.clone();
    low_level.distributor_mut().add(move |e| check_debug(e,&mut inner));
    Ok(())
}
