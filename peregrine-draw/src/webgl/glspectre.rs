use crate::{Message, input::{MarchingAnts, Spectre}};
use super::global::WebGlGlobal;

fn draw_ants(gl: &mut WebGlGlobal, ants: &MarchingAnts) -> Result<(),Message> {
    Ok(())
}

pub(crate) fn draw_spectre(gl: &mut WebGlGlobal, spectre: &Spectre) -> Result<(),Message> {
    match spectre {
        Spectre::MarchingAnts(ants) => { draw_ants(gl,ants)?; }
    }
    Ok(())
}
