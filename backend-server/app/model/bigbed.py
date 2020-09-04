import pyBigWig

def get_bigbed_data(path,chrom,start,end):
    end = min(end,chrom.size)
    bb = pyBigWig.open(path)
    try:
        out = bb.entries(chrom.name,start,end) or []
    except (RuntimeError,OverflowError):
        out = []
    bb.close()
    return out
