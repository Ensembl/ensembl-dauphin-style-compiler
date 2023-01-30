import logging
import dbm
import os.path

from model.species import Species
from command.coremodel import DataAccessor
from model.datalocator import AccessItem
from ncd import NCDRead
from model.version import Version

"""
Attributes:
    PREFIX (str):
"""

PREFIX = "focus:"

class FocusJumpHandler:
    """

    Args:
        data_accessor ():
    """
    def __init__(self):
        self._ncd_files = {}

    def _ensure_ncd(self, data_accessor: DataAccessor, sp_obj: Species):
        if sp_obj.wire_id not in self._ncd_files:
            accessor = data_accessor.resolver.get(sp_obj.item_path("jump"))
            self._ncd_files[sp_obj.wire_id] = NCDRead(accessor.ncd())

    def get(self,data_accessor: DataAccessor, location: str, version: Version):
        """

        Args:
            data_accessor (object):
            location (str):
            version (Version):

        Returns:

        """
        # We need to extract something which at least maps to a genome UUID from the chromosome
        # and use that to choose the file and lookup within it.
        if location.startswith(PREFIX):
            (sp_obj,good_id) = data_accessor.data_model.split_total_wire_id(location[len(PREFIX):])
            self._ensure_ncd(data_accessor,sp_obj)
            lookup_key = "focus:{}".format(good_id)
            cached = data_accessor.cache.get_jump(lookup_key,version)
            if cached is not None:
                return cached
            value = self._ncd_files[sp_obj.wire_id].get(lookup_key.encode("utf-8"))
            if value is not None:
                parts = value.decode("utf-8").split("\t")
                out = (sp_obj.best_name + ":" + parts[0], int(float(parts[1])), int(float(parts[2])))
                data_accessor.cache.set_jump(lookup_key,*out,version)
                return out
        return None
