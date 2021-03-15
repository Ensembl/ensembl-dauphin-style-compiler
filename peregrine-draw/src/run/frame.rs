use super::draw::{ PeregrineDraw, LockedPeregrineDraw };
use commander::{ cdr_tick, cdr_current_time };
use peregrine_data::Commander;
use crate::util::message::Message;

fn animation_tick(web: &mut LockedPeregrineDraw, elapsed: f64) -> Result<(),Message> {
    let read_stage = &web.stage.lock().unwrap().read_stage();
    web.trainset.transition_animate_tick(&web.data_api,&mut web.webgl.lock().unwrap(),elapsed)?;
    if read_stage.ready() {
        web.trainset.draw_animate_tick(read_stage,&mut web.webgl.lock().unwrap())?;
    }
    Ok(())
}

async fn animation_tick_loop(mut web: PeregrineDraw) {
    let mut start = cdr_current_time();
    let lweb = web.lock().await;
    let mut redraw = lweb.stage.lock().unwrap().redraw_needed().clone();
    drop(lweb);
    loop {
        let next = cdr_current_time();
        animation_tick(&mut web.lock().await,next-start);
        cdr_tick(1).await;
        redraw.wait_until_redraw_needed().await;
        start = next;
    }
}

pub fn run_animations(web: &PeregrineDraw) {
    let other = web.clone();
    web.commander().add_task("animator",0,None,None,Box::pin(async move {
        animation_tick_loop(other).await;
        Ok(())
    }));
}
