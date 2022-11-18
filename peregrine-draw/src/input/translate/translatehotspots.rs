use commander::CommanderStream;
use peregrine_toolkit::{debug_log, lock, log};

use crate::{Message, PeregrineInnerAPI, PgCommanderWeb, input::{InputEvent, InputEventKind, low::lowlevel::LowLevelInput}, run::inner::LockedPeregrineInnerAPI, shape::layers::drawingzmenus::HotspotEntryDetails};

fn process_hotspot_event(api: &LockedPeregrineInnerAPI, x: f64, doc_y: f64) -> Result<(),Message> {
    let events = api.trainset.get_hotspot(&api.stage.lock().unwrap().read_stage(), (x,doc_y))?;
    let mut zmenus = vec![];
    for event in &events {
        match event {
            HotspotEntryDetails::ZMenu(z) => {
                zmenus.push(z.value());
            },
            HotspotEntryDetails::Setting(value) => {
                let (path,value) = value.value();
                debug_log!("setting {:?} gets {:?}",path,value);
                let path = path.iter().map(|x| x.as_str()).collect::<Vec<_>>();
                api.data_api.update_switch(&path,value);
            },
            HotspotEntryDetails::Special(value) => {
                let value = value.value();
                log!("special zone {}",value);
            }
        }
    }
    if zmenus.len() > 0 {
        api.report.zmenu_event(x,doc_y,zmenus);
    }
    Ok(())
}

fn process_event(messages: &CommanderStream<Option<(f64,f64)>>, event: &InputEvent, window_y: f64) -> Result<(),Message> {
    match event.details {
        InputEventKind::ZMenu => {
            let x = *event.amount.get(0).unwrap_or(&0.);
            let y = *event.amount.get(1).unwrap_or(&0.);
            messages.add(Some((x,y+window_y)));
        },
        _ => {}
    }
    Ok(())
}

async fn hotspots_loop(inner_api: &mut PeregrineInnerAPI, messages: CommanderStream<Option<(f64,f64)>>) -> Result<(),Message> {
    let messages2 = messages.clone();
    inner_api.lock().await.dom.shutdown().add(move || {
        messages2.add(None);
    });
    while let Some(message) = messages.get().await {
        process_hotspot_event(&inner_api.lock().await,message.0,message.1)?;
    }
    Ok(())
}

pub(crate) fn translate_hotspots(low_level: &mut LowLevelInput, commander: &PgCommanderWeb, inner_api: &PeregrineInnerAPI) {
    let api2 = inner_api.clone();
    let messages = CommanderStream::new();
    let messages2 = messages.clone();
    low_level.distributor_mut().add(move |e| {
        let window_pos = lock!(api2.stage()).read_stage().y().position().unwrap_or(0.);
        process_event(&messages,e,window_pos).ok();
    }); // XXX error distribution
    let mut inner2 = inner_api.clone();
    commander.add("hotspot", 0, None, None, Box::pin(async move { hotspots_loop(&mut inner2,messages2).await }));
}
