use crate::util::message::Message;
use wasm_bindgen::JsValue;

pub(crate) fn confused_browser<R,>(result: Result<R,JsValue>) -> Result<R,Message> {
    result.map_err(|e| Message::ConfusedWebBrowser(e.as_string().unwrap_or_else(|| format!("anonymous object"))))
}
