import collections

def unversioned(data):
    parts = data.rsplit(".",1)
    return parts[0]

def filter_lines_by_criteria(lines,for_id,max_tr,expanded):
    out = []
    num_tr_seen = collections.defaultdict(int)
    for line in lines:
        # 1. if we've been asked about a specific gene and this is not it, skip
        if for_id is not None and line.gene_id != for_id[1] and unversioned(line.gene_id) != for_id[1]:
            continue
        num_tr_seen[line.gene_id] += 1
        # 2. if we only need info about the top-n transcripts of agene, skip the rest
        if max_tr is not None and num_tr_seen[line.gene_id] > max_tr and line.gene_id not in (expanded or []):
            continue
        out.append(line)
    # Record total transcript count for lozenge logic
    for line in out:
        line.all_transcript_counts = num_tr_seen[line.gene_id]
    return out
