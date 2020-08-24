import logging
import os.path
import yaml
from core.config import DATA_FILES
from .species import Species

class DataModel(object):
    def __init__(self):
        self.files_dir = DATA_FILES
        self.species = {}
        self.sticks = {}
        self._load_species()
        self._load_chromosomes()

    def _species_config(self):
        genome_info_path = os.path.join(self.files_dir,"common_files","genome_id_info.yml")
        with open(genome_info_path) as f:
            return yaml.load(f)

    def _load_species(self):
        species = self._species_config()
        for (species_name,species_data) in species.items():
            try:
                species_object = Species(self.files_dir,species_name,species_data)
                self.species[species_object.wire_id] = species_object
            except:
                logging.error("Species {0} failed to configure. Skipping!".format(species_name))

    def _load_chromosomes(self):
        for species in self.species.values():
            for chromosome in species.chromosomes.values():
                self.sticks[chromosome.stick_name] = chromosome
                for alias in chromosome.aliases:
                    self.sticks[alias] = chromosome
