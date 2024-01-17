from uuid import UUID
from .species import Species
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
        if genome_id in self._species:
            return self._species[genome_id]
        else:
            raise RequestException(f"Unknown genome id: {genome_id}")
