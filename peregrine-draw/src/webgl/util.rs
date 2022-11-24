use peregrine_data::DataMessage;
use web_sys::{ WebGlRenderingContext };
use crate::util::message::Message;
use peregrine_toolkit::error::{Error};

#[cfg(debug_webgl)]
pub(crate) fn handle_context_errors2(context: &WebGlRenderingContext) -> Result<(),Error> {
    use peregrine_toolkit::error;

    let mut errors = vec![];
    loop {
        let err = context.get_error();
        if err == WebGlRenderingContext::NO_ERROR { break; }
        errors.push(err);
    }
    if errors.len() > 0 {
        error!("context errors: {}",errors.iter().map(|x| format!("{}",x)).collect::<Vec<_>>().join(","));
    }
    Ok(())
}

#[cfg(not(debug_webgl))]
pub(crate) fn handle_context_errors2(_context: &WebGlRenderingContext) -> Result<(),Error> {
   Ok(())
}

pub(crate) fn handle_context_errors(context: &WebGlRenderingContext) -> Result<(),Message> {
    handle_context_errors2(context).map_err(|e| Message::DataError(DataMessage::XXXTransitional(e)) )
}
