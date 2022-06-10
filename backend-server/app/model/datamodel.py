import logging
from .species import Species
from util.string import split_all
from model.datalocator import AccessItem


class DataModel(object):
    """
    Args:
        data_accessor ():
    """
    def __init__(self, data_accessor):

        self._species = {}
        self._species_aliases = {}
        self._load_species(data_accessor)

    def stick(self, data_accessor, alias):
        """

        Args:
            data_accessor ():
            alias ():

        Returns:

        """
        for (prefix, _) in split_all(":", alias):
            species_name = self._species_aliases.get(prefix)
            if species_name is not None:
                return self._species[species_name].chromosome(data_accessor, alias)
        return None

    @staticmethod
    def _get_species_list(resolver):
        """

        Args:
            resolver ():

        Returns:
            String
        """
        item = AccessItem("species-list")
        accessor = resolver.get(item)
        values = accessor.get().decode("utf-8")
        return values.splitlines()

    def _load_species(self, resolver):
        """

        Args:
            resolver ():

        Returns:
            None

        """
        species_list = self._get_species_list(resolver)
        for species_name in species_list:
            try:
                species_object = Species(species_name)
                self._species[species_object.wire_id] = species_object
                for alias_prefix in species_object.alias_prefixes:
                    self._species_aliases[alias_prefix] = species_object.wire_id
            except:
                logging.error("Species {0} failed to configure. Skipping!".format(species_name))
