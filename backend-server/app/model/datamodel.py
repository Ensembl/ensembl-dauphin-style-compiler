import logging
import os.path
import yaml
from core.config import DATA_FILES
from .species import Species
from util.string import split_all

class DataModel(object):
    def __init__(self):
        self.files_dir = DATA_FILES
        self._species = {}
        self._species_aliases = {}
        self._load_species()

    def stick(self, data_accessor, alias):
        for (prefix,_) in split_all(":",alias):
            species_name = self._species_aliases.get(prefix)
            if species_name != None:
                return self._species[species_name].chromosome(data_accessor,alias)
        return None

    def _species_config(self):
        genome_info_path = os.path.join(self.files_dir,"common_files","genome_id_info.yml")
        with open(genome_info_path) as f:
            return yaml.load(f,Loader=yaml.Loader)

    def _load_species(self):
        species = self._species_config()
        for (species_name,species_data) in species.items():
            try:
                species_object = Species(self.files_dir,species_name,species_data)
                self._species[species_object.wire_id] = species_object
                for alias_prefix in species_object.alias_prefixes:
                    self._species_aliases[alias_prefix] = species_object.wire_id
            except:
                logging.error("Species {0} failed to configure. Skipping!".format(species_name))
