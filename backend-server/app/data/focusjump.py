from model.species import Species
from command.coremodel import DataAccessor
from ncd import NCDRead
from model.thoas import Thoas
from model.version import Version


class FocusJumpHandler:
    """

    Args:
        data_accessor ():
    """
    def __init__(self):
        self._ncd_files = {}
        self._thoas = Thoas()

    def _ensure_ncd(self, data_accessor: DataAccessor, sp_obj: Species):
        if sp_obj.genome_id not in self._ncd_files:
            accessor = data_accessor.resolver.get(sp_obj.item_path("jump"))
            self._ncd_files[sp_obj.genome_id] = NCDRead(accessor.ncd())

    def get(self, data_accessor: DataAccessor, lookup: str, version: Version):
        """

        Args:
            data_accessor (object):
            lookup (str): focus:[gene|variant|location]:<genome_uuid>:<id>
            version (Version):

        Returns:

        """
        if lookup.startswith('focus:') and lookup.count(':') == 3:
            (_, focus_type, genome_id, object_id) = lookup.split(':')
            # extract genome uuid => jump file location
            sp_obj = data_accessor.data_model.species(genome_id)
            self._ensure_ncd(data_accessor, sp_obj)
            cached = data_accessor.cache.get_jump(lookup,version)
            if cached is not None:
                return cached
            value = self._ncd_files[sp_obj.genome_id].get(lookup.encode("utf-8"))
            if value is not None:
                parts = value.decode("utf-8").split("\t")
                out = (sp_obj.genome_id + ":" + parts[0], int(float(parts[1])), int(float(parts[2])))
                data_accessor.cache.set_jump(lookup, *out, version)
                return out

            # Transcript IDs are not currently indexed in jump.ncd; resolve via Thoas.
            if focus_type == "transcript":
                tr_location = self._thoas.get_transcript_location(genome_id, object_id)
                if tr_location is not None:
                    (region_name, start, end) = tr_location
                    out = (f"{genome_id}:{region_name}", start, end)
                    data_accessor.cache.set_jump(lookup, *out, version)
                    return out
        return None
