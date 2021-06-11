use crate::{Message, input::Spectre};
use super::global::WebGlGlobal;

pub(crate) fn draw_spectre( gl: &mut WebGlGlobal, spectre: &Spectre) -> Result<(),Message> {
    use web_sys::console;
    console::log_1(&format!("spectre {:?}",spectre).into());
    Ok(())
}
