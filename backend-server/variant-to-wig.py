#! /usr/bin/env python3

import sys

# syntax: python chrom-name variant-to-wig.py source.z.bed out.wig

severities = {
    '3_prime_UTR_variant': 2,
    '5_prime_UTR_variant': 2,
    'coding_sequence_variant': 3,
    'downstream_gene_variant': 2,
    'feature_elongation': 1,
    'feature_truncation': 1,
    'frameshift_variant': 5,
    'incomplete_terminal_codon_variant': 3,
    'inframe_deletion': 4,
    'inframe_insertion': 4,
    'intergenic_variant': 1,
    'intron_variant': 2,
    'mature_miRNA_variant': 3,
    'missense_variant': 4,
    'NMD_transcript_variant': 2,
    'non_coding_transcript_exon_variant': 2,
    'non_coding_transcript_variant': 2,
    'protein_altering_variant': 4,
    'regulatory_region_ablation': 4,
    'regulatory_region_amplification': 1,
    'regulatory_region_fusion': 1,
    'regulatory_region_translocation': 1,
    'regulatory_region_variant': 1,
    'splice_acceptor_variant': 5,
    'splice_donor_variant': 5,
    'splice_region_variant': 3,
    'start_lost': 5,
    'start_retained_variant': 3,
    'stop_retained_variant': 3,
    'stop_gained': 5,
    'stop_lost': 5,
    'synonymous_variant': 3,
    'TFBS_ablation': 1,
    'TFBS_amplification': 1,
    'TFBS_fusion': 1,
    'TFBS_translocation': 1,
    'TF_binding_site_variant': 1,
    'transcript_ablation': 5,
    'transcript_amplification': 5,
    'transcript_fusion': 4,
    'transcript_translocation': 2,
    'upstream_gene_variant': 2,
}

with open(sys.argv[2]) as file_in:
    with open(sys.argv[3],'w') as file_out:
        file_out.write("variableStep chrom={}\n".format(sys.argv[1]))
        position = 1
        for line in file_in.readlines():
            parts = line.strip().split("\t")
            (start,end,variant_type) = (int(parts[1]),int(parts[2]),parts[3])
            severity = severities.get(variant_type,0)
            while position < end:
                s =  0 if position < start else severity
                file_out.write("\t".join([str(position),str(s)])+"\n")
                position += 1
