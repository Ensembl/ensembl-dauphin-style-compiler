import os.path
import collections
import logging
from typing import Dict, List, Optional
from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from model.bigbed import get_bigbed
from model.chromosome import Chromosome
from model.transcriptfile import TranscriptFileLine
from ..numbers import lesqlite2, compress
from ..sequence8 import sequence_blocks8
from .transcriptorder import sort_data_by_transcript_priority
from .transcriptfilter import filter_lines_by_criteria
from model.datalocator import AccessItem
from tangle.tangle import TangleFactory
from ncd import NCDRead

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

def extract_gene_data(data_accessor: DataAccessor, chrom: Chromosome, panel: Panel, include_exons: bool, include_sequence: bool, for_id: Optional[str]) -> Response:
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
    tangle.run(out,{ "tr_bigbed": lines })
    sequence_blocks8(out,data_accessor,chrom,panel,not include_sequence)
    return Response(5,{ 'data': out })

OV_TANGLE_PATH = os.path.join(os.path.dirname(__file__),"overview-tangle.toml")
TANGLE_OVERVIEW = TANGLE_FACTORY.make_from_tomlfile(OV_TANGLE_PATH,[],processor)
TANGLE_OVERVIEW_WITH_IDS = TANGLE_FACTORY.make_from_tomlfile(OV_TANGLE_PATH,["ids"],processor)

def extract_gene_overview_data(data_accessor: DataAccessor, chrom: Chromosome, start: int, end: int, with_ids: bool) -> Response:
    item = chrom.item_path("transcripts")
    data = get_bigbed(data_accessor,item,start,end)
    lines = [ TranscriptFileLine(x) for x in data ]
    out = {}
    tangle = TANGLE_OVERVIEW_WITH_IDS if with_ids else TANGLE_OVERVIEW
    tangle.run(out,{ "tr_bigbed": lines })
    return out

def for_id(scope):
    id_seq = scope.get("id")
    if id_seq is not None and len(id_seq) > 0:
        return id_seq[0]
    else:
        return None

class TranscriptDataHandler8(DataHandler):
    def __init__(self, seq: bool):
        self._seq = seq

    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return extract_gene_data(data_accessor,chrom,panel,True,self._seq,for_id(scope))

class GeneDataHandler8(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return extract_gene_data(data_accessor,chrom,panel,False,False,for_id(scope))

class GeneOverviewDataHandler8(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel,scope) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        out = extract_gene_overview_data(data_accessor,chrom,panel.start,panel.end,False)
        return Response(5,{ 'data': out })
