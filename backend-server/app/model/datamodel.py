from uuid import UUID
from .species import Species
from util.string import split_all
from core.exceptions import RequestException

"""
Converts a stick id to a Chromosome() object which includes means of access for the data.
The main tasks are resolving species IDs and determining data-sets based on versioning.
"""
class DataModel(object):
    """
    Args:
        data_accessor ():
    """
    def __init__(self):
        self._species = {} # Species instances
        self._uuids = {}

    def stick(self, data_accessor, stick_id:str):
        uuid = stick_id.split(":")[0]
        try:
            UUID(uuid)
        except ValueError:
            raise RequestException(f"Unexpected genome id format: {uuid}")
        if uuid not in self._species:
            self._species[uuid] = Species(uuid)
            self._uuids[uuid] = uuid
        return self._species[uuid].chromosome(data_accessor, stick_id)

    def species(self, uuid:str): # return Species() instance or None
        return self._species.get(uuid)

    def canonical_genome_id(self, alias):
        print(f"canonical_genome_id: {alias}")
        for (prefix, chr) in split_all(":", alias):
            if prefix in self._uuids:
                return prefix
        return None

    def best_stick_id(self, alias):
        for (prefix, chr) in split_all(":", alias):
            uuid = self._uuids.get(prefix)
            if uuid is not None:
                return self._species[uuid].best_name+chr
        return None

    def split_total_wire_id(self, total_wire_id: str):
        print(f"datamodel.split_total_wire_id: {total_wire_id}")
        # we know that we split on a colon, but which one? We go from longest to shortest, trying
        # all combinations of positions, :-( .
        parts = total_wire_id.split(":")
        for num in reversed(range(1,len(parts)+1)):
            for start in range(0,len(parts)-num+1):
                species = ":".join(parts[start:start+num])
                uuid = self._uuids.get(species,None)
                if uuid is not None:
                    species = self._species[uuid]
                    out = parts[:start] + [species.wire_id] + parts[start+num:]
                    return (species,":".join(out))
        raise RequestException("cannot split id")
