#! /usr/bin/env python3

import os
import dbm
import os.path

data_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)),"data")
out_file = os.path.join(data_dir,"jump")
genes_dir = os.path.join(data_dir,"genes_and_transcripts")
out= dbm.open(out_file,'c')
for species_part in os.listdir(genes_dir):
    print(species_part)
    bed_path = os.path.join(genes_dir,species_part,"transcripts.bed")
    if os.path.exists(bed_path):
        with open(bed_path) as f:
            for line in f.readlines():
                parts = line.split("\t")
                ids = [parts[3].split(".")[0],parts[14].split(".")[0]]
                stick = "{}:{}".format(species_part,parts[0])
                stick = stick.replace('.','_')
                left = int(parts[1])
                right = int(parts[2])
                pad = (right-left)/2
                left = left - pad
                right = right + pad
                for id in ids:
                    # print(id)
                    out["focus:{}".format(id)] = "\t".join([stick,str(left),str(right)])
out.close()
