import collections
import logging
import re
from typing import Dict, List, Mapping, Tuple
from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from model.bigbed import get_bigbed
from model.chromosome import Chromosome
from model.transcriptfile import TranscriptFileLine
from .util import classified_numbers, starts_and_ends, starts_and_lengths
from .numbers import delta, zigzag, lesqlite2, compress, classify
from .sequence import sequence_blocks
from model.datalocator import AccessItem
from ncd import NCDRead


BLOCKS_PER_PANEL = 1000

# HACK should use correct codes in the first place
def munge_designation(s):
    s = re.sub(r'_',' ',s)
    if s == 'mane select':
        s = "MANE Select"
    elif s == 'canonical':
        s = "Selected"
    return s

def transcript_grade(designation: str, transcript_biotype: str) -> str:
    if designation == "mane_select":
        return 3
    elif designation == "canonical":
        return 2
    elif transcript_biotype == "protein_coding":
        return 1
    else:
        return 0

def add_exon_data(result: dict, genes: List[str], transcripts: Dict[str,TranscriptFileLine]):
    sizes = []
    thick = []
    # below are needed to get it into the correct allotment
    transcript_sizes = []
    exon_counts = []
    for (gene_idx,gene_id) in enumerate(genes):
        line = transcripts[gene_id]
        transcript_sizes.append((line.transcript_start,line.transcript_end))
        thick.append((line.thick_start,line.thick_end))
        exon_counts.append(len(line.block_starts))
        for (start,length) in zip(line.block_starts,line.block_sizes):
            sizes.append((line.transcript_start+start,length))
    starts_and_lengths(result,sizes,"exon")
    starts_and_ends(result,transcript_sizes,"transcript")
    starts_and_ends(result,thick,"thick")
    result['exon_counts'] = compress(lesqlite2(zigzag(delta(exon_counts))))

def extract_gene_data(data_accessor: DataAccessor, chrom: Chromosome, panel: Panel, include_exons: bool, include_sequence: bool) -> Response:
    out = {}
    item = chrom.item_path("transcripts")
    data = get_bigbed(data_accessor,item,panel.start,panel.end)
    seen_genes = set()
    genes = []
    gene_sizes = {}
    transcript_sizes = {}
    gene_names = {}
    gene_descs = {}
    gene_biotypes = {}
    strands = {}
    designated_transcript = collections.defaultdict(lambda: (-1,None))
    transcript_biotypes = {}
    transcript_designations = {}
    for line in data:
        line = TranscriptFileLine(line)
        if line.gene_id not in seen_genes:
            genes.append(line.gene_id)
            seen_genes.add(line.gene_id)
        gene_sizes[line.gene_id] = (line.gene_start,line.gene_end)
        transcript_sizes[line.gene_id] = (line.transcript_start,line.transcript_end)
        gene_names[line.gene_id] = line.gene_name or line.gene_id
        gene_descs[line.gene_id] = line.gene_description
        gene_biotypes[line.gene_id] = line.gene_biotype
        strands[line.gene_id] = line.strand
        transcript_biotypes[line.transcript_id] = line.transcript_biotype
        transcript_designations[line.transcript_id] = line.transcript_designation
        # store candidate designated transcript
        (dt_grade_stored,_) = designated_transcript[line.gene_id]
        dt_grade = transcript_grade(transcript_designations[line.transcript_id],transcript_biotypes[line.transcript_id])
        if dt_grade > dt_grade_stored:
            designated_transcript[line.gene_id] = (dt_grade,line)
    designated_transcript = { k: v[1] for (k,v) in designated_transcript.items() }
    gene_sizes = list([ gene_sizes[gene] for gene in genes ])
    transcript_sizes = list([ transcript_sizes[gene] for gene in genes ])
    gene_names = "\0".join([ gene_names[gene] for gene in genes ])
    gene_descs = "\0".join([ gene_descs[gene] for gene in genes ])
    gene_biotypes = [ gene_biotypes[gene] for gene in genes ]
    gene_designations = [ designated_transcript[gene].transcript_designation for gene in genes ]
    designated_transcript_ids = [ designated_transcript[gene].transcript_id for gene in genes ]
    designated_transcript_biotypes = [ transcript_biotypes[transcript] for transcript in designated_transcript_ids ]
    designated_transcript_designations = [ transcript_designations[transcript] for transcript in designated_transcript_ids ]
    starts_and_ends(out,gene_sizes,None)
    out['gene_names'] = compress(gene_names)
    out['gene_descs'] = compress(gene_descs)
    out['gene_ids'] = compress("\0".join(genes))
    out['focus_ids'] = compress("\0".join([x.split('.')[0] for x in genes]))
    out['designated_transcript_ids'] = compress("\0".join(designated_transcript_ids))
    out['strands'] = compress(lesqlite2([int(x=='+') for x in strands.values()]))
    classified_numbers(out,gene_designations,"gene_designations")
    classified_numbers(out,gene_biotypes,"gene_biotypes")
    classified_numbers(out,designated_transcript_biotypes,"designated_transcript_biotypes")
    classified_numbers(out,designated_transcript_designations,"designated_transcript_designations")
    if include_exons:
        add_exon_data(out,genes,designated_transcript)
    sequence_blocks(out,data_accessor,chrom,panel,not include_sequence)
    #for (k,v) in out.items():
    #    logging.warn("len({}) = {}".format(k,len(v)))
    return Response(5,{ 'data': out })

def extract_gene_overview_data(data_accessor: DataAccessor, chrom: Chromosome, start: int, end: int, with_ids: bool) -> Response:
    out = {}
    item = chrom.item_path("transcripts")
    data = get_bigbed(data_accessor,item,start,end)
    seen_genes = set()
    genes = []
    gene_sizes = {}
    gene_biotypes = {}
    strands = {}
    for line in data:
        line = TranscriptFileLine(line)
        if line.gene_id not in seen_genes:
            genes.append(line.gene_id)
            seen_genes.add(line.gene_id)
        gene_sizes[line.gene_id] = (line.gene_start,line.gene_end)
        gene_biotypes[line.gene_id] = line.gene_biotype
        strands[line.gene_id] = line.strand
        # store candidate designated transcript
    gene_sizes = list([ gene_sizes[gene] for gene in genes ])
    gene_biotypes = [ gene_biotypes[gene] for gene in genes ]
    (gene_biotypes_keys,gene_biotypes_values) = classify(gene_biotypes)
    min_width = 0 #int((panel.end - panel.start) / BLOCKS_PER_PANEL)
    out['starts'] = compress(lesqlite2(zigzag(delta([ x[0] for x in gene_sizes ]))))
    out['lengths'] = compress(lesqlite2(zigzag(delta([ max(x[1]-x[0],min_width) for x in gene_sizes ]))))
    out['strands'] = compress(lesqlite2([int(x=='+') for x in strands.values()]))
    out['gene_biotypes_keys'] = compress("\0".join(gene_biotypes_keys))
    out['gene_biotypes_values'] = compress(lesqlite2(gene_biotypes_values))
    if with_ids:
        out['focus_ids'] = compress("\0".join([x.split('.')[0] for x in genes]))
    return out

class TranscriptDataHandler(DataHandler):
    def __init__(self, seq: bool):
        self._seq = seq

    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return extract_gene_data(data_accessor,chrom,panel,True,self._seq)

class GeneDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return extract_gene_data(data_accessor,chrom,panel,False,False)

class GeneOverviewDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel,scope) -> Response:
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        out = extract_gene_overview_data(data_accessor,chrom,panel.start,panel.end,False)
        return Response(5,{ 'data': out })


def _get_approx_location(data_accessor: DataAccessor, panel: Panel, id):
    genome = panel.stick.rsplit(":",1)[0]
    key = "focus:{}:{}".format(genome,id)
    accessor = data_accessor.resolver.get(AccessItem("jump"))
    jump_ncd = NCDRead(accessor.ncd())
    value = jump_ncd.get(key.encode("utf-8"))
    if value != None:
        parts = value.decode('utf-8').split("\t")
        if len(parts) == 3:
            on_stick = "{}:{}".format(genome,parts[0])
            if on_stick == panel.stick:
                return (int(parts[1]),int(parts[2]))
    return None

def _remove_version(id: str):
    return id.rsplit('.',1)[0]

def _get_exact_location(data_accessor: DataAccessor, panel, gene_id, approx_start, approx_end):
    chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
    if chrom == None:
        return Response(1,"Unknown chromosome {0}".format(panel.stick))
    item = chrom.item_path("transcripts")
    data = get_bigbed(data_accessor,item,approx_start,approx_end)
    for line in data:
        line = TranscriptFileLine(line)
        line_gene_id = _remove_version(line.gene_id)
        if line_gene_id == gene_id:
            return (line.gene_start,line.gene_end,1 if line.strand == '+' else 0)
    return None

class GeneLocationHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel,scope) -> Response:
        id = scope.get("id",[])
        out = []
        location = None
        if len(id) > 0:
            approx = _get_approx_location(data_accessor,panel,id[0])
            if approx is not None:
                exact = _get_exact_location(data_accessor,panel,id[0],approx[0],approx[1])
                if exact is not None:
                    location = exact
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if location is not None:
            out = extract_gene_overview_data(data_accessor,chrom,exact[0],exact[1],True)
        else:
            out = extract_gene_overview_data(data_accessor,chrom,0,0,True)
            location = []
        out["location"] = compress(lesqlite2(location))
        return Response(5,{ 'data': out })
