from .chromosome import Chromosome
from model.datalocator import AccessItem
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
        self._chromosome_sizes: dict[str, int] | None = None

    def _load_metadata(self, data_accessor, variety, chr, missing_ok=False):
        item = AccessItem(variety, genome=self.genome_id, chromosome=chr)
        accessor = data_accessor.resolver.get(item)
        if accessor is None:
            raise RequestException("cannot resolve chromosome checksum metadata")
        checksum = accessor.get_checksum()
        if not checksum:
            if missing_ok:
                return None
            else:
                raise RequestException("cannot find checksum '{}'".format(chr))
        return checksum

    def _load_chromosome_sizes(self, data_accessor) -> dict[str, int]:
        if self._chromosome_sizes is None:
            item = AccessItem("chrom-sizes", genome=self.genome_id)
            accessor = data_accessor.resolver.get(item)
            if accessor is None:
                raise RequestException("cannot resolve chromosome sizes metadata")
            self._chromosome_sizes = accessor.get_karyotype()
        return self._chromosome_sizes

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
            size = self._load_chromosome_sizes(data_accessor).get(chr)
            if size is None:
                raise RequestException("cannot find chromosome size '{}'".format(chr))
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
