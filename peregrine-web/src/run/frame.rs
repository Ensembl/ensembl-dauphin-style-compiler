use super::web::PeregrineWeb;
use commander::{ cdr_tick, cdr_current_time };
use peregrine_core::Commander;

fn animation_tick(web: &mut PeregrineWeb, elapsed: f64) {
    web.trainset.animate_tick(elapsed);
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
