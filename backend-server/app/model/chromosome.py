import logging
import os.path
from model.datalocator import AccessItem


def chrless(x):
    """

    Args:
        x ():

    Returns:
        list
    """
    if x.startswith("chr"):
        return x[3:]
    else:
        return x


class Chromosome(object):
    """

    Args:
        name ():
        size ():
        seq_hash ():
        species ():
    """
    def __init__(self, name, size, seq_hash, species):
        self.name = name
        self.size = size
        self.topology = "linear"
        self.tags = set(["local"])
        self.seq_hash = seq_hash
        self.genome_id = species.genome_id
        self.stick_name = "{0}:{1}".format(
            species.wire_id, self.name
        )
        self.genome_path = species.genome_id
        self.wire_id = chrless(self.name)

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
