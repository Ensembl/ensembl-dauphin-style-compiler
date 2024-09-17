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
    
    # Add setting switches to a track object
    def _add_settings(self, track: Track, data: dict, switches: list[str]=[]) -> None:
        for switch in switches:
            track.add_setting(switch, data['trigger']+[switch])

    # Create a track object from track metadata
    def _create_track(self, data: dict, program: str) -> Track:
        settings = data['settings'][program]
        track = Track(data['track_id'], program_group="ensembl-webteam/core", program_name=program, program_version=1, scales=settings["scales"])
        # add values to the track from the metadata
        track.add_trigger(data['trigger']) # to turn a track on/off
        track.add_value("track_id", data['track_id']) # will be required for defining the track "leaf" in the tree of tracks
        track.add_value("track_name", data['label']) # value to inject track name into the track program
        track.add_value("display_order", data['display_order']) # initial track order for the track program
        track.add_value("datafile", data['datafiles'][program])
        # add setting switches 
        switches = settings.get("switches", [])
        switches.append("name")  # switch to toggle track name on/off
        self._add_settings(track, data, switches)
        track.add_setting("tab-selected", ["settings", "tab-selected"]) # global setting to track the selected tab
        return track
    
    # Create a track set (consisting of a single track, or a pair for zoomed-in/zoomed-out views)
    def _create_track_set(self, data:dict) -> Tracks:
        tracks = Tracks()
        # each datafile is tied to an Eard program
        for program in data["datafiles"].keys():
            if program not in data["settings"]:
                data["settings"][program] = {}
            # use default track scales (min, max, step) if not defined in Track API
            if "scales" not in data["settings"][program]:
                if program.endswith("summary"):
                    data["settings"][program]["scales"] = [6, 100, 4]
                elif program.endswith("details"):
                    data["settings"][program]["scales"] = [1, 5, 1]
                else:
                    data["settings"][program]["scales"] = [0, 100, 3]
            track = self._create_track(data, program)
            tracks.add_track(f"{data['track_id']}-{program}", track)
        return tracks
    
    # Functions for registering expansion tracks (defined in boot-tracks.toml config)
    def register_track(self, track_id: str) -> Tracks:
        data = self._get_track_data(track_id)
        if not len(data["datafiles"]):
            raise Exception(f"No datafiles defined for track {track_id}")
        return self._create_track_set(data)