import toml

from core.config import TRACK_FALLBACK_PROFILES_TOML
from model.tracks import Track, Tracks
from model.datalocator import TrackFilepathError, validate_track_filepath
from model.trackapi import TrackApiClient


class ExpansionMetadataError(Exception):
    """Raised for expansion track registration failures."""

class Expansions:
    _simple_feature_programs = {"cpg", "trna", "tssp"}

    def __init__(self):
        self._track_api = TrackApiClient()
        self._profiles, self._templates = self._load_profiles()

    @staticmethod
    def _load_profiles() -> tuple[dict[tuple[str, ...], dict], dict[str, dict]]:
        """Load fallback track metadata from toml file."""
        config = toml.load(TRACK_FALLBACK_PROFILES_TOML)
        profiles = config.get("profile", {})
        out = {}
        for name, profile in profiles.items():
            trigger = profile.get("trigger")
            if not isinstance(trigger, list) or not trigger:
                raise ExpansionMetadataError(f"Fallback profile '{name}' has no trigger")
            out[tuple(trigger)] = profile
        templates = config.get("template", {})
        if not isinstance(templates, dict):
            raise ExpansionMetadataError("Fallback templates must be a table")
        return out, templates
    
    # Fetch track metadata from Track API
    def _get_track_data(self, track_id: str) -> dict:
        return self._track_api.get_track(track_id)
    
    # Add setting switches to a track object
    @staticmethod
    def _base_trigger(data: dict) -> list[str]:
        return ["track", "expand", data["track_id"]]

    def _add_settings(self, track: Track, data: dict, switches: dict[str, object]) -> None:
        base = self._base_trigger(data)
        for name, target in switches.items():
            if target == "":
                path = base
            elif isinstance(target, str):
                path = base + [target]
            elif isinstance(target, list):
                path = target
            else:
                raise ExpansionMetadataError(f"Invalid setting target for '{name}'")
            track.add_setting(name, path)

    # Create a track object from track metadata
    def _create_track(self, data: dict, spec: dict) -> Track:
        program = spec["program"]
        settings = spec["settings"]
        track = Track(data['track_id'], program_group="ensembl-webteam/core", program_name=program, program_version=1, scales=settings["scales"])
        # add values to the track from the metadata
        track.add_trigger(self._base_trigger(data)) # to turn a track on/off
        track.add_value("track_id", data['track_id']) # will be required for defining the track "leaf" in the tree of tracks
        track.add_value("track_name", data.get("label") or spec.get("label") or data["track_id"])
        track.add_value("additional_info", data.get("additional_info", "")) # make optional info value available for eard programs 
        track.add_value("display_order", data.get("display_order") or spec.get("display_order", 0))
        if spec.get("datafile") is not None:
            track.add_value("datafile", spec["datafile"])
        for name, value in spec.get("values", {}).items():
            track.add_value(name, value)
        # add track-specific setting switches 
        switches = dict(spec.get("switches", {}))
        switches.setdefault("name", "name")
        self._add_settings(track, data, switches)
        # add global settings
        track.add_setting("no-padding", ["settings", "no-padding"]) # track side margins
        track.add_setting("tab-selected", ["settings", "tab-selected"]) # selected track category
        return track
    
    @staticmethod
    def _substitute(value, variables: dict[str, object]):
        """Fill in variables in toml metadata templates."""
        if isinstance(value, str):
            try:
                return value.format(**variables)
            except KeyError as error:
                raise ExpansionMetadataError(
                    f"Fallback template references missing variable '{error.args[0]}'"
                ) from error
        if isinstance(value, list):
            return [Expansions._substitute(item, variables) for item in value]
        if isinstance(value, dict):
            return {
                Expansions._substitute(key, variables): Expansions._substitute(item, variables)
                for key, item in value.items()
            }
        return value

    def _profile_specs(self, data: dict, profile: dict) -> list[dict]:
        template_name = profile.get("template")
        template = self._templates.get(template_name, {}) if template_name else profile
        templates = template.get("track", []) if isinstance(template, dict) else []
        if not templates:
            raise ExpansionMetadataError("Fallback profile has no track templates")
        specs = []
        for template in templates:
            template = self._substitute(template, profile.get("variables", {}))
            datafile_key = template.get("datafile_key")
            datafile = data["datafiles"].get(datafile_key) if datafile_key else None
            if datafile_key and datafile is None:
                raise ExpansionMetadataError(f"No datafile '{datafile_key}' for track {data['track_id']}")
            api_settings = data.get("settings", {}).get(template.get("id"), {})
            if not api_settings:
                api_settings = data.get("settings", {}).get(template["program"], {})
            settings = dict(api_settings or {})
            settings.setdefault("scales", template["scales"])
            specs.append({
                "id": template.get("id", template["program"]),
                "program": template["program"],
                "datafile": datafile,
                "settings": settings,
                "switches": template.get("settings", template.get("switches", {})),
                "values": template.get("values", {}),
            })
        return specs

    @classmethod
    def _program_for_datafile(cls, datafile_key: str, filepath: str) -> str:
        if datafile_key != "simple-features":
            return datafile_key

        filename = filepath.rsplit("/", 1)[-1].rsplit("_", 1)[-1]
        prefix = "simple-features-"
        if filename.startswith(prefix) and filename.endswith(".bb"):
            program = filename[len(prefix):-3]
            if program in cls._simple_feature_programs:
                return program
        raise ExpansionMetadataError(
            f"Unknown simple-features program for datafile '{filepath}'"
        )

    def _generic_specs(self, data: dict) -> list[dict]:
        specs = []
        datafiles = data["datafiles"]
        for datafile_key, filepath in datafiles.items():
            program = self._program_for_datafile(datafile_key, filepath)
            settings = dict(
                data.get("settings", {}).get(
                    program, data.get("settings", {}).get(datafile_key, {})
                )
            )
            if "scales" not in settings:
                if len(datafiles) == 2 and program.endswith("summary"):
                    settings["scales"] = [6, 100, 4]
                elif len(datafiles) == 2 and program.endswith("details"):
                    settings["scales"] = [3, 5, 1]
                else:
                    settings["scales"] = [0, 100, 3]
            specs.append({"id": program, "program": program, "datafile": filepath, "settings": settings, "switches": {switch: switch for switch in settings.get("switches", [])}})
        return specs

    # Create a track set (consisting of a single track, or a pair for zoomed-in/zoomed-out views)
    def _create_track_set(self, data:dict) -> Tracks:
        tracks = Tracks()
        # Validate every path before creating any tracks, so a malformed response
        # can never result in a partially registered expansion.
        try:
            for program, filepath in data["datafiles"].items():
                data["datafiles"][program] = validate_track_filepath(filepath)
        except TrackFilepathError as e:
            raise ExpansionMetadataError(
                f"Invalid datafile path for track '{data.get('track_id', '<unknown>')}': {e}"
            ) from e
        trigger = data.get("trigger")
        profile = self._profiles.get(tuple(trigger)) if isinstance(trigger, list) else None
        if profile is None and isinstance(trigger, list) and trigger[:2] != ["track", "expand"]:
            raise ExpansionMetadataError(
                f"No fallback profile for track '{data.get('track_id', '<unknown>')}' trigger {trigger!r}"
            )
        specs = self._profile_specs(data, profile) if profile else self._generic_specs(data)
        for spec in specs:
            track = self._create_track(data, spec)
            tracks.add_track(f"{data['track_id']}-{spec['id']}", track)
        return tracks
    
    # Functions for registering expansion tracks (defined in boot-tracks.toml config)
    def register_track(self, track_id: str) -> Tracks:
        data = self._get_track_data(track_id)
        if not isinstance(data.get("datafiles"), dict) or not data["datafiles"]:
            raise ExpansionMetadataError(f"No datafiles defined for track {track_id}")
        return self._create_track_set(data)
