use peregrine_toolkit::{error::Error};
use crate::{MaxiResponse, PeregrineApiQueue, Switches };

pub(crate) fn add_tracks_from_response(response: &MaxiResponse, switches: &Switches, queue: &PeregrineApiQueue) -> Result<(),Error> {
    let (tracks,expansions) = response.tracks().to_track_models()?;
    for track in &tracks {
        switches.add_track_model(track)?;
    }
    for expansion in &expansions {
        switches.add_expansion_model(expansion);
    }
    queue.regenerate_track_config();
    Ok(())
}
