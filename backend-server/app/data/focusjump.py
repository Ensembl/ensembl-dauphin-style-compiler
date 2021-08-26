import logging
import dbm
import os.path

from command.coremodel import DataAccessor
from model.datalocator import AccessItem
from ncd import NCDRead
from util.string import split_all

PREFIX = "focus:"

class FocusJumpHandler:
    def __init__(self, data_accessor: DataAccessor):
        item = AccessItem("jump")
        accessor = data_accessor.resolver.get(item)
        self._jump_ncd = NCDRead(accessor.ncd())

    def get(self,data_accessor: DataAccessor,location: str):
        if location.startswith(PREFIX):
            for (species,chrom) in split_all(":",location[len(PREFIX):]):
                key = PREFIX + species
                cached = data_accessor.cache.get_jump(location)
                if cached != None:
                    logging.error("using cahed: {0}".format(cached))
                    return cached
                value = self._jump_ncd.get(location.encode("utf-8"))
                if value != None:
                    parts = value.decode("utf-8").split("\t")
                    out = (species+":"+parts[0],int(float(parts[1])),int(float(parts[2])))
                    logging.error("using live: {0}".format(cached))
                    data_accessor.cache.set_jump(location,*out)
                    return out
        return None
