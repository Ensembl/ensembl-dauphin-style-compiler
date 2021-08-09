import logging
import pyBigWig
import sys
import time
from model.datalocator import AccessItem
from command.datasources import DataAccessor
from core.exceptions import RequestException

_bigbeds = {}

def get_bigbed_data(path,chrom,start,end):
    end = min(end,chrom.size)
    try:
        if not (path in _bigbeds):
            _bigbeds[path] = pyBigWig.open(path)
        bb = _bigbeds[path]
        out = bb.entries(chrom.name,start,end) or []
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

def get_bigbed(data_accessor: DataAccessor, item: AccessItem, start: int, end: int):
    accessor = data_accessor.resolver.get(item)
    chromosome = data_accessor.data_model.sticks[item.stick()]
    if accessor == None:
        raise RequestException("Cannot resolve item")
    if accessor.file != None:
        return get_bigbed_data(accessor.file,chromosome,start,end)
    elif accessor.url != None:
        return get_bigbed_data(accessor.url,chromosome,start,end)
    else:
        raise RequestException("cannot use accessor to get data")
