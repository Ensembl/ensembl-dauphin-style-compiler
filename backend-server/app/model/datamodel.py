import logging, toml
from .species import Species
from util.string import split_all
from model.datalocator import AccessItem
from core.config import SPECIESLIST_TOML
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
        self._species = {}
        self._species_aliases = {}
        self._load_species()

    def stick(self, data_accessor, alias):
        out = self.try_stick(data_accessor,alias)
        if out is None:
            raise RequestException("cannot find stick")
        return out

    def try_stick(self, data_accessor, alias):
        for (prefix, _) in split_all(":", alias):
            species_name = self._species_aliases.get(prefix)
            if species_name is not None:
                return self._species[species_name].chromosome(data_accessor, alias)
        return None

    def species(self, alias):
        species_name = self._species_aliases.get(alias)
        if species_name is not None:
            return self._species[species_name]
        else:
            return None        

    def canonical_genome_id(self, alias):
        for (prefix, chr) in split_all(":", alias):
            species_name = self._species_aliases.get(prefix)
            if species_name is not None:
                return species_name
        return None

    def best_stick_id(self, alias):
        for (prefix, chr) in split_all(":", alias):
            species_name = self._species_aliases.get(prefix)
            if species_name is not None:
                return self._species[species_name].best_name+chr
        return None

    def _load_species(self):
        with open(SPECIESLIST_TOML) as f:
            species_list = toml.loads(f.read())
            for species_name in species_list["species"]:
                species_conf = species_list["species"][species_name]
                print(species_conf)
                all_names = list(species_conf["other_names"])
                all_names.append(species_conf["best_name"])
                tags = set(species_conf.get("tags",[]))
                try:
                    species_object = Species(species_conf["path"],species_conf['best_name'],all_names,tags)
                    self._species[species_object.wire_id] = species_object
                    for alias_prefix in all_names:
                        self._species_aliases[alias_prefix] = species_object.wire_id
                except:
                    logging.error("Species {0} failed to configure. Skipping!".format(species_name))

    def split_total_wire_id(self, total_wire_id: str):
        # we know that we split on a colon, but which one? We go from longest to shortest, trying
        # all combinations of positions, :-( .
        parts = total_wire_id.split(":")
        for num in reversed(range(1,len(parts)+1)):
            for start in range(0,len(parts)-num+1):
                species = ":".join(parts[start:start+num])
                species_name = self._species_aliases.get(species,None)
                if species_name is not None:
                    species = self._species[species_name]
                    out = parts[:start] + [species.wire_id] + parts[start+num:]
                    return (species,":".join(out))
        raise RequestException("cannot split id")
