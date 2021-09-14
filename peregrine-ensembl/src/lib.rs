use std::collections::HashMap;
use std::fmt::Debug;
mod standalonedom;
use js_sys::Reflect;
use wasm_bindgen::prelude::*;
use peregrine_draw::{ PeregrineAPI, Message, PgCommanderWeb, PeregrineConfig };
use peregrine_data::{Channel, ChannelLocation, StickId };
use peregrine_toolkit::url::Url;
use crate::standalonedom::make_dom;
use web_sys::{ console };
use std::sync::{ Arc, Mutex };
use std::thread_local;

thread_local!{
    pub static CLOSURE : Arc<Mutex<Vec<Option<js_sys::Function>>>> = Arc::new(Mutex::new(vec![]));
}

/*
 * This utility just catches handles serious errors in setting up this demonstration. It's not the main error-handling
 * code.
 */
pub fn js_throw<T,E: Debug>(e: Result<T,E>) -> T {
    match e {
        Ok(e) => e,
        Err(e) => {
            console::error_1(&format!("{:?}",e).into());
            panic!("deliberate panic from js_throw following error. Ignore this trace, see error above.");
        }
    }
}

fn jserror_to_message<T>(e: Result<T,JsValue>) -> Result<T,Message> {
    e.map_err(|f| Message::ConfusedWebBrowser(format!("bad config parameter: {}",f.as_string().unwrap_or_else(|| "*anon*".to_string()))))
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct GenomeBrowser {
    api: PeregrineAPI,
    commander: Option<PgCommanderWeb>,
    closure_index: Option<usize>
}

/* not available to javascript */
impl GenomeBrowser {
    pub fn real_api(&self) -> &PeregrineAPI { &self.api }
    pub fn commander(&self) -> Option<&PgCommanderWeb> { self.commander.as_ref() }
}

#[wasm_bindgen]
impl GenomeBrowser {

    #[wasm_bindgen(constructor)]
    pub fn new() -> GenomeBrowser {
        GenomeBrowser  {
            api: PeregrineAPI::new(),
            commander: None,
            closure_index: None
        }
    }

    fn build_config(&self, config_object: &JsValue) -> Result<HashMap<String,String>,Message> {
        let mut out = HashMap::new();
        for key in jserror_to_message(Reflect::own_keys(config_object))?.iter() {
            let value = jserror_to_message(Reflect::get(config_object,&key))?;
            let key_str = key.as_string().ok_or_else(|| Message::ConfusedWebBrowser("bad key".to_string()))?;
            let value_str = value.as_string().ok_or_else(|| Message::ConfusedWebBrowser("bad value".to_string()))?;
            out.insert(key_str,value_str);
        }
        Ok(out)
    }

    fn go_real(&mut self, config_object: &JsValue) -> Result<(),Message> {
        let config_in = self.build_config(config_object)?;
        /*
         * Set some config options. There aren't many of these yet but there will probably be more. The idea is that
         * things like the speed of changes and positions, colours, cache sizes, etc, can come from the surrounding app.
         * This isn't about the data in the tracks itself but there's probably going to be a whole load of configurable
         * oojimaflips associated with the browser in the end.
         */
         let mut config = PeregrineConfig::new();
         for (k,v) in config_in.iter() {
             config.set(k,v)?;
         }
        /*
         * Here we call standalonedom.rs which sorts out finding an element and setting it up for the genome browser to
         * use. See that file for details.
         */
        let dom = make_dom()?;
        /*
         * Create a genome browser object.
         */
        self.commander = Some(self.api.run(config,dom)?);
        /*
         * Ok, we're ready to go. Bootstrapping causes the genome browser to go to the backend and configure itself.
         */
        let url = config_in.get("backend_url").unwrap().as_str();
        let mut p = self.api.bootstrap(&Channel::new(&ChannelLocation::HttpChannel(js_throw(Url::parse(url)))));
        /*
         * You have to turn on tracks _per se_, but we always want tracks.
         */
        self.api.set_switch(&["track"]);
        self.api.set_switch(&["focus"]);
        self.api.set_switch(&["settings"]);
        self.api.radio_switch(&["focus"],true);
        self.api.radio_switch(&["focus","gene"],true);
        Ok(())
    }

    pub fn go(&mut self, config_object: &JsValue) {
        js_throw(self.go_real(config_object));
    }

    pub fn set_stick(&self,stick_id: &str) {
        self.api.set_stick(&StickId::new(&stick_id));
    }

    pub fn jump(&self,location: &str) {
        self.api.jump(location);
    }

    /*
    * Receive message
    */
    pub fn receive_message(message: &JsValue) {
        console::log_1(&format!("{:?}",message).into());
    }
    
    pub fn goto(&self, left: f64, right: f64) {
        self.api.goto(left,right);
    }

    pub fn set_y(&self,y: f64) {
        self.api.set_y(y);
    }
    
    pub fn set_switch(&self, path: &JsValue) {
        let path : Vec<String> = path.into_serde().unwrap();
        self.api.set_switch(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>());
    }

    pub fn clear_switch(&self, path: &JsValue) {
        let path : Vec<String> = path.into_serde().unwrap();
        self.api.clear_switch(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>());
    }

    /* called first time set_message_reporter is called for each object */
    fn first_set_message_reporter(&mut self, closure: &Arc<Mutex<Vec<Option<js_sys::Function>>>>) {
        let mut closure = closure.lock().unwrap();
        let index = closure.len();
        closure.push(None);
        self.closure_index = Some(index);
        let index2 = index;
        self.api.set_message_reporter(Box::new(move |message| {
            CLOSURE.with(|closure| {
                if let Some(closure) = &closure.lock().unwrap()[index2] {
                    let this = JsValue::null(); 
                    let x = JsValue::from(message.to_string().as_str());
                    let _ = closure.call1(&this, &x);    
                }      
            });
        }));
    }

    pub fn set_message_reporter(&mut self,f: js_sys::Function) {
        CLOSURE.with(move |closure| {
            if self.closure_index.is_none() {
                self.first_set_message_reporter(closure);
            }
            let index = self.closure_index.unwrap();
            closure.lock().unwrap()[index].replace(f);
        });
    }
}

/*
 * This is the code which starts it all.
 */
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    Ok(())
}

/*
 * This is an obscure thing which makes stack traces better.
 */
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
