import logging
import dbm
import os.path

from ncd.accessor import NCDFileAccessor
from command.coremodel import DataAccessor
from model.datalocator import AccessItem
from ncd import NCDRead, NCDHttpAccessor, NCDFileAccessor

PREFIX = "focus"

class FocusJumpHandler:
    def __init__(self, data_accessor: DataAccessor):
        item = AccessItem("jump")
        accessor = data_accessor.resolver.get(item)
        if accessor.url != None:
            ncd_accessor = NCDHttpAccessor(accessor.url)
        elif accessor.file != None:
            ncd_accessor = NCDFileAccessor(accessor.file)
        else:
            raise Exception("bad accessor for jump file")
        self._jump_ncd = NCDRead(ncd_accessor)

    def get(self,data_accessor: DataAccessor,location: str):
        if location.startswith(PREFIX):
            value = self._jump_ncd.get(location.encode("utf-8"))
            if value != None:
                parts = value.decode("utf-8").split("\t")
                return (parts[0],int(float(parts[1])),int(float(parts[2])))
        return None
