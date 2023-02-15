use std::sync::{Arc, Mutex};
use commander::cdr_tick;
use peregrine_data::Commander;
use peregrine_toolkit::{plumbing::oneshot::OneShot, log_extra};
use crate::{Message, PeregrineInnerAPI, input::{Input, translate::translatehotspots::filter_hotspot_events}, stage::stage::{ReadStage, Stage}, train::GlRailway, domcss::dom::PeregrineDom};

fn mouse_move_tick(input: &Input, mouse_position: &mut Option<(f64,f64)>, stage: &ReadStage, train_set: &GlRailway) -> Result<(),Message> {
    let position = input.get_pointer_last_seen();
    if let Some(mut position) = position {
        position.1 += stage.y().position().unwrap_or(0.);
        if let Some(old_position) = mouse_position {
            if *old_position == position { return Ok(()); }
        }
        *mouse_position = Some(position);
        let hotspot = train_set.get_hotspot(stage,position)?;
        let mut hotspot = filter_hotspot_events(hotspot,false);
        let any = hotspot.len() > 0;
        let hover_hotspots = hotspot.drain(..).map(|x| x.entry).filter(|x| x.is_hover()).collect::<Vec<_>>();
        let special_hotspots = train_set.special_hotspots(stage,position)?;
        input.set_hotspot(any,hover_hotspots,&special_hotspots,position);
    }
    Ok(())
}

async fn mouse_move_loop(input: Input, train_set: GlRailway, shutdown: OneShot, stage: Arc<Mutex<Stage>>) {
    let input2 = input.clone();
    shutdown.add(move || {
        input2.create_fake_mouse_move();
    });
    let mut mouse_position: Option<(f64,f64)> = None;
    loop {
        let read_stage = stage.lock().unwrap().read_stage();
        mouse_move_tick(&input,&mut mouse_position,&read_stage,&train_set);
        input.wait_for_mouse_move().await;
        if shutdown.poll() { break; }
        cdr_tick(1).await;
    }
    log_extra!("mouse move loop finished");
}

pub(crate) fn run_mouse_move(web: &mut PeregrineInnerAPI, dom: &PeregrineDom) -> Result<(),Message> {
    let shutdown = dom.shutdown().clone();
    let mut other = web.clone();
    web.commander().add_task("mouse-move-animator",0,None,None,Box::pin(async move {
        let lweb = other.lock().await;
        let input = lweb.input.clone();
        let train_set = lweb.trainset.clone();
        let stage = lweb.stage.clone();
        drop(lweb);
        mouse_move_loop(input,train_set,shutdown,stage).await;
        Ok(())
    }));
    Ok(())
}
