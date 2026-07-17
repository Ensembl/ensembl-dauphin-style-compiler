from command.coremodel import DataAccessor
from model.graphql import CoreApiClient
from model.version import Version


class FocusJumpHandler:
    """

    Args:
        data_accessor ():
    """
    def __init__(self):
        self._core_api = CoreApiClient()

    def get(self, data_accessor: DataAccessor, lookup: str, version: Version):
        """

        Args:
            data_accessor (object):
            lookup (str): focus:[gene|transcript|variant|location]:<genome_uuid>:<id>
            version (Version):

        Returns:

        """
        if lookup.startswith('focus:') and lookup.count(':') == 3:
            (_, focus_type, genome_id, object_id) = lookup.split(':')

            # check cache first
            cached = data_accessor.cache.get_jump(lookup,version)
            if cached is not None:
                return cached

            if focus_type == "transcript":
                (stick, start, end) = self._core_api.get_transcript_location((genome_id, object_id))
            elif focus_type == "gene":
                (stick, start, end) = self._core_api.get_gene_location((genome_id, object_id))
            else:
                return None

            if stick is not None and start is not None and end is not None:
                out = (stick, start, end)
                data_accessor.cache.set_jump(lookup, *out, version)
                return out
        return None
