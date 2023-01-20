use std::collections::HashSet;

use commander::CommanderStream;
use eachorevery::eoestruct::StructValue;
use peregrine_data::{HotspotResult, DataMessage};
use peregrine_toolkit::{debug_log, lock, log, error::Error};
use crate::{Message, PeregrineInnerAPI, PgCommanderWeb, input::{InputEvent, InputEventKind, low::lowlevel::LowLevelInput}, run::inner::LockedPeregrineInnerAPI};

fn process_hotspot_event(api: &LockedPeregrineInnerAPI, x: f64, doc_y: f64) -> Result<(),Message> {
    let events = api.trainset.get_hotspot(&api.stage.lock().unwrap().read_stage(), (x,doc_y))?;
    let mut hotspot_contents = HashSet::new();
    let mut hotspot_varieties = HashSet::new();
    for event in &events {
        if let Some(event) = event.value() {
            match event {
                HotspotResult::Click(variety,_) => {
                    hotspot_varieties.insert(variety);
                },
                _ => {}
            }
        }
    }
    for event in &events {
        if let Some(event) = event.value() {
            match event {
                HotspotResult::Setting(path,value) => {
                    debug_log!("setting {:?} gets {:?}",path,value);
                    let path = path.iter().map(|x| x.as_str()).collect::<Vec<_>>();
                    api.data_api.update_switch(&path,value);
                },
                HotspotResult::Special(_) => {},
                HotspotResult::Click(_,contents) => {
                    hotspot_contents.insert(contents);
                },
                HotspotResult::Setting2(switch,path,value) => {
                    let switch = switch.iter().map(|x| x.as_str()).collect::<Vec<_>>();
                    api.data_api.update_setting(&switch,&path,StructValue::new_expand(&value,None).map_err(|e| {
                        Message::DataError(DataMessage::XXXTransitional(Error::operr(&e)))
                    })?);
                }
            }
        }
    }
    if hotspot_contents.len() > 0 || hotspot_varieties.len() > 0 {
        api.report.hotspot_event(x,doc_y,&hotspot_varieties.iter().cloned().collect::<Vec<_>>(),&hotspot_contents.drain().collect::<Vec<_>>());
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
