# 1. we must keep genes in order of appearance to avoid corrupting gene data
# 2. beyond that transcripts get a value which is maximised

def _transcript_value(line):
    # default transcript ordering logic (ENSWBSITES-1695)
    # designation
    designation = line.transcript_designation.lower()
    if designation == 'mane_select' or designation == "canonical":
        designation_value = 2
    elif designation.startswith('mane'):
        designation_value = 1
    else:
        designation_value = 0
    # biotype
    biotype = line.transcript_biotype
    if biotype == 'protein_coding':
        biotype_value = 5
    elif biotype == 'nonsense_mediated_decay':
        biotype_value = 4
    elif biotype == 'non_stop_decay':
        biotype_value = 3
    elif biotype.startswith('IG_'):
        biotype_value = 2
    elif biotype == 'polymorphic_pseudogene':
        biotype_value = 1
    else:
        biotype_value = 0
    #
    translation_length = line.translation_length
    transcript_length = sum(line.block_sizes)
    return (designation_value,biotype_value,int(translation_length),transcript_length)

def _sort_gene_transcripts(transcripts):
    # use transcript rank if present (assumes all or none have a rank)
    if transcripts[0].rank != -1:
        return sorted(transcripts, key=lambda tr: tr.rank)
    # otherwise use default ordering
    return sorted(transcripts, key=_transcript_value, reverse=True)

def sort_data_by_transcript_priority(data):
    gene_data = {}
    gene_order = []
    for line in data:
        if line.gene_id not in gene_data:
            gene_data[line.gene_id] = []
            gene_order.append(line.gene_id)
        gene_data[line.gene_id].append(line)
    out = []
    for gene in gene_order:
        for line in _sort_gene_transcripts(gene_data[gene]):
            out.append(line)
    return out
