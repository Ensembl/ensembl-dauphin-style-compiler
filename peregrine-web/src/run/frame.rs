use super::web::PeregrineWeb;
use commander::{ cdr_tick, cdr_current_time };
use peregrine_core::Commander;

fn animation_tick(web: &mut PeregrineWeb, elapsed: f64) {
    let mut webgl = web.webgl.lock().unwrap();
    web.trainset.transition_animate_tick(&mut webgl,elapsed);
    if web.stage().redraw_needed().test_and_reset() {
        web.trainset.draw_animate_tick(&web.stage().clone(),&webgl);
    }
}

async fn animation_tick_loop(mut web: PeregrineWeb) {
    let mut start = cdr_current_time();
    loop {
        let next = cdr_current_time();
        animation_tick(&mut web,next-start);
        cdr_tick(1).await;
        start = next;
    }
}

pub fn run_animations(web: &PeregrineWeb) {
    let other = web.clone();
    web.commander.add_task("animator",0,None,None,Box::pin(async move {
        animation_tick_loop(other).await;
        Ok(())
    }));
}
