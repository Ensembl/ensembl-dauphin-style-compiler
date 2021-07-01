import logging
import os.path

def chrless(x):
    if x.startswith("chr"):
        return x[3:]
    else:
        return x

class Chromosome(object):
    def __init__(self, files_dir, name, size, seq_hash, species):
        self.name = name
        self.size = size
        self.topology = "linear"
        self.tags = set(["local"])
        self.seq_hash = seq_hash
        self.genome_id = species.genome_id
        self.stick_name = "{0}:{1}".format(
            species.wire_id,self.name
        )
        self.files_dir = files_dir
        self.genome_path = species.genome_id
        self.aliases = []
        # HACK this hack must die: get the data correct!
        self.aliases.append("{0}:{1}".format(species.wire_id,chrless(self.name)))

    def file_path(self,section,filename):
        path = os.path.join(self.files_dir,section,self.genome_path,filename)
        if not os.path.exists(path):
            logging.warn("Missing file {0}".format(path))
        return path
