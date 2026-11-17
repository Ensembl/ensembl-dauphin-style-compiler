from threading import Lock

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
        self._catalogue_lock = Lock()

    @staticmethod
    def _load_profiles() -> tuple[dict[tuple[str, ...], dict], dict[str, dict]]:
        """Load fallback track metadata profiles for legacy tracks."""
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
        # Only legacy tracks use the trigger value (to select a fallback profile)
        trigger = data.get("trigger")
        profile = self._profiles.get(tuple(trigger)) if isinstance(trigger, list) else None
        specs = self._profile_specs(data, profile) if profile else self._generic_specs(data)
        for spec in specs:
            track = self._create_track(data, spec)
            tracks.add_track(f"{data['track_id']}-{spec['id']}", track)
        return tracks
    
    # Functions for registering expansion tracks
    def register_track(self, track_id: str, cache=None) -> Tracks:
        cached = self._cached_catalogue(track_id, cache)
        if cached is not None:
            return self._create_coverage_tracks(cached)
        data = self._get_track_data(track_id)
        if not isinstance(data.get("datafiles"), dict) or not data["datafiles"]:
            raise ExpansionMetadataError(f"No datafiles defined for track {track_id}")
        if "rnaseq-coverage" in data["datafiles"]:
            # Fetch the transcriptomic track catalogue (from the config endpoint) and register all coverage tracks in a single block.
            with self._catalogue_lock:
                cached = self._cached_catalogue(track_id, cache)
                if cached is not None:
                    return self._create_coverage_tracks(cached)
                if not data.get("genome_id"):
                    raise ExpansionMetadataError("Transcriptomic track has no genome_id")
                genome_id = data["genome_id"]
                catalogue = self._track_api.get_transcriptomic_configuration(
                    genome_id
                )
                track_ids = self._catalogue_track_ids(catalogue)
                records = self._coverage_records(
                    catalogue, self._track_api.get_tracks(track_ids)
                )
                if track_id not in {record["track_id"] for record in records}:
                    raise ExpansionMetadataError(
                        "Requested track is missing from the transcriptomic tracks catalogue"
                    )
                tracks = self._create_coverage_tracks(records)
                if cache is not None:
                    cache.set_metadata(self._catalogue_key(genome_id), records)
                    for record in records:
                        cache.set_metadata(
                            self._member_key(record["track_id"]), [genome_id]
                        )
                return tracks
        return self._create_track_set(data)

    def _member_key(self, track_id):
        return ["transcriptomic-lanes-v3", self._track_api._host, "track", track_id]

    def _catalogue_key(self, genome_id):
        return ["transcriptomic-lanes-v3", self._track_api._host, genome_id]

    def _cached_catalogue(self, track_id, cache):
        if cache is None:
            return None
        member = cache.get_metadata(self._member_key(track_id))
        return cache.get_metadata(self._catalogue_key(*member)) if member is not None else None

    @staticmethod
    def _catalogue_track_ids(catalogue: dict) -> list[str]:
        try:
            track_ids = []
            for selection in catalogue["selections"]:
                for product in selection["track_products"]:
                    if product["track_type"] != "rnaseq_coverage":
                        raise ValueError(f"Unsupported transcriptomic track type: {product['track_type']}")
                    track_ids.append(product["track_id"])
            track_ids = list(dict.fromkeys(track_ids))
            if not track_ids or len(track_ids) != catalogue["track_count"]:
                raise ValueError("Transcriptomic track count does not match the catalogue count")
            return track_ids
        except (KeyError, TypeError, ValueError) as error:
            raise ExpansionMetadataError(f"Invalid transcriptomic track catalogue: {error}") from error

    @classmethod
    def _coverage_records(cls, catalogue: dict, rendering: list[dict]) -> list[dict]:
        try:
            records = {}
            for track in rendering:
                record = {key: track[key] for key in (
                    "track_id", "label", "genome_id", "dataset_id", "display_order",
                    "lane_order", "datafiles", "settings",
                )}
                record["datafiles"] = dict(record["datafiles"])
                if record["genome_id"] != catalogue["genome_id"]:
                    raise ValueError("Coverage track does not match catalogue genome")
                if set(record["datafiles"]) != {"rnaseq-coverage"}:
                    raise ValueError("Expected one RNA-seq coverage datafile")
                if type(record["lane_order"]) is not int or record["lane_order"] < 0:
                    raise ValueError("Invalid lane order")
                if record["track_id"] in records:
                    raise ValueError("Duplicate transcriptomic track UUID")
                records[record["track_id"]] = record
            if records.keys() != set(cls._catalogue_track_ids(catalogue)):
                raise ValueError("Track metadata does not match transcriptomic track catalogue UUIDs")
            if len({record["display_order"] for record in records.values()}) != 1:
                raise ValueError("Coverage tracks disagree on the block display order")
            orders = sorted(record["lane_order"] for record in records.values())
            if orders != list(range(len(records))):
                raise ValueError(
                    "Transcriptomic track catalogue must have unique consecutive lane orders"
                )
            return sorted(records.values(), key=lambda record: record["lane_order"])
        except (KeyError, TypeError, ValueError) as error:
            raise ExpansionMetadataError(f"Invalid transcriptomic rendering catalogue: {error}") from error

    # Create a RNA-Seq coverage track set (drawn as a stack of track lanes)
    def _create_coverage_tracks(self, records: list[dict]) -> Tracks:
        tracks = Tracks()
        first = records[0]
        group_id = f"transcriptomic-{first['genome_id']}"
        for record in records:
            lanes = self._create_track_set(record)
            for lane in lanes._tracks.values():
                lane.add_value("group_id", group_id)
                lane.add_value("lane_order", record["lane_order"])
            tracks.merge(lanes)
        # Create a single furniture track to draw the category label at the top of the stack
        furniture = Track(
            group_id, program_group="ensembl-webteam/core",
            program_name="rnaseq-coverage-furniture", program_version=1,
            scales=[0, 100, 3],
        )
        for record in records:
            furniture.add_trigger(self._base_trigger(record))
        furniture.add_value("group_id", group_id)
        furniture.add_value("display_order", first["display_order"])
        furniture.add_setting("no-padding", ["settings", "no-padding"])
        furniture.add_setting("tab-selected", ["settings", "tab-selected"])
        tracks.add_track(group_id, furniture)
        return tracks
