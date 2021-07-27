import logging
import dbm
import os.path
from command.coremodel import DataAccessor

PREFIX = "focus"

class FocusJumpHandler:
    def get(self,data_accessor: DataAccessor,location):
        if location.startswith(PREFIX):
            jump_file = os.path.join(data_accessor.data_model.files_dir,"jump")
            jumps = dbm.open(jump_file)
            try:
                value = jumps.get(location)
                if value != None:
                    parts = value.decode("utf-8").split("\t")
                    return (parts[0],int(parts[1]),int(parts[2]))
            finally:
                jumps.close()
        return None
