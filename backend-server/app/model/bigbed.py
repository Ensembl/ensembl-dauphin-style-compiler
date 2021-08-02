import logging
import pyBigWig
import sys
import time

def get_bigbed_data(path,chrom,start,end):
    end = min(end,chrom.size)
    try:
        bb = pyBigWig.open(path)
        out = bb.entries(chrom.name,start,end) or []
        bb.close()
    except (RuntimeError,OverflowError):
        out = []
    return out

def get_bigwig_stats(path,chrom,start,end,consolidation="mean",nBins=1000):
    end = min(end,chrom.size)
    start_time = time.time()
    try:
        bw = pyBigWig.open(path)
        out = bw.stats(chrom.name,start,end,nBins=nBins,type=consolidation) or []
        bw.close()
    except (RuntimeError,OverflowError) as e:
        out = []
    return out

def get_bigwig_data(path,chrom,start,end):
    end = min(end,chrom.size)
    start_time = time.time()
    try:
        bw = pyBigWig.open(path)
        out = bw.values(chrom.name,start,end) or []
        bw.close()
    except (RuntimeError,OverflowError) as e:
        out = []
    return out
