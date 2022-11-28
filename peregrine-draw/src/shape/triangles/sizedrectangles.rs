use std::sync::{Arc, Mutex};

use peregrine_data::{SpaceBase, SpaceBaseArea, reactive::{Observable, Observer}, LeafStyle};
use peregrine_toolkit::{eachorevery::EachOrEvery, error::Error, lock};

use crate::shape::core::drawshape::GLAttachmentPoint;

use super::rectangles::{RectanglesImpl, apply_wobble};

fn apply_any_wobble<A: Clone>(spacebase: &SpaceBase<f64,A>, wobble: &Option<SpaceBase<Observable<'static,f64>,()>>) -> SpaceBase<f64,A> {
    if let Some(wobble) = wobble {
        apply_wobble(spacebase,&wobble)
    } else {
        spacebase.clone()
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub(super) struct RectanglesLocationSized {
    attachment: GLAttachmentPoint,
    spacebase: SpaceBase<f64,LeafStyle>,
    run: Option<SpaceBase<f64,()>>,
    wobble: Option<SpaceBase<Observable<'static,f64>,()>>,
    wobbled: Arc<Mutex<(SpaceBaseArea<f64,LeafStyle>,Option<SpaceBase<f64,()>>)>>,
    depth: EachOrEvery<i8>,
    size_x: Vec<f64>,
    size_y: Vec<f64>
}

impl RectanglesLocationSized {
    pub(super) fn new(spacebase: &SpaceBase<f64,LeafStyle>, run: &Option<SpaceBase<f64,()>>, wobble: Option<SpaceBase<Observable<'static,f64>,()>>, depth: EachOrEvery<i8>, size_x: Vec<f64>, size_y: Vec<f64>, attachment: GLAttachmentPoint) -> Result<RectanglesLocationSized,Error> {
        let wobbled = (
            attachment.sized_to_rectangle(&apply_any_wobble(&spacebase,&wobble),&size_x,&size_y)?,
            run.as_ref().map(|x| apply_any_wobble(x,&wobble))
        );
        Ok(RectanglesLocationSized { 
            wobbled: Arc::new(Mutex::new(wobbled)),
            spacebase: spacebase.clone(),
            run: run.clone(),
            wobble, depth, size_x, size_y,
            attachment
        })
    }
}

impl RectanglesImpl for RectanglesLocationSized {
    fn depths(&self) -> &EachOrEvery<i8> { &self.depth }
    fn any_dynamic(&self) -> bool { self.wobble.is_some() }

    fn wobbled_location(&self) -> (SpaceBaseArea<f64,LeafStyle>,Option<SpaceBase<f64,()>>) {
        lock!(self.wobbled).clone()
    }

    fn wobble(&mut self) -> Option<Box<dyn FnMut() + 'static>> {
        self.wobble.as_ref().map(|wobble| {
            let wobble = wobble.clone();
            let pos = self.spacebase.clone();
            let wobbled = self.wobbled.clone();
            let size_x = self.size_x.clone();
            let size_y = self.size_y.clone();
            let wobble2 = wobble.clone();
            let spacebase = apply_any_wobble(&pos,&Some(wobble));
            let run = self.run.as_ref().map(|x| apply_any_wobble(x,&Some(wobble2)));
            let attachment = self.attachment.clone();
            Box::new(move || {
                if let Ok(sized) = attachment.sized_to_rectangle(&spacebase,&size_x,&size_y) {
                    *lock!(wobbled) = (sized,run.clone());
                }
            }) as Box<dyn FnMut() + 'static>
        })
    }

    // XXX PartialEq + Hash for collision
    fn watch(&self, observer: &mut Observer<'static>) {
        if let Some(wobble) = &self.wobble {
            for obs in wobble.iter() {
                observer.observe(obs.base);
                observer.observe(obs.normal);
                observer.observe(obs.tangent);
            }
        }
    }    

    fn len(&self) -> usize { self.spacebase.len() }
}
