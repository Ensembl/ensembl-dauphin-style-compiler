from core.exceptions import UpstreamRequestError
from model.upstream import request_json
import toml

from core.config import SOURCES_TOML

class CoreApiClient:
    """
    Small client for fetching genomic coordinates for a transcript from Core GraphQL API.
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

    _GENE_LOCATION_QUERY = """
query GeneLocation($genomeId: String!, $geneId: String!) {
    gene(by_id: { genome_id: $genomeId, stable_id: $geneId }) {
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
        self._core_api_url = self._read_toml()

    def _read_toml(self) -> str | None:
        with open(SOURCES_TOML) as f:
            toml_file = toml.loads(f.read())
        return toml_file.get("apis", {}).get("core_api", None)

    def _extract_location(
        self, genome_id: str, feature: dict
    ) -> tuple[str, int, int] | tuple[None, None, None]:
        empty_location = (None, None, None)

        slc = feature.get("slice", {})
        region_name = slc.get("region", {}).get("name")
        start = slc.get("location", {}).get("start")
        end = slc.get("location", {}).get("end")

        if region_name is None or start is None or end is None:
            return empty_location

        # Add padding around the feature for viewport postitioning.
        padding = (end - start) / 2
        start -= padding
        end += padding
        stick = f"{genome_id}:{region_name}"
        return (stick, max(0, int(start)), int(end))

    def _get_feature_location(
        self,
        genome_id: str,
        feature_id: str,
        feature_type: str,
        variable_name: str,
        query: str,
    ) -> tuple[str, int, int] | tuple[None, None, None]:
        empty_location = (None, None, None)

        if not self._core_api_url:
            return empty_location

        payload: dict[str, str | dict[str, str]] = {
            "query": query,
            "variables": {
                "genomeId": genome_id,
                variable_name: feature_id,
            },
        }

        body = request_json("Core API", "POST", self._core_api_url, json=payload)
        if not isinstance(body, dict) or body.get("errors"):
            raise UpstreamRequestError(
                "Core API", "POST", self._core_api_url,
                body.get("errors") if isinstance(body, dict) else "Invalid GraphQL response",
                upstream_status=200,
            )

        data = body.get("data")
        if data is None:
            return empty_location
        if not isinstance(data, dict):
            raise UpstreamRequestError(
                "Core API", "POST", self._core_api_url, "Invalid GraphQL data", upstream_status=200,
            )
        feature = data.get(feature_type)
        if not feature:
            return empty_location
        if not isinstance(feature, dict):
            raise UpstreamRequestError(
                "Core API", "POST", self._core_api_url, "Invalid GraphQL feature", upstream_status=200,
            )

        return self._extract_location(genome_id, feature)

    def get_transcript_location(
        self, for_id: tuple[str,str]
    ) -> tuple[str, int, int] | tuple[None, None, None]:
        (genome_id, transcript_id) = for_id
        return self._get_feature_location(
            genome_id,
            transcript_id,
            "transcript",
            "transcriptId",
            self._TRANSCRIPT_LOCATION_QUERY,
        )

    def get_gene_location(
        self, for_id: tuple[str, str]
    ) -> tuple[str, int, int] | tuple[None, None, None]:
        (genome_id, gene_id) = for_id
        return self._get_feature_location(
            genome_id,
            gene_id,
            "gene",
            "geneId",
            self._GENE_LOCATION_QUERY,
        )
