use super::inner::{ PeregrineInnerAPI, LockedPeregrineInnerAPI };
use super::size::{SizeManager, start_size_manager};
use commander::{ cdr_tick, cdr_current_time };
use peregrine_data::{Commander, Assets, PeregrineCoreBase};
use peregrine_toolkit::lock;
use crate::input::Input;
use crate::stage::stage::ReadStage;
use crate::util::message::Message;
use crate::webgl::DrawingSession;
use super::dom::PeregrineDom;

fn draw_objects_and_spectres(lweb: &mut LockedPeregrineInnerAPI, read_stage: &ReadStage, elapsed: f64) -> Result<(),Message> {
    if read_stage.ready() {
        let gl = lweb.webgl.clone();
        lweb.trainset.transition_animate_tick(&lweb.data_api,&mut *lock!(gl),elapsed)?;
        let assets = lweb.assets.clone();
        let mut session = DrawingSession::new(lweb.trainset.scale());
        session.begin(&mut *lock!(gl))?;
        lweb.trainset.draw_animate_tick(read_stage,&gl,&mut session)?;
        lweb.spectre_manager.draw(&gl,&assets,read_stage,&mut session)?;
        session.finish(lweb.data_api)?;
    }
    Ok(())
}

fn tick_again_if_spectres(lweb: &mut LockedPeregrineInnerAPI, input: &Input) {
    let spectres = input.get_spectres();
    if spectres.len() > 0 {
        lock!(lweb.stage).redraw_needed().set();
    }
}

fn animation_tick(lweb: &mut LockedPeregrineInnerAPI, input: &Input, elapsed: f64) -> Result<(),Message> {
    let read_stage = &lock!(lweb.stage).read_stage();
    input.update_stage(read_stage);
    tick_again_if_spectres(lweb,input);
    draw_objects_and_spectres(lweb,read_stage,elapsed)?;
    Ok(())
}

async fn animation_tick_loop(mut web: PeregrineInnerAPI, input: Input) {
    let mut start = cdr_current_time();
    let lweb = web.lock().await;
    let redraw = lock!(lweb.stage).redraw_needed().clone();
    drop(lweb);
    loop {
        let next = cdr_current_time();
        let r = animation_tick(&mut web.lock().await,&input,next-start);
        if let Err(e) = r { 
            web.lock().await.message_sender.add(e);
        }
        cdr_tick(1).await;
        redraw.wait_until_needed().await;
        start = next;
    }
}

pub fn run_animations(web: &mut PeregrineInnerAPI, dom: &PeregrineDom) -> Result<(),Message> {
    let mut other = web.clone();
    let dom = dom.clone();
    web.commander().add_task("animator",0,None,None,Box::pin(async move {
        // TODO factor this pattern
        let lweb = other.lock().await;
        let input = lweb.input.clone();
        drop(lweb);
        animation_tick_loop(other,input).await;
        Ok(())
    }));
    start_size_manager(web,&dom);
    Ok(())
}
