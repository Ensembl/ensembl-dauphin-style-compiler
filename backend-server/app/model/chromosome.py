import logging
import os.path
from model.datalocator import AccessItem


class Chromosome(object):
    """

    Args:
        name ():
        size ():
        seq_hash ():
        species ():
    """
    def __init__(self, name, size, seq_hash, species, tags):
        self.name = name
        self.size = size
        self.topology = "linear"
        self.tags = tags
        self.seq_hash = seq_hash
        self.genome_id = species.genome_id
        self.stick_name = "{0}:{1}".format(
            species.genome_id, self.name
        )

    def item_path(self, variety):
        """

        Args:
            variety ():

        Returns:
            AccessItem:
        """
        return AccessItem(variety, self.genome_id, self.name)

    def item_seq_path(self, variety):
        """

        Args:
            variety ():

        Returns:
            AccessItem:
        """
        return AccessItem(variety, self.genome_id, self.seq_hash)
