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
        track_id = data["track_id"]
        if not len(data["datafiles"]):
            raise Exception(f"No datafiles found for track {track_id}")
        if "settings" not in data:
            data["settings"] = {}
        tracks = Tracks()
        # each datafile is tied to an Eard program
        for program in data["datafiles"].keys():
            if program not in data["settings"]:
                data["settings"][program] = {}
            # set default track scales (min, max, step) if not defined in metadata
            if "scales" not in data["settings"][program]:
                data["settings"][program]["scales"] = [6, 100, 4] if program.endswith("summary") else [1, 5, 1] if program.endswith("details") else [0, 100, 3]
            track = self._create_track(data, program)
            tracks.add_track(f"{track_id}-{program}", track)
        return tracks
    
    # Functions for registering expansion tracks (defined in boot-tracks.toml config)
    def register_track(self, track_id: str) -> Tracks:
        data = self._get_track_data(track_id)
        return self._create_track_set(data)

    # Special case for variation tracks (until migrated to generic expansion track)
    def register_variation_track(self, track_id: str) -> Tracks:
        data = self._get_track_data(track_id)
        # stub for upcoming Track API changes
        data["datafiles"]["variant-details"] = data["datafiles"].pop("details")
        data["datafiles"]["variant-summary"] = data["datafiles"].pop("summary")
        data["settings"] = {}
        data["settings"]["variant-details"] = {}
        data["settings"]["variant-details"]["switches"] = ["label-snv-id",
            "label-snv-alleles", "label-other-id", "label-other-alleles", "show-extents"]
        return self._create_track_set(data)
