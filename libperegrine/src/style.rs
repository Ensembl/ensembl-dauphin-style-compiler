use std::sync::{Arc, Mutex};
use eard_interp::{GlobalContext, GlobalBuildContext, Return};
use peregrine_data::ProgramShapesBuilder;
use peregrine_toolkit::lock;

pub(crate) fn op_style(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<Arc<Mutex<Option<ProgramShapesBuilder>>>>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let path = ctx.force_string(regs[0])?.to_string();
        let key = ctx.force_finite_string(regs[1])?;
        let value = ctx.force_finite_string(regs[2])?;
        if key.len() != value.len() {
            return Err(format!("keys and values different length in style command"));
        }
        let kvs = key.iter().zip(value.iter()).map(|(k,v)| {
            (k.to_string(),v.to_string())
        }).collect();
        let shapes = ctx.context.get_mut(&shapes);
        let mut shapes = lock!(shapes);
        shapes.as_mut().unwrap().add_style(&path,kvs);
        Ok(Return::Sync)
    }))
}