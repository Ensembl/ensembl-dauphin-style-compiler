from uuid import UUID
from .species import Species
from core.exceptions import RequestException
"""
Converts a stick id to a Chromosome() object which includes means of access for the data.
The main tasks are resolving species IDs and determining data-sets based on versioning.
New Species() objects (and genome UUIDs) are added to the cache as they are requested.
"""
class DataModel(object):
    """
    Args:
        data_accessor ():
    """
    def __init__(self, data_accessor) -> None:
        self._species: dict[str, Species] = {}
        self._data_accessor = data_accessor

    # Args: stick string (<genome_uuid>:<chr>)
    # Returns: Chromosome instance
    def stick(self, data_accessor, stick_id:str):
        genome_id = stick_id.split(":")[0]
        species = self.species(genome_id)
        return species.chromosome(self._data_accessor, stick_id)

    def species(self, genome_id:str):
        if genome_id not in self._species:
            try:
                UUID(genome_id)
            except ValueError:
                raise RequestException(
                    f"Unexpected genome id format: {genome_id}")
            self._species[genome_id] = Species(genome_id)

        return self._species[genome_id]
