import logging
import pyBigWig

def get_bigbed_data(path,chrom,start,end):
    end = min(end,chrom.size)
    try:
        bb = pyBigWig.open(path)
        out = bb.entries(chrom.name,start,end) or []
    except (RuntimeError,OverflowError):
        out = []
    bb.close()
    return out

def get_bigwig_data(path,chrom,start,end):
    end = min(end,chrom.size)
    try:
        bw = pyBigWig.open(path)
        out = bw.stats(chrom.name,start,end,nBins=1000) or []
    except (RuntimeError,OverflowError) as e:
        logging.error(e)
        out = []
    bw.close()
    return out
