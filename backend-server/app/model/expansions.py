from model.tracks import Track, Tracks
from model.datalocator import TrackFilepathError, validate_track_filepath
from model.trackapi import TrackApiClient


class ExpansionMetadataError(Exception):
    """Track API metadata cannot safely be registered as an expansion."""

class Expansions:
    def __init__(self):
        self._track_api = TrackApiClient()
    
    # Fetch track metadata from Track API
    def _get_track_data(self, track_id: str) -> dict:
        return self._track_api.get_track(track_id)
    
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
        track.add_value("track_name", data['label']) # for drawing track labels in eard programs
        track.add_value("additional_info", data.get("additional_info", "")) # make optional info value available for eard programs 
        track.add_value("display_order", data['display_order']) # initial track order for the track program
        track.add_value("datafile", data['datafiles'][program])
        # add track-specific setting switches 
        switches = settings.get("switches", [])
        switches.append("name")  # toggle track name on/off
        self._add_settings(track, data, switches)
        # add global settings
        track.add_setting("no-padding", ["settings", "no-padding"]) # track side margins
        track.add_setting("tab-selected", ["settings", "tab-selected"]) # selected track category
        return track
    
    # Create a track set (consisting of a single track, or a pair for zoomed-in/zoomed-out views)
    def _create_track_set(self, data:dict) -> Tracks:
        tracks = Tracks()
        # each datafile is tied to an Eard program
        programs = list(data["datafiles"].keys())
        # Validate every path before creating any tracks, so a malformed response
        # can never result in a partially registered expansion.
        try:
            for program in programs:
                data["datafiles"][program] = validate_track_filepath(data["datafiles"][program])
        except TrackFilepathError as e:
            raise ExpansionMetadataError(
                f"Invalid datafile path for track '{data.get('track_id', '<unknown>')}': {e}"
            ) from e
        for program in programs:
            if program not in data["settings"]:
                data["settings"][program] = {}
            # set default track scales (min, max, step) when not defined in Track API
            if "scales" not in data["settings"][program]:
                if len(programs) == 2:
                    if program.endswith("summary"):
                        data["settings"][program]["scales"] = [6, 100, 4]
                    elif program.endswith("details"):
                        data["settings"][program]["scales"] = [3, 5, 1]
                    else:
                        raise Exception(f"Unexpected program name: {program}")
                else:
                    data["settings"][program]["scales"] = [0, 100, 3]
            track = self._create_track(data, program)
            tracks.add_track(f"{data['track_id']}-{program}", track)
        return tracks
    
    # Functions for registering expansion tracks (defined in boot-tracks.toml config)
    def register_track(self, track_id: str) -> Tracks:
        data = self._get_track_data(track_id)
        if not isinstance(data.get("datafiles"), dict) or not data["datafiles"]:
            raise ExpansionMetadataError(f"No datafiles defined for track {track_id}")
        return self._create_track_set(data)
