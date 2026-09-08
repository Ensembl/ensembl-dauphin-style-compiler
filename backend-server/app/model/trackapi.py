import requests
import toml

from core.config import SOURCES_TOML
from model.datalocator import TrackFilepathError, validate_track_filepath


class TrackApiError(Exception):
    """Track API could not provide a usable datafile for a browser track."""


class TrackApiClient:
    """Small client for the Track API endpoints used by Genome Browser."""

    def __init__(self):
        with open(SOURCES_TOML) as f:
            config = toml.loads(f.read())
        self._host = config["apis"].get("track_api", "localhost").rstrip("/")

    def get_track(self, track_id: str) -> dict:
        response = requests.get(f"{self._host}/track/{track_id}", timeout=5)
        if response.status_code != requests.codes.ok:
            raise TrackApiError(f"Track API request failed for track '{track_id}': {response.reason}")
        track = response.json()
        if track.get("track_id") != track_id:
            raise TrackApiError(f"Track {track_id} not found in Track API payload")
        return track

    def get_track_categories(self, genome_id: str) -> list[dict]:
        response = requests.get(f"{self._host}/track_categories/{genome_id}", timeout=5)
        if response.status_code != requests.codes.ok:
            raise TrackApiError(
                f"Track API request failed for genome '{genome_id}': {response.reason}"
            )
        payload = response.json()
        categories = payload.get("track_categories")
        if not isinstance(categories, list):
            raise TrackApiError(f"Invalid track_categories payload for genome '{genome_id}'")
        return categories


class TrackDatafileResolver:
    """Resolve static-browser endpoints to Track API datafiles for one genome."""

    # Fixed-switch tracks are selected by their stable client trigger. Focus
    # tracks have no Track API entry, so they reuse the first source track of
    # the given type, by the convention supplied by Track API.
    _endpoint_sources = {
        "gc": ("trigger", ["track", "gc"], "gc"),
        "contig": ("trigger", ["track", "contig"], "contig"),
        "shimmer-contig": ("trigger", ["track", "contig"], "contig"),
        "regulation": ("trigger", ["track", "regulation"], "regulation"),
        "gene-overview": ("type", "gene", "gene"),
        "gene": ("type", "gene", "gene"),
        "transcript": ("type", "gene", "gene"),
        "variant-summary": ("type", "variant", "variant-summary"),
        "variant-details": ("type", "variant", "variant-details"),
    }

    def __init__(self):
        self._client = TrackApiClient()
        self._categories: dict[str, list[dict]] = {}
        self._tracks: dict[str, dict] = {}
        self._datafiles: dict[tuple[str, str], str] = {}

    def _track_categories(self, genome_id: str) -> list[dict]:
        if genome_id not in self._categories:
            self._categories[genome_id] = self._client.get_track_categories(genome_id)
        return self._categories[genome_id]

    def _track(self, track_id: str) -> dict:
        if track_id not in self._tracks:
            self._tracks[track_id] = self._client.get_track(track_id)
        return self._tracks[track_id]

    def _find_track_id(self, genome_id: str, selector: str, value) -> str:
        for category in self._track_categories(genome_id):
            if not isinstance(category, dict):
                continue
            tracks = category.get("track_list", [])
            if not isinstance(tracks, list):
                continue
            for track in tracks:
                if not isinstance(track, dict):
                    continue
                if track.get(selector) == value and track.get("track_id"):
                    return track["track_id"]
        raise TrackApiError(
            f"No Track API track for genome '{genome_id}' matches {selector}={value!r}"
        )

    def datafile_for_endpoint(self, genome_id: str, endpoint: str) -> str | None:
        source = self._endpoint_sources.get(endpoint)
        if source is None:
            return None

        cache_key = (genome_id, endpoint)
        if cache_key not in self._datafiles:
            selector, value, program = source
            track_id = self._find_track_id(genome_id, selector, value)
            datafiles = self._track(track_id).get("datafiles", {})
            if not isinstance(datafiles, dict):
                raise TrackApiError(f"Invalid datafiles payload for Track API track '{track_id}'")
            filepath = datafiles.get(program)
            if filepath is None:
                raise TrackApiError(
                    f"Track API track '{track_id}' has no datafile for program '{program}'"
                )
            try:
                self._datafiles[cache_key] = validate_track_filepath(filepath)
            except TrackFilepathError as e:
                raise TrackApiError(
                    f"Invalid datafile path for Track API track '{track_id}': {e}"
                ) from e
        return self._datafiles[cache_key]
