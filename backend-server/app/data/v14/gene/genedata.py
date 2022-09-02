import os.path
import collections
import logging
from typing import Dict, List, Optional, Tuple
from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from command.datacmd import DataException
from model.bigbed import get_bigbed
from model.chromosome import Chromosome
from model.transcriptfile import TranscriptFileLine
from ..sequence import sequence_blocks8
from .transcriptorder import sort_data_by_transcript_priority
from .transcriptfilter import filter_lines_by_criteria
from tangle.tangle import TangleFactory
from model.datalocator import AccessItem
from ncd import NCDRead

# We might be asked for very zoomed-in views even when zoomed out for example if we are zoomed
# in and want info on a whole transcipt object. Even in this case we don't want to emit very big
# data sets such as sequences.
MAX_SEQ_SCALE = 10

def accept_to_tangling_config(accept):
    compress = True
    to_bytes = True
    if accept == "dump":
        compress = False
        to_bytes = False
    elif accept == "uncompressed":
        compress = False
    return { 'compress': compress, 'to_bytes': to_bytes }

class TangleProcessor:
    def before_first_dot(self, data):
        return data.split('.')[0]

    def plus_strand(self, data):
        return int(data=="+")

processor = TangleProcessor()

TANGLE_FACTORY = TangleFactory()

TR_TANGLE_PATH = os.path.join(os.path.dirname(__file__),"transcript-tangle.toml")
TANGLE_NO_EXON = TANGLE_FACTORY.make_from_tomlfile(TR_TANGLE_PATH,[],processor)
TANGLE_EXON = TANGLE_FACTORY.make_from_tomlfile(TR_TANGLE_PATH,["exon"],processor)

OV_TANGLE_PATH = os.path.join(os.path.dirname(__file__),"overview-tangle.toml")
TANGLE_OVERVIEW = TANGLE_FACTORY.make_from_tomlfile(OV_TANGLE_PATH,[],processor)
TANGLE_OVERVIEW_WITH_IDS = TANGLE_FACTORY.make_from_tomlfile(OV_TANGLE_PATH,["ids"],processor)

def get_approx_location(data_accessor: DataAccessor, genome: str, id):
    key = "focus:{}:{}".format(genome,id)
    accessor = data_accessor.resolver.get(AccessItem("jump"))
    jump_ncd = NCDRead(accessor.ncd())
    value = jump_ncd.get(key.encode("utf-8"))
    if value != None:
        parts = value.decode('utf-8').split("\t")
        if len(parts) == 3:
            on_stick = "{}:{}".format(genome,parts[0])
            return (on_stick,int(parts[1]),int(parts[2]))
    return (None,None,None)

# We need to return all the data for the focus gene wherever we are (except for the sequence) as
# transcript configuration, ordering, etc is still relevant.
def update_panel_from_id(data_accessor: DataAccessor, panel: Panel, for_id: Tuple[str,str]):
    (stick,start,end) = get_approx_location(data_accessor,for_id[0],for_id[1])
    if stick is not None:
        panel.stick = stick
        panel.start = start
        panel.end = end

# For non-focus genes we need to make
# sure we include all the transcripts even ones which start&end completely off-panel.

def extract_data_for_lines(out, data, for_id: Optional[Tuple[str,str]], include_exons: bool, accept: str) -> Response:
    lines = [ TranscriptFileLine(row) for row in data ]

    # sort the data
    lines = sort_data_by_transcript_priority(lines)
    max_tr = 5 if for_id is None else None

    # filter the data
    lines = filter_lines_by_criteria(lines,for_id,max_tr)

    # serialize the data
    tangle = TANGLE_EXON if include_exons else TANGLE_NO_EXON
    tangle.run(out,{ "tr_bigbed": lines },**accept_to_tangling_config(accept))

def extract_gene_data(data_accessor: DataAccessor, panel: Panel, include_exons: bool, include_sequence: bool, for_id: Optional[Tuple[str,str]], accept: str) -> Response:
    # sequence data (only applies to requested region)
    out = {}
    chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
    if chrom == None:
        raise DataException("Unknown chromosome {0}".format(panel.stick))
    sequence_blocks8(out,data_accessor,chrom,panel,not include_sequence)
    # other data needs to be id-based because it's used to drive metadata, etc
    # fix location
    if for_id is not None:
        update_panel_from_id(data_accessor,panel,for_id)
    chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
    if chrom == None:
        raise DataException("Unknown chromosome {0}".format(panel.stick))
    # get the data
    item = chrom.item_path("transcripts")
    data = get_bigbed(data_accessor,item,panel.start,panel.end)
    extract_data_for_lines(out,data,for_id,include_exons,accept)
    return out

def extract_gene_overview_data(data_accessor: DataAccessor, chrom: Chromosome, start: int, end: int, with_ids: bool, accept: str) -> Response:
    item = chrom.item_path("transcripts")
    data = get_bigbed(data_accessor,item,start,end)
    lines = [ TranscriptFileLine(x) for x in data ]
    out = {}
    tangle = TANGLE_OVERVIEW_WITH_IDS if with_ids else TANGLE_OVERVIEW
    tangle.run(out,{ "tr_bigbed": lines },**accept_to_tangling_config(accept))
    return out

def for_id(scope):
    genome_id = scope.get("genome")
    if genome_id is not None and len(genome_id) == 0:
        genome_id = None
    obj_id = scope.get("id")
    if obj_id is not None and len(obj_id) == 0:
        obj_id = None
    if genome_id is not None and obj_id is not None:
        return (genome_id[0],obj_id[0])
    else:
        return None

class TranscriptDataHandler(DataHandler):
    def __init__(self, seq: bool):
        self._seq = seq

    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope, accept) -> Response:
        include_seq = self._seq
        if panel.scale > MAX_SEQ_SCALE:
            include_seq = False
        return extract_gene_data(data_accessor,panel,True,include_seq,for_id(scope),accept)

class GeneDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope, accept) -> Response:
        return extract_gene_data(data_accessor,panel,False,False,for_id(scope),accept)

class GeneOverviewDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel,scope, accept) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        out = extract_gene_overview_data(data_accessor,chrom,panel.start,panel.end,False,accept)
        return out
