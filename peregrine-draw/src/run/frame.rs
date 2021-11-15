use super::inner::{ PeregrineInnerAPI, LockedPeregrineInnerAPI };
use super::size::SizeManager;
use commander::{ cdr_tick, cdr_current_time };
use peregrine_data::Commander;
use crate::input::Input;
use crate::util::message::Message;
use crate::webgl::DrawingSession;
use super::dom::PeregrineDom;

fn animation_tick(web: &mut LockedPeregrineInnerAPI, size_manager: &SizeManager, input: &Input, elapsed: f64) -> Result<(),Message> {
    size_manager.tick(web)?;
    let read_stage = &web.stage.lock().unwrap().read_stage();
    input.update_stage(read_stage);
    let spectres = input.get_spectres();
    if spectres.len() > 0 {
        web.stage.lock().unwrap().redraw_needed().set();
    }
    let gl = &mut web.webgl.lock().unwrap();
    let assets = web.assets.clone();
    web.trainset.transition_animate_tick(&web.data_api,gl,elapsed)?;
    if read_stage.ready() {
        let mut session = DrawingSession::new(web.trainset.scale());
        session.begin(gl)?;
        web.trainset.draw_animate_tick(read_stage,gl,&mut session)?;
        web.spectre_manager.draw(gl,&assets,read_stage,&mut session)?;
        session.finish(web.data_api)?;
    }
    Ok(())
}

async fn animation_tick_loop(mut web: PeregrineInnerAPI, size_manager: SizeManager, input: Input) {
    let mut start = cdr_current_time();
    let lweb = web.lock().await;
    let redraw = lweb.stage.lock().unwrap().redraw_needed().clone();
    drop(lweb);
    loop {
        let next = cdr_current_time();
        let mut lweb = web.lock().await;
        let r = animation_tick(&mut lweb,&size_manager,&input,next-start);
        if let Err(e) = r { 
            lweb.message_sender.add(e);
        }
        drop(lweb);
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
        let dom = dom.clone();    
        let message_sender = lweb.message_sender.clone();
        drop(lweb);
        let size_manager = SizeManager::new(&mut other,&dom).await;
        match size_manager {
            Ok(size_manager) => {
                animation_tick_loop(other,size_manager,input).await;
            },
            Err(e) => {
                message_sender.add(e);
            }
        }
        Ok(())
    }));
    Ok(())
}
