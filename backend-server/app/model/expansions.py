import requests
import toml

from model.tracks import Track, Tracks
from core.config import SOURCES_TOML

class Expansions:
    def __init__(self):
        self._track_api_host = self._read_toml()

    # Read the Track API hostname from sources-<env>.toml file
    def _read_toml(self) -> str:
        with open(SOURCES_TOML) as f:
            toml_file = toml.loads(f.read())
        return toml_file["apis"].get("track_api","localhost")
    
    # Fetch track metadata from Track API
    def _get_track_data(self, track_id: str) -> dict:
        resp = requests.get(f"{self._track_api_host}/track/{track_id}", timeout=5)
        if resp.status_code != requests.codes.ok:
            raise Exception(f"Track API request failed for track '{track_id}': {resp.reason}")
        track_data = resp.json()
        if("track_id" not in track_data or track_data["track_id"] != track_id):
            raise Exception(f"Track {track_id} not found in Track API payload: {track_data}")
        return track_data
    
    # Add settings (switches) to a track object
    def _add_settings(self, track: Track, data: dict, settings: list[str]=[]) -> None:
        for setting in settings:
            track.add_setting(setting, data['trigger']+[setting])

    # Create a track object from track metadata
    def _create_track(self, data: dict, program_name: str='', scales: list[int]=[0,100,3], settings: list[str]=[]) -> Track:
        # declare a program to be run (with optional trigger zoom levels)
        try:
            filekey = list(data['datafiles'].keys()).pop()
        except IndexError:
            raise Exception(f"No datafiles found for track {data['track_id']}")
        track = Track(data['track_id'], program_group="ensembl-webteam/core", program_name=program_name or filekey, program_version=1, scales=scales)
        # add values from track metadata
        track.add_trigger(data['trigger']) # to turn a track on/off
        track.add_value("track_id", data['track_id']) # will be required for defining the track "leaf" in the tree of tracks
        track.add_value("track_name", data['label']) # value to inject track name into the track program
        track.add_value("display_order", data['display_order']) # initial track order for the track program
        track.add_value("datafile", data['datafiles'][filekey])
        # add settings/switches
        settings.append("name") # switch to toggle track name on/off
        self._add_settings(track, data, settings)
        track.add_setting("tab-selected", ["settings", "tab-selected"])
        return track
    
    # Create a pair of tracks for zoomed-in/zoomed-out views
    def _create_track_set(self, track_id: str, name: str, scales:dict|None=None, settings:dict={}) -> Tracks:
        track_data = self._get_track_data(track_id)
        scales = scales or {"summary": [6, 100, 4], "details": [1, 5, 1]}
        tracks = Tracks()
        for zoom_level in ['summary', 'details']:
            track = self._create_track(data=track_data, program_name=name+'-'+zoom_level, scales=scales.get(zoom_level,[]), settings=settings.get(zoom_level,[]))
            if(zoom_level in track_data["datafiles"]):
                track.add_value("datafile", track_data["datafiles"][zoom_level])
            tracks.add_track(f"{track_id}-{zoom_level}", track)
        return tracks
    
    # Functions for registering expansion tracks. Called on boot time from boot-tracks.toml config
    def register_track(self, track_id: str) -> Tracks:
        data = self._get_track_data(track_id)
        track = self._create_track(data)
        tracks = Tracks()
        tracks.add_track(track_id, track)
        return tracks
   
    def register_variation_track(self, track_id: str) -> Tracks:
        details_track_settings = ["label-snv-id", "label-snv-alleles", "label-other-id", "label-other-alleles", "show-extents"]
        return self._create_track_set(track_id, "variant", settings={"details": details_track_settings})
