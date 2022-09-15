import logging
import re
from .chromosome import Chromosome
from model.datalocator import AccessItem
from ncd import NCDRead
from core.exceptions import RequestException


class Species(object):
    """

    Args:
        genome_id ():
    """
    def __init__(self, genome_id, best_name, names):
        self.genome_id = genome_id
        self.genome_path = self.genome_id
        self.wire_id = re.sub(r'\W', '_', self.genome_id)
        self.chromosomes = {}
        self.best_name = best_name
        self._names = names
        self.alias_prefixes = [self.wire_id]

    def _load_ncd(self, data_accessor, variety, wire_id):
        """

        Args:
            data_accessor ():
            variety ():
            wire_id ():

        Returns:

        """
        item = AccessItem(variety, self.genome_id)
        accessor = data_accessor.resolver.get(item)
        hash_reader = NCDRead(accessor.ncd())
        hash_data = hash_reader.get(wire_id.encode("utf-8"))
        if hash_data == None:
            raise RequestException("cannot find hash '{}'".format(wire_id))
        return hash_data.decode("utf-8").split("\t")

    def split_total_wire_id(self, total_wire_id: str):
        for name in self._names:
            if total_wire_id.startswith(name+":"):
                return (name,total_wire_id[len(name)+1:])
            elif total_wire_id == name:
                return (name,"")
        raise RequestException("cannot split id")

    def _load_chromosome(self, data_accessor, total_wire_id):
        """

        Args:
            data_accessor ():
            total_wire_id ():

        Returns:

        """
        (_, wire_id) = self.split_total_wire_id(total_wire_id)
        hash_value = self._load_ncd(data_accessor, "chrom-hashes", wire_id)[0]
        size = int(self._load_ncd(data_accessor, "chrom-sizes", wire_id)[0])
        return Chromosome(wire_id, size, hash_value, self)

    def chromosome(self, data_accessor, wire_id):
        """

        Args:
            data_accessor ():
            wire_id ():

        Returns:
            chromosome():

        """
        if not (wire_id in self.chromosomes):
            self.chromosomes[wire_id] = self._load_chromosome(data_accessor, wire_id)
        return self.chromosomes.get(wire_id)

    # def item_path(self, variety):
    #     """

    #     Args:
    #         variety ():

    #     Returns:

    #     """
    #     return AccessItem(variety, self.genome_id)
