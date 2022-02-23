use peregrine_data::{Assets, Carriage, CarriageExtent, ZMenuProxy};
use peregrine_toolkit::lock;
use peregrine_toolkit::sync::asynconce::AsyncOnce;
use peregrine_toolkit::sync::needed::Needed;
use crate::{PgCommanderWeb};
use crate::shape::layers::drawing::{ Drawing };
use crate::webgl::DrawingSession;
use crate::webgl::global::WebGlGlobal;
use std::hash::{ Hash, Hasher };
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use crate::stage::stage::ReadStage;
use crate::util::message::Message;

struct GLCarriageData {
    commander: PgCommanderWeb,
    extent: CarriageExtent,
    opacity: Mutex<f64>,
    drawing: AsyncOnce<Result<Drawing,Message>>
}

fn get_drawing(data: &GLCarriageData) -> Result<Option<Drawing>,Message> {
    data.drawing.peek().map(|x| x.map(|x| Some(x))).unwrap_or(Ok(None))
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

impl PartialEq for GLCarriage {
    fn eq(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.0,&other.0) { return true; }
        lock!(self.0).extent == lock!(other.0).extent
    }
}

impl Eq for GLCarriage {}

impl Hash for GLCarriage {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        lock!(self.0).extent.hash(hasher);
    }
}

impl GLCarriage {
    pub fn new(redraw_needed: &Needed, commander: &PgCommanderWeb, carriage: &Carriage, gl: &Arc<Mutex<WebGlGlobal>>, assets: &Assets) -> Result<GLCarriage,Message> {
        let carriage2 = carriage.clone();
        let gl = gl.clone();
        let assets = assets.clone();
        let redraw_needed = redraw_needed.clone();
        let our_carriage = GLCarriage(Arc::new(Mutex::new(GLCarriageData {
            commander: commander.clone(),
            extent: carriage.extent().clone(),
            opacity: Mutex::new(1.),
            drawing: AsyncOnce::new(async move {
                let carriage = carriage2;
                let scale = carriage.extent().train().scale();
                let shapes = carriage.shapes().ok().unwrap(); // XXX
                let drawing = Drawing::new(Some(scale),shapes,&gl,carriage.extent().left_right().0,&assets).await;
                carriage.set_ready();
                redraw_needed.set();
                drawing
            })
        })));
        our_carriage.preflight_freewheel(carriage);
        Ok(our_carriage)
    }

    pub(super) async fn preflight(&self, _carriage: &Carriage) -> Result<(),Message> {
        let state = lock!(self.0);
        let drawing = state.drawing.clone();
        drop(state);
        drawing.get().await.as_ref().map(|_| ()).map_err(|e| e.clone())?;
        Ok(())
    }

    pub fn preflight_freewheel(&self, carriage: &Carriage) {
        let self2 = self.clone();
        let commander = lock!(self.0).commander.clone();
        let carriage = carriage.clone();
        commander.add::<Message>("load", 0, None, None, Box::pin(async move {
            self2.preflight(&carriage).await
        }));
    }

    pub fn extent(&self) -> CarriageExtent { lock!(self.0).extent.clone() }

    pub(super) fn set_opacity(&self, amount: f64) {
        *lock!(self.0).opacity.lock().unwrap() = amount;
    }

    pub fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &mut DrawingSession) -> Result<(),Message> {
        let state = lock!(self.0);
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

    pub(crate) fn get_hotspot(&self, stage: &ReadStage, position: (f64,f64)) -> Result<Vec<Rc<ZMenuProxy>>,Message> {
        let state = lock!(self.0);
        if let Some(drawing) = get_drawing(&state)? {
            drawing.get_hotspot(stage,position)
        } else {
            Ok(vec![])
        }
    }

    pub fn discard(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        let state = lock!(self.0);
        if let Some(mut drawing) = get_drawing(&state)? {
            drawing.discard(gl)?;
        }
        Ok(())
    }
}
