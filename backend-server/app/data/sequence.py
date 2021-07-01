import logging
from typing import Dict
from command.coremodel import Panel
from model.chromosome import Chromosome
from .numbers import delta, zigzag, lesqlite2, compress, classify
from .util import classified_numbers

def sequence_blocks(out: Dict[str,bytes], chrom: Chromosome, panel: Panel, dummy: bool):
    file_path = chrom.file_path("seqs",chrom.seq_hash)
    starts = []
    letters = []
    if not dummy:
        with open(file_path) as f:
            f.seek(panel.start)
            sequence = f.read(panel.end-panel.start)
            for (offset,letter) in enumerate(sequence):
                starts.append(panel.start+offset)
                letters.append(letter if letter in "CGAT" else "")
    out['seq_starts'] = compress(lesqlite2(zigzag(delta(starts))))
    classified_numbers(out,letters,"seq")

