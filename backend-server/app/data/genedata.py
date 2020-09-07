import collections
import logging
from command.coremodel import DataHandler, Panel, DataAccessor, Response
from model.bigbed import get_bigbed_data
from model.chromosome import Chromosome
from model.transcriptfile import TranscriptFileLine
from .numbers import delta, zigzag, lesqlite2, compress, classify

# HACK should use correct codes in the first place
def munge_designation(s):
    s = re.sub(r'_',' ',s)
    if s == 'mane select':
        s = "MANE Select"
    elif s == 'canonical':
        s = "Selected"
    return s

def transcript_grade(designation: str, transcript_biotype: str) -> str:
    if designation == "MANE Select":
        return 3
    elif designation == "Selected":
        return 2
    elif transcript_biotype == "protein_coding":
        return 1
    else:
        return 0

def extract_gene_data(chrom: Chromosome, panel: Panel) -> Response:
    out = {}
    path = chrom.file_path("genes_and_transcripts","transcripts.bb")
    logging.warn("hello from gene panel = {0} path = {1} {2}-{3}".format(str(vars(panel)),path,panel.start,panel.end))
    data = get_bigbed_data(path,chrom,panel.start,panel.end)
    seen_genes = set()
    genes = []
    gene_sizes = {}
    gene_names = {}
    gene_descs = {}
    gene_biotypes = {}
    strands = {}
    designated_transcript = collections.defaultdict(lambda: (-1,None))
    for line in data:
        line = TranscriptFileLine(line)
        if line.gene_id not in seen_genes:
            genes.append(line.gene_id)
            seen_genes.add(line.gene_id)
        gene_sizes[line.gene_id] = (line.gene_start,line.gene_end)
        gene_names[line.gene_id] = line.gene_name or line.gene_id
        gene_descs[line.gene_id] = line.gene_description
        gene_biotypes[line.gene_id] = line.gene_biotype
        strands[line.gene_id] = line.strand
        # store candidate designated transcript
        (dt_grade_stored,_) = designated_transcript[line.gene_id]
        dt_grade = transcript_grade(line.transcript_designation,line.transcript_biotype)
        if dt_grade > dt_grade_stored:
            designated_transcript[line.gene_id] = (dt_grade,line)
    gene_sizes = list([ gene_sizes[gene] for gene in genes ])
    gene_names = "\0".join([ gene_names[gene] for gene in genes ])
    gene_descs = "\0".join([ gene_descs[gene] for gene in genes ])
    gene_designations = [ designated_transcript[gene][1].transcript_designation for gene in genes ]
    designated_transcript_ids = [ designated_transcript[gene][1].transcript_id for gene in genes ]
    (gene_designations_keys,gene_designations_values) = classify(gene_designations)
    (gene_biotypes_keys,gene_biotypes_values) = classify(gene_biotypes)
    out['starts'] = compress(lesqlite2(zigzag(delta([ x[0] for x in gene_sizes ]))))
    out['lengths'] = compress(lesqlite2(zigzag(delta([ x[1]-x[0] for x in gene_sizes ]))))
    out['gene_names'] = compress(gene_names)
    out['gene_descs'] = compress(gene_descs)
    out['gene_ids'] = compress("\0".join(genes))
    out['designated_transcript_ids'] = compress("\0".join(designated_transcript_ids))
    out['strands'] = compress(lesqlite2([x=='+' for x in strands.values()]))
    out['gene_designations_keys'] = compress("\0".join(gene_designations_keys))
    out['gene_designations_values'] = compress(lesqlite2(gene_designations_values))
    out['gene_biotypes_keys'] = compress("\0".join(gene_biotypes_keys))
    out['gene_biotypes_values'] = compress(lesqlite2(gene_biotypes_values))
    logging.warn("got {0} genes".format(len(genes)))
    for (k,v) in out.items():
        logging.warn("len({0}) = {1}".format(k,len(v)))
    return Response(5,{ 'data': out })


class GeneDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel) -> Response:
        chrom = data_accessor.data_model.sticks.get(panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return extract_gene_data(chrom,panel)