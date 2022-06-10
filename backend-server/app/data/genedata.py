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

#
# THIS FILE CAN BE DELETEDWHEN VERSION 7 IS REMOVED
#

"""
Attributes:
    BLOCKS_PER_PANEL (int) Don't care about anything which occupies less space on the screen
    than a 1/BLOCKS_PER_PANEL-th of the screen in overviews. Removes tiny crud that no one
    will see anyway.
"""
BLOCKS_PER_PANEL = 1000

def transcript_grade(designation: str, transcript_biotype: str) -> int:
    """
    Computes a value to order a transcript by based on its properties.

    Args:
        designation (str): MANE/canonical/not etc
        transcript_biotype (str): transcript_biotype

    Returns:
        int: A value where the more prioritised transcripts have a higher value than
        the less during this request. No other guarantees about the stability/meaning of the 
        number beyond that.
    """
    if designation == "mane_select":
        return 3
    elif designation == "canonical":
        return 2
    elif transcript_biotype == "protein_coding":
        return 1
    else:
        return 0


def add_exon_data(result: dict, genes: List[str], transcripts: Dict[str, TranscriptFileLine]):
    """
    Called by extract_gene_data to add exon data to the output object, if include_exons is
    set. Note that this method (and file!) is only used prior to multiple transcripts being
    implemented, so there is a 1-to-1 mapping for genes and transcripts.

    Args:
        result (dict): Dict to add the exon data to (output variable)
        genes (List[str]): list of genes we want to include data for
        transcripts (Dict[str, TranscriptFileLine]): the line in the data file we want to use for this gene.

    Returns:
        Nothing. Updates result object.

    """
    sizes = []
    thick = []
    # below are needed to get it into the correct allotment
    gene_biotypes = []
    strands = []
    transcript_sizes = []
    exon_counts = []
    for (gene_idx, gene_id) in enumerate(genes):
        line = transcripts[gene_id]
        transcript_sizes.append((line.transcript_start, line.transcript_end))
        thick.append((line.thick_start, line.thick_end))
        exon_counts.append(len(line.block_starts))
        for (start, length) in zip(line.block_starts, line.block_sizes):
            sizes.append((line.transcript_start + start, length))
            gene_biotypes.append(line.gene_biotype)
            strands.append(line.strand == '+')
    starts_and_lengths(result, sizes, "exon")
    starts_and_ends(result, transcript_sizes, "transcript")
    starts_and_ends(result, thick, "thick")
    classified_numbers(result, gene_biotypes, "exon_gene_biotypes")
    result['exon_strands'] = compress(lesqlite2(strands))
    result['exon_counts'] = compress(lesqlite2(zigzag(delta(exon_counts))))


def extract_gene_data(data_accessor: DataAccessor, chrom: Chromosome, panel: Panel, include_exons: bool,
                      include_sequence: bool) -> Response:
    """
    Actually performs the actions described in GeneDataHandler and TranscriptDataHadnler, ie
    retrieves data for zoomed in scales (see those objects for more details). Separated out as
    top level function as there is no need for self etc when doing this activity and allows two
    objects to share an implementation (with different flags)

    Args:
        data_accessor (DataAccessor): The means of accessing data
        chrom (Chromosome): The chromosome we want
        panel (Panel): The panel (ie genomic location, scale) we want
        include_exons (bool): include info about exon location (ie for transcript views)
        include_sequence (bool): include info aboutsequence (ie for super-zoomed-in views)

    Returns:
        Response object
    """
    out = {}
    item = chrom.item_path("transcripts")
    data = get_bigbed(data_accessor, item, panel.start, panel.end)
    seen_genes = set()
    genes = []
    gene_sizes = {}
    transcript_sizes = {}
    gene_names = {}
    gene_descs = {}
    gene_biotypes = {}
    strands = {}
    designated_transcript = collections.defaultdict(lambda: (-1, None))
    transcript_biotypes = {}
    transcript_designations = {}
    for line in data:
        line = TranscriptFileLine(line)
        if line.gene_id not in seen_genes:
            genes.append(line.gene_id)
            seen_genes.add(line.gene_id)
        gene_sizes[line.gene_id] = (line.gene_start, line.gene_end)
        transcript_sizes[line.gene_id] = (line.transcript_start, line.transcript_end)
        gene_names[line.gene_id] = line.gene_name or line.gene_id
        gene_descs[line.gene_id] = line.gene_description
        gene_biotypes[line.gene_id] = line.gene_biotype
        strands[line.gene_id] = line.strand
        transcript_biotypes[line.transcript_id] = line.transcript_biotype
        transcript_designations[line.transcript_id] = line.transcript_designation
        # store candidate designated transcript
        (dt_grade_stored, _) = designated_transcript[line.gene_id]
        dt_grade = transcript_grade(transcript_designations[line.transcript_id],
                                    transcript_biotypes[line.transcript_id])
        if dt_grade > dt_grade_stored:
            designated_transcript[line.gene_id] = (dt_grade, line)
    designated_transcript = {k: v[1] for (k, v) in designated_transcript.items()}
    gene_sizes = list([gene_sizes[gene] for gene in genes])
    transcript_sizes = list([transcript_sizes[gene] for gene in genes])
    gene_names = "\0".join([gene_names[gene] for gene in genes])
    gene_descs = "\0".join([gene_descs[gene] for gene in genes])
    gene_biotypes = [gene_biotypes[gene] for gene in genes]
    gene_designations = [designated_transcript[gene].transcript_designation for gene in genes]
    designated_transcript_ids = [designated_transcript[gene].transcript_id for gene in genes]
    designated_transcript_biotypes = [transcript_biotypes[transcript] for transcript in designated_transcript_ids]
    designated_transcript_designations = [transcript_designations[transcript] for transcript in
                                          designated_transcript_ids]
    starts_and_ends(out, gene_sizes, None)
    out['gene_names'] = compress(gene_names)
    out['gene_descs'] = compress(gene_descs)
    out['gene_ids'] = compress("\0".join(genes))
    out['focus_ids'] = compress("\0".join([x.split('.')[0] for x in genes]))
    out['designated_transcript_ids'] = compress("\0".join(designated_transcript_ids))
    out['strands'] = compress(lesqlite2([int(x == '+') for x in strands.values()]))
    classified_numbers(out, gene_designations, "gene_designations")
    classified_numbers(out, gene_biotypes, "gene_biotypes")
    classified_numbers(out, designated_transcript_biotypes, "designated_transcript_biotypes")
    classified_numbers(out, designated_transcript_designations, "designated_transcript_designations")
    if include_exons:
        add_exon_data(out, genes, designated_transcript)
    sequence_blocks(out, data_accessor, chrom, panel, not include_sequence)
    # for (k,v) in out.items():
    #    logging.warn("len({}) = {}".format(k,len(v)))
    return Response(5, {'data': out})


def extract_gene_overview_data(data_accessor: DataAccessor, chrom: Chromosome, panel: Panel) -> Response:
    """
    Actually performs the actions described in GeneOverviewDataHandler, ie retrieves data for
    overview scales (see that object for more details). Separated out as top level function as
    there is no need for self etc when doing this activity.

    Args:
        data_accessor (DataAccessor): The means of accessing data
        chrom (Chromosome): The chromosome we want
        panel (Panel): The panel (ie genomic location, scale) we want

    Returns:
        Response object
    """
    out = {}
    item = chrom.item_path("transcripts")
    data = get_bigbed(data_accessor, item, panel.start, panel.end)
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
        gene_sizes[line.gene_id] = (line.gene_start, line.gene_end)
        gene_biotypes[line.gene_id] = line.gene_biotype
        strands[line.gene_id] = line.strand
        # store candidate designated transcript
    gene_sizes = list([gene_sizes[gene] for gene in genes])
    gene_biotypes = [gene_biotypes[gene] for gene in genes]
    (gene_biotypes_keys, gene_biotypes_values) = classify(gene_biotypes)
    # BUG Disabled during a bug hunt: will just lead to bigger data payloads while disabled
    min_width = 0  # int((panel.end - panel.start) / BLOCKS_PER_PANEL)
    out['starts'] = compress(lesqlite2(zigzag(delta([x[0] for x in gene_sizes]))))
    out['lengths'] = compress(lesqlite2(zigzag(delta([max(x[1] - x[0], min_width) for x in gene_sizes]))))
    out['strands'] = compress(lesqlite2([int(x == '+') for x in strands.values()]))
    out['gene_biotypes_keys'] = compress("\0".join(gene_biotypes_keys))
    out['gene_biotypes_values'] = compress(lesqlite2(gene_biotypes_values))
    out['focus_ids'] = compress("\0".join([x.split('.')[0] for x in genes]))
    return Response(5, {'data': out})


class TranscriptDataHandler(DataHandler):
    """
    Handle a request for Transcript data. This is the scale for genes where genes are 
    displayed either:
    a. as blocks with exon structure, labels, zmenus, etc, or
    b. as actual sequence.

    Whether a or b is chosen depends on whether False (a) or True (b) was passed to the
    object constructor.

    Args:
        seq (bool): Include sequence or not
    """

    def __init__(self, seq: bool):
        self._seq = seq

    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope) -> Response:
        """
        Handle a request for Transcript data. This is the scale for genes where genes are 
        displayed either:
        a. as blocks with exon structure, labels, zmenus, etc, or
        b. as actual sequence.

        Whether a or b is chosen depends on whether False (a) or True (b) was passed to the
        object constructor.

        Args:
            data_accessor (DataAccessor): The means of accessing data
            panel (Panel): The panel (ie genomic location, scale) we want
            scope (): extra scope info. Arg provided by caller but none defined for this endpoint.

        Returns: A complete Resonse object

        """
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return extract_gene_data(data_accessor,chrom,panel,True,self._seq)

class GeneDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope) -> Response:
        """
        Handle a request for Gene data. This is the scale for genes where genes are displayed
        as blocks without exon structure but including labels, zmenus, etc.

        Args:
            data_accessor (DataAccessor): The means of accessing data
            panel (Panel): The panel (ie genomic location, scale) we want
            scope (): extra scope info. Arg provided by caller but none defined for this endpoint.

        Returns: A complete Resonse object

        """
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return extract_gene_data(data_accessor,chrom,panel,False,False)

class GeneOverviewDataHandler(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope) -> Response:
        """
        Handle a request for GeneOverview data. This is the scale for genes where the track is
        so zoomed out that it's just a sequence of unclickable rectangles so that there's very
        little information *PER* gene but many genes.

        Args:
            data_accessor (DataAccessor): The means of accessing data
            panel (Panel): The panel (ie genomic location, scale) we want
            scope (): extra scope info. Arg provided by caller but none defined for this endpoint.

        Returns: A complete Resonse object

        """
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if chrom == None:
            return Response(1,"Unknown chromosome {0}".format(panel.stick))
        return extract_gene_overview_data(data_accessor,chrom,panel)
