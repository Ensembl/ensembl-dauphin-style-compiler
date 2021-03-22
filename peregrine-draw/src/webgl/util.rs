use anyhow::{ anyhow as err };
use web_sys::{ WebGlRenderingContext };
use crate::util::message::Message;

pub(crate) fn handle_context_errors(context: &WebGlRenderingContext) -> Result<(),Message> {
    let mut errors = vec![];
    loop {
        let err = context.get_error();
        if err == WebGlRenderingContext::NO_ERROR { break; }
        errors.push(err);
    }
    if errors.len() > 0 {
        Err(Message::XXXTmp(format!("webgl errors: {}",errors.iter().map(|x| format!("{}",x)).collect::<Vec<_>>().join(","))))
    } else {
        Ok(())
    }
}