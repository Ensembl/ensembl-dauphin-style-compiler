import logging
import pyBigWig
import sys
import time
from model.datalocator import AccessItem
from command.datasources import DataAccessor
from core.exceptions import RequestException

_bigbeds = {}
_bigwigs = {}

def _get_bigbed_data(path,chrom,start,end):
    end = min(end,chrom.size)
    try:
        if not (path in _bigbeds):
            _bigbeds[path] = pyBigWig.open(path)
        bb = _bigbeds[path]
        out = bb.entries(chrom.name,start,end) or []
    except (RuntimeError,OverflowError):
        out = []
    return out

def get_bigwig_stats_data(path,chrom,start,end,consolidation="mean",nBins=1000):
    end = min(end,chrom.size)
    if end < start:
        return ([],start)
    try:
        if not (path in _bigwigs):
            _bigwigs[path] = pyBigWig.open(path)
        bw = _bigwigs[path]
        out = bw.stats(chrom.name,start,end,nBins=nBins,type=consolidation) or []
    except (RuntimeError,OverflowError) as e:
        out = []
    return (out,end)

def get_bigwig_data(path,chrom,start,end):
    end = min(end,chrom.size)
    if end < start:
        return ([],start)
    try:
        if not (path in _bigwigs):
            _bigwigs[path] = pyBigWig.open(path)
        bw = _bigwigs[path]
        out = bw.values(chrom.name,start,end) or []
    except (RuntimeError,OverflowError) as e:
        out = []
    return (out,end)

def get_bigbed(data_accessor: DataAccessor, item: AccessItem, start: int, end: int):
    accessor = data_accessor.resolver.get(item)
    chromosome = data_accessor.data_model.stick(data_accessor,item.stick())
    if accessor == None:
        raise RequestException("Cannot resolve item")
    if accessor.file != None:
        return _get_bigbed_data(accessor.file,chromosome,start,end)
    elif accessor.url != None:
        return _get_bigbed_data(accessor.url,chromosome,start,end)
    else:
        raise RequestException("cannot use accessor to get data")

def get_bigwig(data_accessor: DataAccessor, item: AccessItem, start: int, end: int):
    accessor = data_accessor.resolver.get(item)
    chromosome = data_accessor.data_model.stick(data_accessor,item.stick())
    if accessor == None:
        raise RequestException("Cannot resolve item")
    if accessor.file != None:
        return get_bigwig_data(accessor.file,chromosome,start,end)
    elif accessor.url != None:
        return get_bigwig_data(accessor.url,chromosome,start,end)
    else:
        raise RequestException("cannot use accessor to get data")

def get_bigwig_stats(data_accessor: DataAccessor, item: AccessItem, start: int, end: int, consolidation : str  = "mean",nBins : int = 1000):
    accessor = data_accessor.resolver.get(item)
    chromosome = data_accessor.data_model.stick(data_accessor,item.stick())
    if accessor == None:
        raise RequestException("Cannot resolve item")
    if accessor.file != None:
        return get_bigwig_stats_data(accessor.file,chromosome,start,end,consolidation=consolidation,nBins=nBins)
    elif accessor.url != None:
        return get_bigwig_stats_data(accessor.url,chromosome,start,end,consolidation=consolidation,nBins=nBins)
    else:
        raise RequestException("cannot use accessor to get data")
