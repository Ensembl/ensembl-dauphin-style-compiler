import logging

import requests
import toml

from core.config import SOURCES_TOML


class CoreApiClient:
    """
    Small client for fetching genomic coordinates for a transcript from Core GraphQL.
    This is used for focus transcript requests to reposition the panel and query bigbed.
    """

    _TRANSCRIPT_LOCATION_QUERY = """
query TranscriptLocation($genomeId: String!, $transcriptId: String!) {
  transcript(by_id: { genome_id: $genomeId, stable_id: $transcriptId }) {
    slice {
      region {
        name
      }
      location {
        start
        end
      }
    }
  }
}
"""

    def __init__(self):
        self._logger = logging.getLogger(__name__)
        self._core_api_url = self._read_toml()

    def _read_toml(self) -> str | None:
        with open(SOURCES_TOML) as f:
            toml_file = toml.loads(f.read())
        return toml_file.get("apis", {}).get("core_api", None)

    def get_transcript_location(
        self, for_id: tuple[str,str]
    ) -> tuple[str, int, int] | None:

        (genome_id, transcript_id) = for_id
        if not self._core_api_url:
            return None

        payload: dict[str, str | dict[str, str]] = {
            "query": self._TRANSCRIPT_LOCATION_QUERY,
            "variables": {
                "genomeId": genome_id,
                "transcriptId": transcript_id,
            },
        }

        try:
            response = requests.post(self._core_api_url, json=payload, timeout=5)
            response.raise_for_status()
            body = response.json()
        except Exception as e:
            self._logger.warning(
                "Thoas transcript lookup failed for '%s' (%s): %s",
                transcript_id,
                genome_id,
                e,
            )
            return None

        if body.get("errors"):
            self._logger.warning(
                "Thoas transcript lookup returned GraphQL errors for '%s': %s",
                transcript_id,
                body.get("errors"),
            )
            return None

        transcript = body.get("data", {}).get("transcript")
        if not transcript:
            return None

        slc = transcript.get("slice", {})
        region_name = slc.get("region", {}).get("name")
        start = slc.get("location", {}).get("start")
        end = slc.get("location", {}).get("end")

        if region_name is None or start is None or end is None:
            return None

        # add padding around the transcript (for the viewport; same as in NCD file)
        padding = (end-start)/2
        start -= padding
        end += padding
        return (region_name, int(start), int(end))
