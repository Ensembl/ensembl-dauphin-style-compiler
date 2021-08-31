use commander::CommanderStream;

use crate::{Message, PeregrineInnerAPI, PgCommanderWeb, input::{InputEvent, InputEventKind, low::lowlevel::LowLevelInput}, run::inner::LockedPeregrineInnerAPI, train::GlTrainSet};

fn process_zmenu_event(api: &LockedPeregrineInnerAPI, x: f64, y: f64) -> Result<(),Message> {
    let mut gl = api.webgl.lock().unwrap();
    let zmenus = api.trainset.get_hotspot(&mut gl,&api.stage.lock().unwrap().read_stage(), (x,y))?;
    let zmenus = zmenus.iter().map(|z| z.value()).collect::<Vec<_>>();
    use web_sys::console;
    console::log_1(&format!("{:?}",zmenus).into());
    Ok(())
}

fn process_event(messages: &CommanderStream<(f64,f64)>, event: &InputEvent) -> Result<(),Message> {
    match event.details {
        InputEventKind::ZMenu => {
            let x = *event.amount.get(0).unwrap_or(&0.);
            let y = *event.amount.get(1).unwrap_or(&0.);
            messages.add((x,y));
        },
        _ => {}
    }
    Ok(())
}

async fn zmenus_loop(inner_api: &mut PeregrineInnerAPI, messages: CommanderStream<(f64,f64)>) -> Result<(),Message> {
    loop {
        let message = messages.get().await;
        process_zmenu_event(&inner_api.lock().await,message.0,message.1)?;
    }
}

pub(crate) fn translate_zemnus(low_level: &mut LowLevelInput, commander: &PgCommanderWeb, inner_api: &PeregrineInnerAPI) {
    let messages = CommanderStream::new();
    let messages2 = messages.clone();
    low_level.distributor_mut().add(move |e| { process_event(&messages,e).ok(); }); // XXX error distribution
    let mut inner2 = inner_api.clone();
    commander.add("zmenu", 0, None, None, Box::pin(async move { zmenus_loop(&mut inner2,messages2).await }));
}
