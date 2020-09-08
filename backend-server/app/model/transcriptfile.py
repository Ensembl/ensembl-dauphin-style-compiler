import logging
import base64

"""
    (
    string chrom;                   "Reference sequence chromosome or scaffold"
    uint chromStart;                "Gene start position in chromosome"
    uint chromEnd;                  "Gene end position in chromosome"
    string name;                    "Transcript id (versioned)"
    char[1] strand;                 "Strand direction (+ or -)"
    uint thickStart;                "Start of coding sequence"
    uint thickEnd;                  "End of coding sequence"
    int blockCount;                 "Number of blocks (exons) in a transcript"
    int[blockCount] blockSizes;     "Comma-separated list of block (exon) sizes"
    int[blockCount] blockStarts;    "Comma-separated list of block (exon) start positions relative to chromStart"
    uint transcriptStart;           "Transcript start position in chromosome"
    uint transcriptEnd;             "Transcript end position in chromosome"
    string transcriptBiotype;       "Transcript biotype"
    string transcriptDesignation;   "enum('-', 'mane_select', 'canonical', 'ensembl_select')"
    string geneId;                  "Primary identifier for gene"
    string geneName;                "Name of the gene"
    lstring base64GeneDescription;  "Gene description (UTF-8) encoded as base64. Set to - if unknown"
    string geneBiotype;             "Gene biotype"
"""

class TranscriptFileLine(object):
    def __init__(self,data):
        (self.gene_start, self.gene_end, rest) = data
        (
            self.transcript_id, self.strand, self.thick_start, self.thick_end,
            self.block_count, block_sizes, block_starts, self.transcript_start,
            self.transcript_end, self.transcript_biotype, self.transcript_designation,
            self.gene_id, self.gene_name, base64_gene_description, self.gene_biotype
        ) = rest.split("\t")
        self.block_sizes = block_sizes.split(",")
        self.block_starts = block_starts.split(",")
        self.gene_description = base64.decodebytes(base64_gene_description.encode("ascii")).decode("utf8")

