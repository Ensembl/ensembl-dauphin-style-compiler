class Chromosome(object):
    def __init__(self, name, size, seq_hash, species):
        self.name = name
        self.size = size
        self.topology = "linear"
        self.tags = set(["local"])
        self.seq_hash = seq_hash
        self.genome_id = species.genome_id
        self.stick_name = "{0}:{1}".format(
            species.wire_id,self.name
        )
        self.genome_path = species.genome_id
        self.aliases = []
        # HACK this hack must die: get the data correct!
        if self.name.startswith("chr"):
            self.aliases.append("{0}:{1}".format(
                species.wire_id,self.name[3:]
            ))
