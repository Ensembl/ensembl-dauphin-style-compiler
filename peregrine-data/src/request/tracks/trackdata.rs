use peregrine_toolkit::log;
use crate::MaxiResponse;

pub(crate) fn add_tracks_from_response(response: &MaxiResponse) {
    log!("tracks: {:?}",response.tracks());
}
