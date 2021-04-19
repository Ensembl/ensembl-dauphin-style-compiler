use super::inner::{ PeregrineInnerAPI, LockedPeregrineInnerAPI };
use super::size::SizeManager;
use commander::{ cdr_tick, cdr_current_time };
use peregrine_data::Commander;
use crate::util::message::Message;
use super::dom::PeregrineDom;

fn animation_tick(web: &mut LockedPeregrineInnerAPI, size_manager: &SizeManager, elapsed: f64) -> Result<(),Message> {
    size_manager.maybe_update_canvas_size(web)?;
    let read_stage = &web.stage.lock().unwrap().read_stage();
    web.trainset.transition_animate_tick(&web.data_api,&mut web.webgl.lock().unwrap(),elapsed)?;
    if read_stage.ready() {
        web.trainset.draw_animate_tick(read_stage,&mut web.webgl.lock().unwrap())?;
    }
    Ok(())
}

async fn animation_tick_loop(mut web: PeregrineInnerAPI, size_manager: SizeManager) {
    let mut start = cdr_current_time();
    let lweb = web.lock().await;
    let redraw = lweb.stage.lock().unwrap().redraw_needed().clone();
    drop(lweb);
    loop {
        let next = cdr_current_time();
        let mut lweb = web.lock().await;
        let r = animation_tick(&mut lweb,&size_manager,next-start);
        if let Err(e) = r { 
            lweb.message_sender.add(e);
        }
        let position =  lweb.stage.lock().unwrap().read_stage().position();
        drop(lweb);
        if let Ok(position) = position {
            web.set_position(position);
        }
        cdr_tick(1).await;
        redraw.wait_until_redraw_needed().await;
        start = next;
    }
}

pub fn run_animations(web: &mut PeregrineInnerAPI, dom: &PeregrineDom) -> Result<(),Message> {
    let mut other = web.clone();
    let dom = dom.clone();
    web.commander().add_task("animator",0,None,None,Box::pin(async move {
        // TODO factor this pattern
        let lweb = other.lock().await;
        let message_sender = lweb.message_sender.clone();
        drop(lweb);
        let size_manager = SizeManager::new(&mut other,&dom).await;
        match size_manager {
            Ok(size_manager) => {
                animation_tick_loop(other,size_manager).await;
            },
            Err(e) => {
                message_sender.add(e);
            }
        }
        Ok(())
    }));
    Ok(())
}
