use super::web::PeregrineWeb;
use commander::{ cdr_tick, cdr_current_time };
use peregrine_data::Commander;

fn animation_tick(web: &mut PeregrineWeb, elapsed: f64) {
    let mut webgl = web.webgl.lock().unwrap();
    web.trainset.transition_animate_tick(&web.data_api,&mut webgl,elapsed);
    if web.test_and_reset_redraw() {
        web.trainset.draw_animate_tick(&web.read_stage(),&mut webgl);
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
