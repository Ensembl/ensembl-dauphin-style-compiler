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

    def __init__(self, genome_id):
        self.genome_id = genome_id
        self.chromosomes = {}
        self._tags = []

    def item_path(self, variety):
        """

        Args:
            variety ():

        Returns:
            AccessItem:
        """
        return AccessItem(variety, self.genome_id)

    def _load_metadata(self, data_accessor, variety, chr, missing_ok=False):
        item = AccessItem(variety, genome=self.genome_id, chromosome=chr)
        checksum = data_accessor.resolver.get(item).get_checksum()
        if not checksum:
            if missing_ok:
                return None
            else:
                raise RequestException("cannot find checksum '{}'".format(chr))
        return checksum
        pass

    def _load_ncd(self, data_accessor, variety, chr, missing_ok=False):
        """

        Args:
            data_accessor ():
            variety ():
            chr ():

        Returns:

        """
        item = AccessItem(variety, genome=self.genome_id)
        accessor = data_accessor.resolver.get(item)
        hash_reader = NCDRead(accessor.ncd())
        hash_data = hash_reader.get(chr.encode("utf-8"))
        if not hash_data:
            if missing_ok:
                return None
            else:
                raise RequestException("cannot find hash '{}'".format(chr))
        return hash_data.decode("utf-8").split("\t")

    def _load_chromosome(self, data_accessor, stick: str):
        """

        Args:
            data_accessor ():
            stick (str): <genome_uuid>:<chr>

        Returns:

        """
        (genome, chr) = stick.split(':')
        hash_value = self._load_metadata(data_accessor, "chrom-hashes", chr, missing_ok=True)
        if hash_value is not None:
            size = int(self._load_ncd(data_accessor, "chrom-sizes", chr)[0])
            return Chromosome(chr, size, hash_value, self, self._tags)
        else:
            return None

    def chromosome(self, data_accessor, stick: str):
        """

        Args:
            data_accessor ():
            stick (str): stick <genome_uuid>:<chr>

        Returns:
            chromosome():

        """
        if not (stick in self.chromosomes):
            self.chromosomes[stick] = self._load_chromosome(data_accessor, stick)
        return self.chromosomes.get(stick)
