use std::sync::{Arc, Mutex};

use commander::cdr_tick;
use peregrine_data::Commander;

use crate::{Message, PeregrineDom, PeregrineInnerAPI, input::Input, stage::stage::{ReadStage, Stage}, train::GlRailway, webgl::global::WebGlGlobal};

fn mouse_move_tick(input: &Input, mouse_position: &mut Option<(f64,f64)>, stage: &ReadStage, train_set: &GlRailway) -> Result<(),Message> {
    let position = input.get_pointer_last_seen();
    if let Some(position) = position {
        if let Some(old_position) = mouse_position {
            if *old_position == position { return Ok(()); }
        }
        *mouse_position = Some(position);
        input.set_hotspot(train_set.get_hotspot(stage,position)?.len() > 0);
    }
    Ok(())
}

async fn mouse_move_loop(input: Input, train_set: GlRailway, stage: Arc<Mutex<Stage>>) {
    let mut mouse_position: Option<(f64,f64)> = None;
    loop {
        let read_stage = stage.lock().unwrap().read_stage();
        mouse_move_tick(&input,&mut mouse_position,&read_stage,&train_set);
        input.wait_for_mouse_move().await;
        cdr_tick(1).await;
    }
}

pub fn run_mouse_move(web: &mut PeregrineInnerAPI, dom: &PeregrineDom) -> Result<(),Message> {
    let mut other = web.clone();
    web.commander().add_task("mouse-move-animator",0,None,None,Box::pin(async move {
        let lweb = other.lock().await;
        let input = lweb.input.clone();
        let train_set = lweb.trainset.clone();
        let stage = lweb.stage.clone();
        drop(lweb);
        mouse_move_loop(input,train_set,stage).await;
        Ok(())
    }));
    Ok(())
}
