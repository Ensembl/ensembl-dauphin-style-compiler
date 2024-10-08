use std::{collections::HashMap, sync::{ Arc, Mutex }};
use std::fmt::Debug;
use eachorevery::eoestruct::StructValue;
use js_sys::Reflect;
use js_sys::{ Array, JSON };
use serde_wasm_bindgen::from_value;
use wasm_bindgen::{prelude::*, JsCast};
use peregrine_draw::{Endstop, Message, PeregrineAPI, PeregrineConfig, PgCommanderWeb};
use peregrine_data::{StickId, DataMessage };
use peregrine_message::{MessageKind, PeregrineMessage};
use peregrine_toolkit::{ warn, log, error, js::jstojsonvalue::js_to_json, error::{ErrorType,CallToAction }};
use web_sys::{ Element };
use serde::{Serialize, Deserialize};
use serde_json::{ Map as JsonMap, Value as JsonValue };

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
            panic!("{:?}",e);
        }
    }
}

fn jserror_to_message<T>(e: Result<T,JsValue>) -> Result<T,Message> {
    e.map_err(|f| Message::ConfusedWebBrowser(format!("bad config parameter: {}",f.as_string().unwrap_or_else(|| "*anon*".to_string()))))
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

enum ConfigValue {
    String(String),
    Element(Element)
}

impl ConfigValue {
    fn new(input: &JsValue) -> Result<ConfigValue,Message> {
        if let Some(element) = input.clone().dyn_into().ok() {
            Ok(ConfigValue::Element(element))
        } else {
            Ok(ConfigValue::String(input.as_string().ok_or_else(|| Message::ConfusedWebBrowser("bad value".to_string()))?))
        }
    }

    fn to_string(&self) -> Result<String,Message> {
        match self {
            ConfigValue::String(x) => Ok(x.to_string()),
            ConfigValue::Element(el) => Ok(el.id())
        }
    }

    fn to_element(&self) -> Result<Element,Message> {
        match self {
            ConfigValue::String(id) => {
                let window = web_sys::window().ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get window")))?;
                let document = window.document().ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get document")))?;
                Ok(document.get_element_by_id(id).ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get element")))?)
            },
            ConfigValue::Element(x) => Ok(x.clone())
        }
    }
}

fn get_or_error<'a,X>(config: &'a HashMap<String,X>, keys: &[&str]) -> Result<&'a X,Message> {
    for key in keys {
        if let Some(value) = config.get(*key) {
            return Ok(value);
        }
    }
    Err(Message::ConfusedWebBrowser(format!("missing keys {}",keys.join(", "))))
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

    fn build_config(&self, config_object: &JsValue) -> Result<HashMap<String,ConfigValue>,Message> {
        let mut out = HashMap::new();
        for key in jserror_to_message(Reflect::own_keys(config_object))?.iter() {
            let value = jserror_to_message(Reflect::get(config_object,&key))?;
            let key_str = key.as_string().ok_or_else(|| Message::ConfusedWebBrowser("bad key".to_string()))?;
            out.insert(key_str,ConfigValue::new(&value)?);
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
        let mut config = PeregrineConfig::new()?;
        for (k,v) in config_in.iter() {
            config.set(k,&v.to_string()?)?;
        }
        /*
         * Here we call standalonedom.rs which sorts out finding an element and setting it up for the genome browser to
         * use. See that file for details.
         */
        let target_element = get_or_error(&config_in,&["target_element","target_element_id"])?;
        /*
         * Create a genome browser object.
         */
        let url = config_in.get("backend_url").unwrap().to_string()?;
        self.commander = Some(self.api.run(config,&target_element.to_element()?,&url)?);
        /*
         * You have to turn on tracks _per se_, but we always want tracks.
         */
        let tmpl_true = StructValue::new_boolean(true);
        self.api.switch(&["track"],tmpl_true.clone());
        self.api.switch(&["focus"],tmpl_true.clone());
        self.api.switch(&["settings"],tmpl_true.clone());
        self.api.radio_switch(&["focus"],true);  // make this a radio switch (only one focus at a time)
        self.api.radio_switch(&["focus","gene"],true);
        self.api.radio_switch(&["track","focus","item"],true);
        self.api.switch(&["track","focus"],tmpl_true.clone());
        self.api.switch(&["track","focus","variant"],tmpl_true.clone());
        self.api.switch(&["track","focus","item"],tmpl_true.clone());
        self.api.switch(&["ruler"],tmpl_true.clone());
        self.api.switch(&["ruler","one_based"],tmpl_true.clone());
        self.api.switch(&["track", "expand"],tmpl_true.clone()); // enable expansion node tracks
        self.api.switch(&["buttons"],tmpl_true.clone());
        self.api.switch(&["buttons","gene"],tmpl_true.clone());
        self.api.switch(&["buttons","gene","expand"],tmpl_true.clone());
        self.api.switch(&["buttons","gene","contract"],tmpl_true.clone());
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
        log!("{:?}",message);
    }
        
    pub fn set_artificial(&self, name: &str, start: bool) {
        self.api.set_artificial(name,start);
    }
    
    pub fn goto(&self, left: f64, right: f64) {
        /* convert ensembl co-ordinates to regular ones */
        self.api.goto(left-1.,right);
    }

    pub fn set_y(&self,y: f64) {
        self.api.set_y(y);
    }
    
    pub fn add_jsapi_channel(&self, name: &str, payload: JsValue) {
        self.api.add_jsapi_channel(name,payload);
    }

    pub fn switch(&self, path: &JsValue, value: &JsValue) {
        let path : Vec<String> = from_value(path.clone()).unwrap();
        if let Ok(json) = js_to_json(value) {
            let value = StructValue::new_json_value(&json);
            self.api.switch(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>(),value);
        }
    }

    pub fn radio_switch(&self, path: &JsValue, yn: bool) {
        let path : Vec<String> = from_value(path.clone()).unwrap();
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
                            let mut skip = false;
                            let mut out_of_date = false;
                            if let Message::DataError(DataMessage::XXXTransitional(x)) = message {
                                if let ErrorType::Temporary = x.error_type {
                                    skip = true;
                                } else if let ErrorType::Unavailable(CallToAction::BadVersion) = x.error_type {
                                    out_of_date = true;
                                }
                            }
                            if !skip {
                                let kind = JsValue::from("error");
                                let msg = match message {
                                    Message::DataError(DataMessage::XXXTransitional(x)) => x.message.clone(),
                                    x => x.to_string()
                                };
                                let msg = JsValue::from(msg);
                                let _ = closure.call2(&this,&kind,&msg);
                            }
                            if out_of_date {
                                error!("FE OUT OF DATE");
                            }
                        },
                        MessageKind::Interface => {
                            match message {
                                Message::CurrentLocation(stick,start,end) => {
                                    let args = Array::new();
                                    args.set(0,JsValue::from("current_position"));

                                    /* "+1."" converts back to ensembl co-ordinates */
                                    args.set(1,JsValue::from(js_throw(JsValue::from_serde(&LocationData {
                                        stick: stick.to_string(),
                                        start: (start+1.).round(),
                                        end: end.round()
                                    }))));

                                    let _ = closure.apply(&this,&args);                    
                                }
                                Message::TargetLocation(stick,start,end) => {
                                    let args = Array::new();
                                    args.set(0,JsValue::from("target_position"));

                                    /* "+1."" converts back to ensembl co-ordinates */
                                    args.set(1,JsValue::from(js_throw(JsValue::from_serde(&LocationData {
                                        stick: stick.to_string(),
                                        start: (start+1.).round(),
                                        end: end.round()
                                    }))));
                                    
                                    let _ = closure.apply(&this,&args);                    
                                },
                                Message::Ready => {},
                                Message::AllotmentMetadataReport(metadata) => {
                                    let args = Array::new();
                                    let mut summary = JsonMap::new();
                                    summary.insert("summary".to_string(),metadata.summarize_json());
                                    let js_summary = JSON::parse(&JsonValue::Object(summary).to_string()).unwrap(); // Yuk!
                                    args.set(0,JsValue::from("track_summary"));
                                    args.set(1,js_summary);
                                    let _ = closure.apply(&this,&args);
                                },
                                Message::HotspotEvent(he) => {
                                    let value = StructValue::new_object(vec![
                                        ("x".to_string(),StructValue::new_number(he.x)),
                                        ("y".to_string(),StructValue::new_number(he.y)),
                                        ("hotspot-area".to_string(),
                                            StructValue::new_object(vec![
                                                ("top".to_string(),StructValue::new_number(he.area.top)),
                                                ("right".to_string(),StructValue::new_number(he.area.right)),
                                                ("bottom".to_string(),StructValue::new_number(he.area.bottom)),
                                                ("left".to_string(),StructValue::new_number(he.area.left))
                                            ])
                                        ),
                                        ("start".to_string(),StructValue::new_boolean(he.start)),
                                        ("variety".to_string(),StructValue::new_array(he.varieties.to_vec())),
                                        ("content".to_string(),StructValue::new_array(he.content.to_vec()))
                                    ]);
                                    let args = Array::new();
                                    args.set(0,JsValue::from("hotspot"));
                                    args.set(1,JsValue::from(js_throw(JsValue::from_serde(&value))));
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
                                    warn!("unexpected information: {}",x.to_string());
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
    #[cfg(not(debug_assertions))]
    use std::panic;

    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    #[cfg(not(debug_assertions))]
    panic::set_hook(Box::new(|_| {}));
}
