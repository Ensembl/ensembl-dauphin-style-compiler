import logging
import requests
import toml
from model.tracks import Track, Tracks
from core.config import SOURCES_TOML

class Expansions:
    def test(self, step):
        red = int(step[0:2],16)
        green = int(step[2:4],16)
        blue = int(step[4:6],16)
        track = Track(step,program_group="ensembl-webteam/core",program_name='test',program_version=2,scales=[0,100,1])
        track.add_trigger(["track","expand",step])
        track.add_value("red",red)
        track.add_value("green",green)
        track.add_value("blue",blue)
        tracks = Tracks()
        tracks.add_track("test",track)
        return tracks
    
    def _read_toml(self):
        with open(SOURCES_TOML) as f:
            toml_file = toml.loads(f.read())
        return toml_file["apis"].get("track_api","localhost")

    def define_variation_track(self, track_id):
        # Read track metadata from Track API
        track_api_host = self._read_toml()
        resp = requests.get(f"{track_api_host}/track/{track_id}", timeout=5)
        if resp.status_code != requests.codes.ok:
            raise Exception(f"Track API request failed for track '{track_id}': {resp.reason}")
        track_data = resp.json()
        if("track_id" not in track_data or track_data["track_id"] != track_id):
            raise Exception(f"Track {track_id} not found in Track API payload: {track_data}")

        # For a single track ID from client, we set up two internal tracks (for zoomed-out & zoomed-in view)
        # declare different programs to be run at different scales 
        track_summary_view = Track(track_id, program_group="ensembl-webteam/core", program_name='variant-summary', program_version=1 ,scales=[6,100,4])
        track_details_view = Track(track_id, program_group="ensembl-webteam/core", program_name='variant-zoomed', program_version=1 ,scales=[1,5,1])

        # define common settings for both tracks
        for track in [track_summary_view, track_details_view]:
            track.add_trigger(["track", "expand-variation", track_id]) # to turn a track on/off
            track.add_setting("name", ["track", "expand-variation", track_id, "name"]) # toggle track name on/off
            #track.add_setting("rank", ["track", "expand-variation", track_id, "display_order"]) # set track order
            track.add_value("track_id", track_id) # will be required for defining the track "leaf" in the tree of tracks
            track.add_value("track_name", track_data['label']) # value to inject track name into the track program
            track.add_value("display_order", track_data['display_order']) # initial track order for the track program

        # define a value for zoomed-out track (datafile location)
        track_summary_view.add_value("datafile", track_data['datafiles']['summary'])

        # define settings/values for zoomed-in track
        track_details_view.add_value("datafile", track_data['datafiles']['details'])
        track_details_view.add_setting("label-snv-id", ["track", "expand-variation", track_id, "label-snv-id"])
        track_details_view.add_setting("label-snv-alleles", ["track", "expand-variation", track_id, "label-snv-alleles"])
        track_details_view.add_setting("label-other-id", ["track", "expand-variation", track_id, "label-other-id"])
        track_details_view.add_setting("label-other-alleles", ["track", "expand-variation", track_id, "label-other-alleles"])
        track_details_view.add_setting("show-extents", ["track", "expand-variation", track_id, "show-extents"])

        # register tracks (with custom registry IDs)
        tracks = Tracks()
        tracks.add_track(f"{track_id}-summary", track_summary_view)
        tracks.add_track(f"{track_id}-detailed", track_details_view)

        return tracks
