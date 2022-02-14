use std::{collections::HashMap, future::Ready, sync::{ Arc, Mutex }};
use std::fmt::Debug;
mod standalonedom;
use js_sys::Reflect;
use js_sys::Array;
use wasm_bindgen::prelude::*;
use anyhow::{ self };
use commander::{Executor, cdr_timer};
use peregrine_draw::{Endstop, Message, PeregrineAPI, PeregrineConfig, PgCommanderWeb};
use peregrine_data::{Channel, ChannelLocation, StickId, zmenu_fixed_vec_to_json, zmenu_to_json };
use peregrine_message::{MessageKind, PeregrineMessage};
use peregrine_toolkit::url::Url;
use crate::standalonedom::make_dom;
use web_sys::{HtmlElement, console };
use serde::{Serialize, Deserialize};
use serde_json::Value as JSONValue;

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

/*
 * This function just does some daft stuff to kick the tyres for now. Don't worry too much about it. But it is quite
 * a good example of the sort of API calls you might make on receiving events from the browser chrome. The exact
 * name and arguments to this API are still up in the air, but you get the idea....
 */

pub async fn test_cdr(api: &PeregrineAPI) -> anyhow::Result<()> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let el = document.get_element_by_id("other").unwrap().dyn_into::<HtmlElement>().ok().unwrap();
    api.set_switch(&["track"]);
    api.set_switch(&["track","gene-pc-fwd"]);
    api.set_switch(&["track","gene-other-fwd"]);
    api.set_stick(&StickId::new("homo_sapiens_GCA_000001405_27:1"));
    let mut left = 2500000.;
    let mut right = 1000000.;
    api.goto(left,right);
    let mut i = 0_f64;
    loop {
            i += 1.;

            left = 2500000. + 400000. * (i/50.).sin();
            right = 2500000. + 400000. * (i/15.).cos();

            api.goto(left,right);
            cdr_timer(1.).await; // Wait one second
    }
    cdr_timer(100.).await;
    el.class_list().add_1("other2");
    Ok(())
}

#[wasm_bindgen]
pub async fn test(api: GenomeBrowser) {
    let pg_api = api.real_api().clone();
    api.commander().unwrap().add("test",100,None,None,Box::pin(async move { test_cdr(&pg_api).await }));
}

#[derive(Serialize, Deserialize)]
struct TrackMetadata {
    summary: Vec<HashMap<String,String>>
}

#[derive(Serialize, Deserialize)]
struct LocationData {
    stick: String,
    start: f64,
    end: f64
}

#[derive(Serialize, Deserialize)]
struct ZmenuData {
    x: f64,
    y: f64,
    content: serde_json::Value
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
        let target_element_id = config_in.get("target_element_id").unwrap().as_str();
        let dom = make_dom(target_element_id)?;
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

    pub fn copy(&self) -> GenomeBrowser {
        self.clone()
    }

    pub fn set_stick(&self,stick_id: &str) {
        self.api.set_stick(&StickId::new(&stick_id));
    }

    pub fn jump(&self,location: &str) {
        self.api.jump(location);
    }

    pub fn wait(&self) {
        self.api.wait();
    }

    pub fn receive_message(message: &JsValue) {
        console::log_1(&format!("received {:?}",message).into());
    }
        
    pub fn set_artificial(&self, name: &str, start: bool) {
        self.api.set_artificial(name,start);
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

    pub fn radio_switch(&self, path: &JsValue, yn: bool) {
        let path : Vec<String> = path.into_serde().unwrap();
        self.api.radio_switch(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>(),yn);
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
                    match message.kind() {
                        MessageKind::Error => {
                            /* func("error",error_as_string) */
                            let kind = JsValue::from("error");
                            let msg = JsValue::from(message.to_string().as_str());
                            let _ = closure.call2(&this,&kind,&msg);            
                        },
                        MessageKind::Interface => {
                            match message {
                                Message::CurrentLocation(stick,start,end) => {
                                    let args = Array::new();
                                    args.set(0,JsValue::from("current_position"));

                                    args.set(1,JsValue::from(js_throw(JsValue::from_serde(&LocationData {
                                        stick: stick.to_string(),
                                        start: *start as f64,
                                        end: *end as f64
                                    }))));

                                    let _ = closure.apply(&this,&args);                    
                                }
                                Message::TargetLocation(stick,start,end) => {
                                    let args = Array::new();
                                    args.set(0,JsValue::from("target_position"));

                                    args.set(1,JsValue::from(js_throw(JsValue::from_serde(&LocationData {
                                        stick: stick.to_string(),
                                        start: *start as f64,
                                        end: *end as f64
                                    }))));
                                    
                                    let _ = closure.apply(&this,&args);                    
                                },
                                Message::Ready => {},
                                Message::AllotmentMetadataReport(metadata) => {
                                    let args = Array::new();
                                    args.set(0,JsValue::from("track_summary"));
                                    args.set(1,JsValue::from(js_throw(JsValue::from_serde(&TrackMetadata {
                                        summary: metadata.summarize().to_vec()
                                    }))));
                                    let _ = closure.apply(&this,&args);
                                },
                                Message::ZMenuEvent(x,y,zmenus) => {
                                    let args = Array::new();
                                    let json = zmenu_to_json(*x,*y,zmenus);
                                    args.set(0,JsValue::from("zmenu"));
                                    args.set(1,js_throw(JsValue::from_serde(&json)));
                                    let _ = closure.apply(&this,&args);
                                },
                                Message::HitEndstop(endstops) => {
                                    let args = Array::new();
                                    let values = Array::new();
                                    for (i,endstop) in endstops.iter().enumerate() {
                                        let name = match endstop {
                                            Endstop::Left => { "left" },
                                            Endstop::Right => { "right" },
                                            Endstop::MaxZoomIn => { "in" },
                                            Endstop::MaxZoomOut => { "out" }
                                        };
                                        values.set(i as u32,JsValue::from(name));
                                    }
                                    args.set(0,JsValue::from("endstops"));
                                    args.set(1,JsValue::from(values));
                                    let _ = closure.apply(&this,&args);
                                }
                                x => {
                                    use web_sys::console;
                                    console::warn_1(&format!("unexpected information: {}",x.to_string()).into());
                                }
                            }
                        }
                    }
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
    use std::panic;

    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    #[cfg(not(debug_assertions))]
    panic::set_hook(Box::new(|_| {}));
}
