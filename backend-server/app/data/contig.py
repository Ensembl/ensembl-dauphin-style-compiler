import collections
import logging
import random
from typing import List, Tuple
from command.coremodel import DataHandler, Panel, DataAccessor, Response
from model.bigbed import get_bigbed_data, get_bigwig_data
from model.chromosome import Chromosome
from model.transcriptfile import TranscriptFileLine
from .numbers import delta, zigzag, lesqlite2, compress, classify
from .util import starts_and_ends

DOMINO_COUNT = 200

# Shimmering is the process of making sure we give similar weight to two senses present in a track when so far out
# that they occupy less than one pixel. To achieve this, the panel is divided into a large number of equal-sized 
# "dominoes", the number chosen such that they occupy at least a couple of pixels, but not much more. The base-pairs
# corresponding to each domino are examined for presence and absence throughout. If both are present, a domino is
# chosen with half set and half not set (which half is random). If only one is present, the whole domino is that
# colour. Without applying dominoing either one colour dominates over the other misleadingly or the track degenerates
# into a misleadingly uniform intermediate colour, hiding the variety of densities within.
def shimmer_push(out_positions: List[List[int]], out_sense: List[bool], start: int, end: int,sense: bool):
    if len(out_sense) > 0 and sense == out_sense[-1] and out_positions[-1][1] == start:
        out_positions[-1][1] = end
    else:
        out_positions.append([start,end])
        out_sense.append(sense)

def shimmer(positions: List[Tuple[int]], sense: List[bool], start: int, end: int) -> Tuple[List[Tuple[int,int]],List[int]]:
    domino_bp = (end-start)/DOMINO_COUNT
    domino_onoff = [0] * DOMINO_COUNT
    for ((start_p,end_p),sense) in zip(positions,sense):
        start_d = int((start_p-start)/domino_bp)
        end_d = int((end_p-start)/domino_bp)
        for domino in range(start_d,end_d+1):
            domino_onoff[domino] |= (2 if sense else 1)
    out_position = []
    out_sense = []
    for domino in range(0,DOMINO_COUNT):
        start_d = start + domino * domino_bp
        end_d = start_d + domino_bp
        if domino_onoff[domino] == 3:
            flip = random.randint(0,1) != 0
            shimmer_push(out_position,out_sense,start_d,(start_d+end_d)/2.0,flip)
            shimmer_push(out_position,out_sense,(start_d+end_d)/2.0,end_d,not flip)
        elif domino_onoff[domino] == 2:
            shimmer_push(out_position,out_sense,start_d,end_d,True)
        elif domino_onoff[domino] == 1:
            shimmer_push(out_position,out_sense,start_d,end_d,False)
    return (out_position,out_sense)

def get_contig(chrom: Chromosome, panel: Panel, do_shimmer: bool) -> Response:
    path = chrom.file_path("contigs","contigs.bb")
    data = get_bigbed_data(path,chrom,panel.start,panel.end)
    positions = []
    senses = []
    for line in data:
        (contig_start, contig_end, rest) = line
        (name, value, sense) = rest.split("\t")
        positions.append((contig_start,contig_end))
        senses.append(sense=='+')
    if do_shimmer:
        (positions,senses) = shimmer(positions,senses,panel.start,panel.end)
    out = {
        "sense": compress(lesqlite2(senses))
    }
    starts_and_ends(out,positions,"contig")
    return Response(5,{ 'data': out })

class ContigDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel) -> Response:
        chrom = data_accessor.data_model.sticks.get(panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return get_contig(chrom,panel,False)

class ShimmerContigDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel) -> Response:
        chrom = data_accessor.data_model.sticks.get(panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return get_contig(chrom,panel,True)