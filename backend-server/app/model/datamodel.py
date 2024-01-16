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
        # genome UUIDs => Species obj. cache
        self._species = {}

    # Args: stick string (<genome_uuid>:<chr>)
    # Returns: Chromosome instance
    def stick(self, data_accessor, stick_id:str):
        genome_id = stick_id.split(":")[0]
        # Handle genome UUIDs. New UUIDs are fed from StickHandler requests.
        if genome_id not in self._species:
            try:
                UUID(genome_id)
            except ValueError:
                raise RequestException(f"Unexpected genome id format: {genome_id}")
            self._species[genome_id] = Species(genome_id)
        
        return self._species[genome_id].chromosome(data_accessor, stick_id)

    def species(self, genome_id:str):
        # Returns Species() instance or None
        return self._species.get(genome_id)

    def canonical_genome_id(self, alias):
        print(f"canonical_genome_id IN: {alias}")
        for (prefix, chr) in split_all(":", alias):
            if prefix in self._species:
                print(f"canonical_genome_id OUT: {prefix}")
                return prefix
        return None

    def best_stick_id(self, alias):
        print(f"best_stick_id IN: {alias}")
        for (prefix, chr) in split_all(":", alias):
            if prefix in self._species:
                print("best_stick_id OUT: {0}".format(self._species[prefix].best_name+chr))
                return self._species[prefix].best_name+chr
        return None

    def split_wire_id(self, wire_id: str):
        parts = wire_id.split(":")
        for id in parts:
            if id in self._species:
                species = self._species[id]
                return (species, wire_id)
        raise RequestException("cannot split id")
