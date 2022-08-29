import collections

def unversioned(data):
    parts = data.rsplit(".",1)
    return parts[0]

# 1. if we've been asked about a specific gene and this is not it, skip
# 2. if we only need info about the top-n transcripts of agene, skip the rest
def filter_lines_by_criteria(lines,for_id,max_tr):
    num_tr_seen = collections.defaultdict(int)
    for line in lines:
        if for_id is not None and line.gene_id != for_id and unversioned(line.gene_id) != for_id:
            continue
        num_tr_seen[line.gene_id] += 1
        if max_tr is not None and num_tr_seen[line.gene_id] > max_tr:
            continue
        yield line
