use crate::PeregrineInnerAPI;
use crate::{input::InputEvent};
use crate::run::{ PgPeregrineConfig };
use crate::input::{ InputEventKind };
use crate::input::low::lowlevel::LowLevelInput;
use crate::util::Message;
use peregrine_toolkit::log;

fn debug(inner: &mut PeregrineInnerAPI, index: u8) {
    log!("sending debug action {}",index);
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
