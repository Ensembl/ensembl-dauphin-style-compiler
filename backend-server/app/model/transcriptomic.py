from threading import Lock
from typing import Callable

from model.trackapi import TrackApiClient
from model.tracks import ExpansionMetadataError, Track, Tracks


class TranscriptomicTracks:
    """Resolve and register a genome's independently switched transcriptomic lanes."""

    _programs = {"rnaseq-coverage", "rnaseq-gene-model", "rnaseq-splice-junction"}

    def __init__(
        self, track_api: TrackApiClient, create_track_set: Callable[[dict], Tracks]
    ):
        self._track_api = track_api
        self._create_track_set = create_track_set
        self._catalogue_lock = Lock()

    @classmethod
    def supports(cls, data: dict) -> bool:
        return bool(cls._programs.intersection(data["datafiles"]))

    def cached_tracks(self, track_id: str, cache) -> Tracks | None:
        records = self._cached_catalogue(track_id, cache)
        return self._create_tracks(records) if records is not None else None

    def register(self, data: dict, cache=None) -> Tracks:
        track_id = data["track_id"]
        with self._catalogue_lock:
            cached = self.cached_tracks(track_id, cache)
            if cached is not None:
                return cached
            if not data.get("genome_id"):
                raise ExpansionMetadataError("Transcriptomic track has no genome_id")
            genome_id = data["genome_id"]
            catalogue = self._track_api.get_transcriptomic_configuration(genome_id)
            track_ids = self._catalogue_track_ids(catalogue)
            records = self._rendering_records(
                catalogue, self._track_api.get_tracks(track_ids)
            )
            if track_id not in {record["track_id"] for record in records}:
                raise ExpansionMetadataError(
                    "Requested track is missing from the transcriptomic tracks catalogue"
                )
            tracks = self._create_tracks(records)
            if cache is not None:
                cache.set_metadata(self._catalogue_key(genome_id), records)
                for record in records:
                    cache.set_metadata(
                        self._member_key(record["track_id"]), [genome_id]
                    )
            return tracks

    def _member_key(self, track_id):
        return ["transcriptomic-lanes", self._track_api._host, "track", track_id]

    def _catalogue_key(self, genome_id):
        return ["transcriptomic-lanes", self._track_api._host, genome_id]

    def _cached_catalogue(self, track_id, cache):
        if cache is None:
            return None
        member = cache.get_metadata(self._member_key(track_id))
        return (
            cache.get_metadata(self._catalogue_key(*member))
            if member is not None
            else None
        )

    @staticmethod
    def _catalogue_track_ids(catalogue: dict) -> list[str]:
        try:
            track_ids = []
            for selection in catalogue["selections"]:
                for product in selection["track_products"]:
                    if product["track_type"] not in {"bigwig", "bigbed"}:
                        raise ValueError(
                            f"Unsupported transcriptomic track type: {product['track_type']}"
                        )
                    track_ids.append(product["track_id"])
            track_ids = list(dict.fromkeys(track_ids))
            if not track_ids or len(track_ids) != catalogue["track_count"]:
                raise ValueError(
                    "Transcriptomic track count does not match the catalogue count"
                )
            return track_ids
        except (KeyError, TypeError, ValueError) as error:
            raise ExpansionMetadataError(
                f"Invalid transcriptomic track catalogue: {error}"
            ) from error

    @classmethod
    def _rendering_records(cls, catalogue: dict, rendering: list[dict]) -> list[dict]:
        try:
            products = {
                product["track_id"]: product
                for selection in catalogue["selections"]
                for product in selection["track_products"]
            }
            records = {}
            for track in rendering:
                record = {
                    key: track[key]
                    for key in (
                        "track_id",
                        "label",
                        "genome_id",
                        "dataset_id",
                        "display_order",
                        "lane_order",
                        "datafiles",
                        "settings",
                    )
                }
                record["datafiles"] = dict(record["datafiles"])
                if record["genome_id"] != catalogue["genome_id"]:
                    raise ValueError(
                        "Transcriptomic track does not match catalogue genome"
                    )
                if len(record["datafiles"]) != 1 or not cls.supports(record):
                    raise ValueError("Expected one transcriptomic renderer datafile")
                product = products[record["track_id"]]
                program = next(iter(record["datafiles"]))
                expected_format = "bigwig" if program == "rnaseq-coverage" else "bigbed"
                if product["track_type"] != expected_format:
                    raise ValueError(
                        "Transcriptomic product format does not match its renderer"
                    )
                record["label"] = f"{record['label']} · {product['display_label']}"
                if type(record["lane_order"]) is not int or record["lane_order"] < 0:
                    raise ValueError("Invalid lane order")
                if record["track_id"] in records:
                    raise ValueError("Duplicate transcriptomic track UUID")
                records[record["track_id"]] = record
            if records.keys() != set(cls._catalogue_track_ids(catalogue)):
                raise ValueError(
                    "Track metadata does not match transcriptomic track catalogue UUIDs"
                )
            if len({record["display_order"] for record in records.values()}) != 1:
                raise ValueError(
                    "Transcriptomic tracks disagree on the block display order"
                )
            orders = sorted(record["lane_order"] for record in records.values())
            if orders != list(range(len(records))):
                raise ValueError(
                    "Transcriptomic track catalogue must have unique consecutive lane orders"
                )
            return sorted(records.values(), key=lambda record: record["lane_order"])
        except (KeyError, TypeError, ValueError) as error:
            raise ExpansionMetadataError(
                f"Invalid transcriptomic rendering catalogue: {error}"
            ) from error

    def _create_tracks(self, records: list[dict]) -> Tracks:
        tracks = Tracks()
        first = records[0]
        group_id = f"transcriptomic-{first['genome_id']}"
        for record in records:
            lanes = self._create_track_set(record)
            for lane in lanes._tracks.values():
                lane.add_value("group_id", group_id)
                lane.add_value("lane_order", record["lane_order"])
            tracks.merge(lanes)
        furniture = Track(
            group_id,
            program_group="ensembl-webteam/core",
            program_name="transcriptomic-furniture",
            program_version=1,
            scales=[0, 100, 3],
        )
        for record in records:
            furniture.add_trigger(["track", "expand", record["track_id"]])
        furniture.add_value("group_id", group_id)
        furniture.add_value("display_order", first["display_order"])
        furniture.add_setting("no-padding", ["settings", "no-padding"])
        furniture.add_setting("tab-selected", ["settings", "tab-selected"])
        tracks.add_track(group_id, furniture)
        return tracks
