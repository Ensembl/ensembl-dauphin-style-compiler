use std::{collections::{BTreeMap, BTreeSet}, future::Ready, sync::{ Arc, Mutex }};
use std::collections::HashMap;
use crate::{PeregrineDom, run::PgPeregrineConfig};
use crate::util::Message;
use crate::run::{ CursorCircumstance, PgConfigKey };
use crate::util::error::confused_browser;

/* handles help avoid nested stuff causing chaos */
pub struct CursorHandle(Arc<Mutex<CursorState>>,pub usize);

impl Drop for CursorHandle {
    fn drop(&mut self) {
        self.0.lock().unwrap().free(self.1);
    }
}

struct CursorState {
    default: CursorCircumstance,
    callback: Box<dyn Fn(&CursorCircumstance)>,
    next_handle: usize,
    handle_values: BTreeMap<usize,CursorCircumstance>
}

impl CursorState {
    fn new<F>(cb: F, default: CursorCircumstance) -> CursorState where F: Fn(&CursorCircumstance) + 'static {
        CursorState {
            default,
            callback: Box::new(cb),
            next_handle: 0,
            handle_values: BTreeMap::new()
        }
    }

    fn update(&self) {
        (self.callback)(self.handle_values.iter().next_back().map(|x| x.1).unwrap_or(&self.default));
    }

    fn allocate(&mut self, circ: &CursorCircumstance) -> usize {
        let index = self.next_handle;
        self.next_handle += 1;
        self.handle_values.insert(index,circ.clone());
        self.update();
        index
    }

    fn free(&mut self, handle: usize) {
        self.handle_values.remove(&handle);
        self.update();
    }
}

#[derive(Clone)]
pub(crate) struct Cursor {
    state: Arc<Mutex<CursorState>>
}

impl Cursor {
    pub fn new(dom: &PeregrineDom, config: &PgPeregrineConfig) -> Result<Cursor,Message> {
        let mut configs = HashMap::new();
        for circ in CursorCircumstance::each() {
            let value = config.try_get_str(&PgConfigKey::Cursor(circ.clone()))
                .map(|x| Ok(x))
                .unwrap_or_else(|| {
                    config.get_str(&PgConfigKey::Cursor(CursorCircumstance::Default))
                })?;
            let values = value.split_whitespace().map(|x| x.to_string()).collect::<Vec<_>>();
            configs.insert(circ,values);
        }
        let el = dom.canvas_frame().clone();        
        Ok(Cursor {
            state: Arc::new(Mutex::new(CursorState::new(move |circ| {
                let values = configs.get(&circ).unwrap(); // XXX report error
                for value in values {
                    confused_browser(el.style().set_property("cursor",value)).ok(); // XXX report error
                }
            }, CursorCircumstance::Default))),
        })
    }

    pub fn set(&self, circ: &CursorCircumstance) -> CursorHandle {
        let index = self.state.lock().unwrap().allocate(circ);
        CursorHandle(self.state.clone(),index)
    }
}
