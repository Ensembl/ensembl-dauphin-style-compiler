import logging
import pyBigWig
import numpy
import sys
import time

if not pyBigWig.numpy:
    logging.error("numpy must be installed before pyBigWig for speed reasons. Please install numpy then reinsall pyBIgWig")
    sys.exit(1)

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
    start_time = time.time()
    try:
        bw = pyBigWig.open(path)
        out = bw.stats(chrom.name,start,end,nBins=1000) or []
    except (RuntimeError,OverflowError) as e:
        logging.error(e)
        out = []
    logging.error("{0}bp {1}ms".format(end-start,int((time.time()-start_time)*1000)))
    bw.close()
    return out
