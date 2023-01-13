use eachorevery::eoestruct::{StructValue, StructConst};
use eard_interp::{GlobalContext, ContextItem, Return, GlobalBuildContext, Value};
use peregrine_data::ShapeRequest;

fn to_const(value: &StructValue) -> Option<StructConst> {
    match value {
        StructValue::Const(c) => Some(c.clone()),
        _ => None
    }
}

fn value_to_atom(value: &StructValue, contents: &[String]) -> Result<Vec<StructConst>,String> {
    let contents = contents.iter().map(|x| x.as_str()).collect::<Vec<_>>();
    Ok(match value.extract(&contents).ok() {
        Some(StructValue::Const(c)) => vec![c],
        Some(StructValue::Array(a)) => a.iter().filter_map(|x| to_const(x)).collect(),
        Some(StructValue::Object(obj)) => obj.keys().map(|x| StructConst::String(x.clone())).collect(),
        None => vec![]
    })
}

fn setting_value(r1: usize, r2: usize, ctx: &mut GlobalContext, request: &ContextItem<ShapeRequest>, is_null_test: bool) -> Result<Vec<StructConst>,String> {
    let key = ctx.force_string(r1)?;
    let path = ctx.force_finite_string(r2)?.to_vec();
    let request = ctx.context.get(&request);
    let config = request.track();
    Ok(if let Some(value) = config.value(&key) {
        value_to_atom(value,&path)?
    } else if is_null_test && path.len() == 0 {
        vec![StructConst::Null]
    } else {
        vec![]
    })
}

fn to_number(value: StructConst) -> f64 {
    match value {
        StructConst::String(s) => s.parse::<f64>().ok().unwrap_or(0.),
        StructConst::Number(n) => n,
        StructConst::Boolean(b) => if b { 1. } else { 0. },
        StructConst::Null => 0.
    }
}

fn to_string(value: StructConst) -> String {
    match value {
        StructConst::String(s) => s,
        StructConst::Number(n) => n.to_string(),
        StructConst::Boolean(b) => if b { "true" } else { "false" }.to_string(),
        StructConst::Null => "".to_string()
    }
}

pub(crate) fn op_setting_boolean_seq(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shape_request = gctx.patterns.lookup::<ShapeRequest>("shape-request")?;
    Ok(Box::new(move |ctx,regs| {
        let value = setting_value(regs[1],regs[2],ctx,&shape_request,false)?;
        let value = value.iter().map(|x| x.truthy()).collect();
        ctx.set(regs[0],Value::FiniteBoolean(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_number_seq(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shape_request = gctx.patterns.lookup::<ShapeRequest>("shape-request")?;
    Ok(Box::new(move |ctx,regs| {
        let mut value = setting_value(regs[1],regs[2],ctx,&shape_request,false)?;
        let value = value.drain(..).map(|x| to_number(x)).collect();
        ctx.set(regs[0],Value::FiniteNumber(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_string_seq(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shape_request = gctx.patterns.lookup::<ShapeRequest>("shape-request")?;
    Ok(Box::new(move |ctx,regs| {
        let mut value = setting_value(regs[1],regs[2],ctx,&shape_request,false)?;
        let value = value.drain(..).map(|x| to_string(x)).collect();
        ctx.set(regs[0],Value::FiniteString(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_boolean(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shape_request = gctx.patterns.lookup::<ShapeRequest>("shape-request")?;
    Ok(Box::new(move |ctx,regs| {
        let value = setting_value(regs[1],regs[2],ctx,&shape_request,false)?;
        let value = value.iter().map(|x| x.truthy()).any(|x| x);
        ctx.set(regs[0],Value::Boolean(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_number(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shape_request = gctx.patterns.lookup::<ShapeRequest>("shape-request")?;
    Ok(Box::new(move |ctx,regs| {
        let mut value = setting_value(regs[1],regs[2],ctx,&shape_request,false)?;
        let value = value.drain(..).map(|x| to_number(x)).reduce(f64::max).unwrap_or(0.);
        ctx.set(regs[0],Value::Number(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_string(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shape_request = gctx.patterns.lookup::<ShapeRequest>("shape-request")?;
    Ok(Box::new(move |ctx,regs| {
        let mut value = setting_value(regs[1],regs[2],ctx,&shape_request,false)?;
        let value = value.drain(..).map(|x| to_string(x)).collect::<Vec<_>>();
        ctx.set(regs[0],Value::String(value.join("\n")))?;
        Ok(Return::Sync)
    }))
}
