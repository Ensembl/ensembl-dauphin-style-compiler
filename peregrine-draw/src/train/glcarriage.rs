use peregrine_data::{Assets, DrawingCarriage, CarriageExtent};
use peregrine_toolkit::{lock, warn, error };
use peregrine_toolkit_async::sync::asynconce::AsyncOnce;
use peregrine_toolkit_async::sync::needed::Needed;
use crate::shape::layers::drawingzmenus::HotspotEntryDetails;
use crate::{PgCommanderWeb};
use crate::shape::layers::drawing::{ Drawing };
use crate::webgl::DrawingSession;
use crate::webgl::global::WebGlGlobal;
use std::sync::{Arc, Mutex};
use crate::stage::stage::ReadStage;
use crate::util::message::Message;

struct GLCarriageData {
    commander: PgCommanderWeb,
    extent: CarriageExtent,
    opacity: Mutex<f64>,
    drawing: AsyncOnce<Result<Option<Drawing>,Message>>,
    preflight_done: bool,
    discarded: bool
}

fn get_drawing(data: &GLCarriageData) -> Result<Option<Drawing>,Message> {
    let current = data.drawing.peek();
    let result = if let Some(x) = current { x } else { return Ok(None); };
    let drawing = match result {
        Ok(x) => x,
        Err(e) => { return Err(e); }
    };
    Ok(drawing)
}

impl GLCarriageData {
    fn in_view(&self, stage: &ReadStage) -> Result<bool,Message> {
        let stage = stage.x().left_right()?;
        let carriage = self.extent.left_right();
        Ok(!(stage.0 > carriage.1 || stage.1 < carriage.0))
    }
}

#[derive(Clone)]
pub(crate) struct GLCarriage(Arc<Mutex<GLCarriageData>>);

impl GLCarriage {
    pub fn new(redraw_needed: &Needed, commander: &PgCommanderWeb, carriage: &DrawingCarriage, gl: &Arc<Mutex<WebGlGlobal>>, assets: &Assets) -> Result<GLCarriage,Message> {
        let carriage2 = carriage.clone();
        let gl = gl.clone();
        let assets = assets.clone();
        let redraw_needed = redraw_needed.clone();
        let our_carriage = GLCarriage(Arc::new(Mutex::new(GLCarriageData {
            commander: commander.clone(),
            extent: carriage.extent().clone(),
            opacity: Mutex::new(1.),
            preflight_done: false,
            discarded: false,
            drawing: AsyncOnce::new(async move {
                let carriage = carriage2;
                let scale = carriage.extent().scale();
                let shapes = carriage.shapes().clone();
                let drawing = Drawing::new(Some(scale),shapes,&gl,carriage.extent().left_right().0,&assets,&carriage.relevancy()).await;
                redraw_needed.set();
                drawing
            })
        })));
        our_carriage.preflight_freewheel(carriage);
        Ok(our_carriage)
    }

    pub(super) async fn preflight(&self, carriage: &DrawingCarriage) -> Result<(),Message> {
        let state = lock!(self.0);
        let drawing = state.drawing.clone();
        drop(state);
        let g = drawing.get().await;
        let x = g.as_ref().map(|_| ()).map_err(|e| e.clone());
        if let Err(e) = x {
            error!("{}",e);
        }
        lock!(self.0).preflight_done = true;
        carriage.set_ready();
        Ok(())
    }

    pub fn preflight_freewheel(&self, carriage: &DrawingCarriage) {
        let self2 = self.clone();
        let commander = lock!(self.0).commander.clone();
        let carriage = carriage.clone();
        commander.add::<Message>("load", 2, None, None, Box::pin(async move {
            self2.preflight(&carriage).await
        }));
    }

    pub fn extent(&self) -> CarriageExtent { lock!(self.0).extent.clone() }

    pub(super) fn set_opacity(&self, amount: f64) {
        *lock!(self.0).opacity.lock().unwrap() = amount;
    }

    pub fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &mut DrawingSession) -> Result<(),Message> {
        let state = lock!(self.0);
        if !state.preflight_done {
            warn!("draw without preflight");
        }
        if state.discarded {
            panic!("draw on discarded glcarriage");
        }
        let opacity = state.opacity.lock().unwrap().clone();
        let in_view =  state.in_view(stage)?;
        if let Some(mut drawing) = get_drawing(&state)? {
            drawing.set_zmenu_px_per_screen(stage.x().drawable_size()?);
            if in_view {
                drawing.draw(gl,stage,session,opacity)?;
            }
        }
        Ok(())
    }

    pub(crate) fn get_hotspot(&self, stage: &ReadStage, position: (f64,f64)) -> Result<Vec<HotspotEntryDetails>,Message> {
        let state = lock!(self.0);
        if let Some(drawing) = get_drawing(&state)? {
            drawing.get_hotspot(stage,position)
        } else {
            Ok(vec![])
        }
    }

    pub fn discard(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        let mut state = lock!(self.0);
        state.discarded = true;
        if let Some(mut drawing) = get_drawing(&state)? {
            drawing.discard(gl)?;
        }
        Ok(())
    }
}
