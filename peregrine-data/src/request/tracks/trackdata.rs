use crate::{MaxiResponse, PeregrineApiQueue, Switches};

pub(crate) fn add_tracks_from_response(response: &MaxiResponse, switches: &Switches, queue: &PeregrineApiQueue) {
    for track in response.tracks().iter() {
        switches.add_track_model(track);
    }
    queue.regenerate_track_config();
}
