import pyBigWig

from model.datalocator import AccessItem
from command.datasources import DataAccessor
from core.exceptions import RequestException

"""
Attributes:
    _bigbeds (dict)
    _bigwigs (dict)
"""
_bigbeds = {}
_bigwigs = {}


def _get_bigbed_data(path, chrom, start, end):
    """

    Args:
        path ():
        chrom ():
        start (int):
        end (int):

    Returns:
        list

    """
    end = min(end, chrom.size)
    try:
        if not (path in _bigbeds):
            _bigbeds[path] = pyBigWig.open(path)
        bb = _bigbeds[path]
        out = bb.entries(chrom.name, start, end) or []
    except (RuntimeError, OverflowError):
        out = []
    return out


def get_bigwig_stats_data(path, chrom, start, end, consolidation="mean", nBins=500):
    """

    Args:
        path ():
        chrom ():
        start (int):
        end (int):
        consolidation (str):
        nBins ():

    Returns:
        tuple(list, int, int)
    """
    # angel's share: extra at start end end to allow seamless overlap
    angel_share = int((end - start) / nBins) + 1
    start = start - 2 * angel_share
    end = end + 2 * angel_share
    start = max(start, 0)
    end = min(end, chrom.size)
    if end < start:
        return [], start, start
    try:
        if not (path in _bigwigs):
            _bigwigs[path] = pyBigWig.open(path)
        bw = _bigwigs[path]
        out = bw.stats(chrom.name, start, end, nBins=nBins, type=consolidation) or []
    except (RuntimeError, OverflowError, RequestException) as e:
        out = []
    return out, start, end


def get_bigwig_data(path, chrom, start, end):
    """

    Args:
        path ():
        chrom ():
        start (int):
        end (int):

    Returns:
        tuple(list, int, int)
    """
    end = min(end, chrom.size)
    if end < start:
        return [], start, start
    try:
        if not (path in _bigwigs):
            _bigwigs[path] = pyBigWig.open(path)
        bw = _bigwigs[path]
        out = bw.values(chrom.name, start, end) or []
    except (RuntimeError, OverflowError, RequestException) as e:
        out = []
    return out, start, end


def get_bigbed(data_accessor: DataAccessor, item: AccessItem, start: int, end: int):
    """

    Args:
        data_accessor ():
        item ():
        start (int):
        end (int):

    Returns:

    """
    accessor = data_accessor.resolver.get(item)
    chromosome = data_accessor.data_model.stick(data_accessor, item.stick())
    if accessor is None:
        return []
    if accessor.file is not None:
        return _get_bigbed_data(accessor.file, chromosome, start, end)
    elif accessor.url is not None:
        return _get_bigbed_data(accessor.url, chromosome, start, end)
    else:
        raise RequestException("cannot use accessor to get data")


def get_bigwig(data_accessor: DataAccessor, item: AccessItem, start: int, end: int):
    """

    Args:
        data_accessor ():
        item ():
        start (int):
        end (int):

    Returns:

    """
    accessor = data_accessor.resolver.get(item)
    chromosome = data_accessor.data_model.stick(data_accessor, item.stick())
    if accessor is None:
        return [], start, end
    if accessor.file is not None:
        return get_bigwig_data(accessor.file, chromosome, start, end)
    elif accessor.url is not None:
        return get_bigwig_data(accessor.url, chromosome, start, end)
    else:
        raise RequestException("cannot use accessor to get data")


def get_bigwig_stats(data_accessor: DataAccessor, item: AccessItem, start: int, end: int, consolidation: str = "mean",
                     nBins: int = 1000):
    """

    Args:
        data_accessor ():
        item ():
        start (int):
        end (int):
        consolidation ():
        nBins ():

    Returns:

    """
    accessor = data_accessor.resolver.get(item)
    chromosome = data_accessor.data_model.stick(data_accessor, item.stick())
    if accessor is None:
        raise RequestException("Cannot resolve item")
    if accessor.file is not None:
        return get_bigwig_stats_data(accessor.file, chromosome, start, end, consolidation=consolidation, nBins=nBins)
    elif accessor.url is not None:
        return get_bigwig_stats_data(accessor.url, chromosome, start, end, consolidation=consolidation, nBins=nBins)
    else:
        raise RequestException("cannot use accessor to get data")
