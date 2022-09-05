import logging
import dbm
import os.path

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
        self._jump_ncd = None

    def _ensure_ncd(self, data_accessor: DataAccessor):
        if self._jump_ncd is None:
            item = AccessItem("jump")
            accessor = data_accessor.resolver.get(item)
            self._jump_ncd = NCDRead(accessor.ncd())

    def get(self,data_accessor: DataAccessor, location: str, version: Version):
        """

        Args:
            data_accessor (object):
            location (str):
            version (Version):

        Returns:

        """
        self._ensure_ncd(data_accessor)
        if location.startswith(PREFIX):
            (sp_obj,_,chr_name) = data_accessor.data_model.split_total_wire_id(location[len(PREFIX):])
            lookup_key = "focus:{}:{}".format(sp_obj.wire_id,chr_name)
            cached = data_accessor.cache.get_jump(lookup_key,version)
            if cached is not None:
                return cached
            value = self._jump_ncd.get(lookup_key.encode("utf-8"))
            if value is not None:
                parts = value.decode("utf-8").split("\t")
                out = (sp_obj.best_name + ":" + parts[0], int(float(parts[1])), int(float(parts[2])))
                data_accessor.cache.set_jump(lookup_key,*out,version)
                return out
        return None
