use peregrine_toolkit::error::Error;
use crate::{MaxiResponse, PeregrineApiQueue, Switches, PgDauphin};

pub(crate) async fn add_tracks_from_response(response: &MaxiResponse, switches: &Switches, queue: &PeregrineApiQueue, loader: &PgDauphin) -> Result<(),Error> {
    let (tracks,expansions) = response.tracks().to_track_models().await?;
    for track in &tracks {
        switches.add_track_model(track,loader).await?;
    }
    for expansion in &expansions {
        switches.add_expansion_model(expansion);
    }
    queue.regenerate_track_config();
    Ok(())
}
