import os.path
import collections
import logging
from typing import Dict, List, Optional
from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from model.bigbed import get_bigbed
from model.chromosome import Chromosome
from model.transcriptfile import TranscriptFileLine
from ..sequence import sequence_blocks8
from .transcriptorder import sort_data_by_transcript_priority
from .transcriptfilter import filter_lines_by_criteria
from tangle.tangle import TangleFactory
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

def extract_gene_data(data_accessor: DataAccessor, chrom: Chromosome, panel: Panel, include_exons: bool, include_sequence: bool, for_id: Optional[str], accept: str) -> Response:
    # get the data
    item = chrom.item_path("transcripts")
    data = get_bigbed(data_accessor,item,panel.start,panel.end)
    lines = [ TranscriptFileLine(row) for row in data ]

    # sort the data
    lines = sort_data_by_transcript_priority(lines)
    max_tr = 5 if for_id is None else None

    # filter the data
    lines = filter_lines_by_criteria(lines,for_id,max_tr)

    # serialize the data
    tangle = TANGLE_EXON if include_exons else TANGLE_NO_EXON
    out = {}
    tangle.run(out,{ "tr_bigbed": lines },**accept_to_tangling_config(accept))
    sequence_blocks8(out,data_accessor,chrom,panel,not include_sequence)
    return out

OV_TANGLE_PATH = os.path.join(os.path.dirname(__file__),"overview-tangle.toml")
TANGLE_OVERVIEW = TANGLE_FACTORY.make_from_tomlfile(OV_TANGLE_PATH,[],processor)
TANGLE_OVERVIEW_WITH_IDS = TANGLE_FACTORY.make_from_tomlfile(OV_TANGLE_PATH,["ids"],processor)

def extract_gene_overview_data(data_accessor: DataAccessor, chrom: Chromosome, start: int, end: int, with_ids: bool, accept: str) -> Response:
    item = chrom.item_path("transcripts")
    data = get_bigbed(data_accessor,item,start,end)
    lines = [ TranscriptFileLine(x) for x in data ]
    out = {}
    tangle = TANGLE_OVERVIEW_WITH_IDS if with_ids else TANGLE_OVERVIEW
    tangle.run(out,{ "tr_bigbed": lines },**accept_to_tangling_config(accept))
    return out

def for_id(scope):
    id_seq = scope.get("id")
    if id_seq is not None and len(id_seq) > 0:
        return id_seq[0]
    else:
        return None

class TranscriptDataHandler(DataHandler):
    def __init__(self, seq: bool):
        self._seq = seq

    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope, accept) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        include_seq = self._seq
        if panel.scale > MAX_SEQ_SCALE:
            include_seq = False
        return extract_gene_data(data_accessor,chrom,panel,True,include_seq,for_id(scope),accept)

class GeneDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope, accept) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return extract_gene_data(data_accessor,chrom,panel,False,False,for_id(scope),accept)

class GeneOverviewDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel,scope, accept) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        out = extract_gene_overview_data(data_accessor,chrom,panel.start,panel.end,False,accept)
        return out
