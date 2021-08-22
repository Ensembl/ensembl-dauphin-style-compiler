import re
import os.path
from .chromosome import Chromosome
import logging
from model.datalocator import AccessItem
from ncd import NCDRead
from core.exceptions import RequestException

class Species(object):
    def __init__(self, files_dir, species_name, species_data):
        for (k,v) in species_data.items():
            # make contents of config hash available as attirubtes
            setattr(self,k,species_data[k])
        self.genome_path = self.genome_id
        self.wire_id = re.sub(r'\W','_',self.genome_id)
        self.files_dir = files_dir
        self.chromosomes = {}
        self.alias_prefixes = [self.wire_id]

    def _load_ncd(self,data_accessor,variety,wire_id):
        item = AccessItem(variety,self.genome_id)
        accessor = data_accessor.resolver.get(item)
        hash_reader = NCDRead(accessor.ncd())
        hash_data = hash_reader.get(wire_id.encode("utf-8"))
        if hash_data == None:
            raise RequestException("cannot find hash")
        return hash_data.decode("utf-8").split("\t")

    def _load_chromosome(self, data_accessor, total_wire_id):
        wire_id = total_wire_id[(len(self.wire_id)+1):]
        hash_value = self._load_ncd(data_accessor,"chrom-hashes",wire_id)[0]
        size = int(self._load_ncd(data_accessor,"chrom-sizes",wire_id)[0])
        return Chromosome(self.files_dir,wire_id,size,hash_value,self)

    def chromosome(self, data_accessor, wire_id):
        if not (wire_id in self.chromosomes):
            self.chromosomes[wire_id] = self._load_chromosome(data_accessor,wire_id)
        return self.chromosomes.get(wire_id)

    def file_path(self,section,filename):
        path = os.path.join(self.files_dir,section,self.genome_path,filename)
        if not os.path.exists(path):
            logging.warn("Missing file {0}".format(path))
        return path

    def item_path(self,variety):
        return AccessItem(variety,self.genome_id)
