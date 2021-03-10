use anyhow::{ anyhow as err };
use web_sys::{ WebGlRenderingContext };

pub(crate) fn handle_context_errors(context: &WebGlRenderingContext) -> anyhow::Result<()> {
    let mut errors = vec![];
    loop {
        let err = context.get_error();
        if err == WebGlRenderingContext::NO_ERROR { break; }
        errors.push(err);
    }
    if errors.len() > 0 {
        Err(err!("webgl errors: {}",errors.iter().map(|x| format!("{}",x)).collect::<Vec<_>>().join(",")))
    } else {
        Ok(())
    }
}