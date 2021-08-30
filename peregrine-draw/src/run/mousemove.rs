use commander::cdr_tick;
use peregrine_data::Commander;

use crate::{Message, PeregrineDom, PeregrineInnerAPI, input::Input};

fn process_mouse_position(input: &Input, position: (f64,f64)) {
    use web_sys::console;
    console::log_1(&format!("zmenu position2 {:?}",position).into());
}

fn mouse_move_tick(input: &Input, mouse_position: &mut Option<(f64,f64)>) {
    let position = input.get_pointer_last_seen();
    if let Some(position) = position {
        if let Some(old_position) = mouse_position {
            if *old_position == position { return; }
        }
        *mouse_position = Some(position);
        process_mouse_position(input,position);
    }
}

async fn mouse_move_loop(input: Input) {
    let mut mouse_position: Option<(f64,f64)> = None;
    loop {
        mouse_move_tick(&input,&mut mouse_position);
        input.wait_for_mouse_move().await;
        cdr_tick(1).await;
    }
}

pub fn run_mouse_move(web: &mut PeregrineInnerAPI, dom: &PeregrineDom) -> Result<(),Message> {
    let mut other = web.clone();
    let dom = dom.clone();
    web.commander().add_task("mouse-move-animator",0,None,None,Box::pin(async move {
        let lweb = other.lock().await;
        let input = lweb.input.clone();
        drop(lweb);
        mouse_move_loop(input).await;
        Ok(())
    }));
    Ok(())
}
