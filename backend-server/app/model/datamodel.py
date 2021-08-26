import logging
from .species import Species
from util.string import split_all
from model.datalocator import AccessItem

class DataModel(object):
    def __init__(self, data_accessor):
        self._species = {}
        self._species_aliases = {}
        self._load_species(data_accessor)

    def stick(self, data_accessor, alias):
        for (prefix,_) in split_all(":",alias):
            species_name = self._species_aliases.get(prefix)
            if species_name != None:
                return self._species[species_name].chromosome(data_accessor,alias)
        return None

    def _get_species_list(self, resolver):
        item = AccessItem("species-list")
        accessor = resolver.get(item)
        values = accessor.get().decode("utf-8")
        return values.splitlines()

    def _load_species(self, resolver):
        species_list = self._get_species_list(resolver)
        for species_name in species_list:
            try:
                species_object = Species(species_name)
                self._species[species_object.wire_id] = species_object
                for alias_prefix in species_object.alias_prefixes:
                    self._species_aliases[alias_prefix] = species_object.wire_id
            except:
                logging.error("Species {0} failed to configure. Skipping!".format(species_name))
