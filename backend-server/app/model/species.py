import re
import os.path
from .chromosome import Chromosome
import logging
from model.datalocator import AccessItem

class Species(object):
    def __init__(self, files_dir, species_name, species_data):
        for (k,v) in species_data.items():
            # make contents of config hash available as attirubtes
            setattr(self,k,species_data[k])
        self.genome_path = self.genome_id
        self.wire_id = re.sub(r'\W','_',self.genome_id)
        self.files_dir = files_dir
        hashes = self._load_hashes(files_dir)
        self.chromosomes = {}
        self._load_chromosomes(files_dir,hashes)

    def _load_hashes(self, files_dir):
        hashes = {}
        with open(os.path.join(files_dir,"common_files",self.genome_path,"chrom.hashes")) as f:
            for line in f.readlines():
                parts = line.strip().split("\t")
                hashes[parts[0]] = parts[1]
        return hashes

    def _load_chromosomes(self, files_dir, hashes):
        with open(os.path.join(files_dir,"common_files",self.genome_path,"chrom.sizes")) as f:
            for line in f.readlines():
                parts = line.strip().split("\t")
                seq_hash = hashes[parts[0]]
                chrom = Chromosome(files_dir,parts[0],int(parts[1]),seq_hash,self)
                self.chromosomes[chrom.name] = chrom

    def file_path(self,section,filename):
        path = os.path.join(self.files_dir,section,self.genome_path,filename)
        if not os.path.exists(path):
            logging.warn("Missing file {0}".format(path))
        return path

    def item_path(self,variety):
        return AccessItem(variety,self.genome_id)
