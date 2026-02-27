import pyBigWig

from model.datalocator import AccessItem
from command.datasources import DataAccessor
from command.coremodel import Panel
from core.exceptions import RequestException

"""
Attributes:
    _bigbeds (dict) A cache of open bigbed files, for efficiency
    _bigwigs (dict) A cache of open bigwig files, for efficiency
"""
_bigbeds = {}
_bigwigs = {}


def _get_bigbed_data(path, chrom, start, end):
    """
    Retrieve data lines from bigbed file.

    This low-level, file-internal function is only used by get_bigbed further down this
    file.

    Args:
        path (str): file path of bigbed file
        chrom (str): chromosome name as used in bigbed file
        start (int): start co-ordinate of range of interest
        end (int): end co-ordinate of range of interest

    Returns:
        List of 3-tuples each being: start coordinate; end coordinate; rest of string

    """
    if start >= chrom.size:
        return []
    end = min(end, chrom.size)
    try:
        if not (path in _bigbeds):
            _bigbeds[path] = pyBigWig.open(path)
        bb = _bigbeds[path]
        out = bb.entries(chrom.name, start, end) or []
    except (RuntimeError, OverflowError) as e:
        print(f"Error reading BigBed from {path}: {e}")
        out = []
    return out


def _get_bigwig_stats_data(path, chrom, start, end, consolidation="mean", nBins=500):
    """
    Retrieve binned data from bigwig file. Binned data is data where each data-point
    represents a summary of more than one data-point in the file, consolidtaed with
    the given function, and so is useful at more zoomed-out levels.

    This low-level, file-internal function is only used by get_bigwig_stats further down
    this file.

    Args:
        path (str): path to bigwig file
        chrom (str): chromosome in bigwig file
        start (int): start coordinate of region of interest
        end (int): end coordinate of region of interest
        consolidation (str): method to use to reduce multiple values to single datapoint
        nBins (int): number of bins (ie datapoints) to produce

    Returns:
        tuple(list, int, int): first value is the actual datapoints, other two are the
        actual start and end used, which will have expanded a little over the start and
        end given. This is needed to assume that there are no edge-effects from the
        binning opertaion.
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
        print(f"Error reading BigWig from {path}: {e}")
        out = []
    return out, start, end


def _get_bigwig_data(path, chrom, start, end):
    """
    Retireve raw datapoints from a bigwig file. As every datapoint in the file within
    the region is returned, this is only useful when very zoomed in.

    This low-level, file-internal function is only used by get_bigwig further down this
    file.

    Args:
        path (str): path to bigwig file
        chrom (str): chromosome name
        start (int): start coordinate of region of interest
        end (int): end coordinate of region of interest

    Returns:
        tuple(list, int, int): first value is the actual datapoints, other two are the
        actual start and end used, which will have expanded a little over the start and
        end given. This is needed to assume that there are no edge-effects from the
        binning opertaion.
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
        print(f"Error reading BigWig from {path}: {e}")
        out = []
    return out, start, end


def get_bigbed(data_accessor: DataAccessor, item: AccessItem, start: int, end: int):
    """
    Retrieve bigbed data using am AccessItem, which is an abstraction over a source of
    some data and might map to a local file or a url, etc.

    Args:
        data_accessor (DataAccessor): singleton business object containing location-resolution code
        item (AccessItem): some data source in bigbed format
        start (int): start coordinate of region of interest
        end (int): end coordinate of region of interest

    Returns:
        List of 3-tuples each being: start coordinate; end coordinate; rest of string

    """
    if end <= start:
        return []
    accessor = data_accessor.resolver.get(item)
    chromosome = data_accessor.data_model.stick(item.stick())
    if accessor is None:
        return []
    if accessor.file is not None:
        return _get_bigbed_data(accessor.file, chromosome, start, end)
    elif accessor.url is not None:
        return _get_bigbed_data(accessor.url, chromosome, start, end)
    else:
        raise RequestException("cannot use accessor to get data")


def get_bigbed_fields(
    data_accessor: DataAccessor,
    panel: Panel,
    filename: str,
    rest_fields: list[str],
) -> dict[str, list]:
    chrom = panel.get_chrom(data_accessor)
    data = get_bigbed(data_accessor, chrom.item_path(filename), panel.start, panel.end)

    out: dict[str, list] = {field: [] for field in rest_fields}
    out["chr"] = []
    out["start"] = []
    out["end"] = []

    for (start, end, rest) in data:
        out["chr"].append(chrom.name)
        out["start"].append(start)
        out["end"].append(end)

        parts = rest.split("\t")
        for field, idx in zip(rest_fields, range(len(rest_fields))):
            value = parts[idx] if idx < len(parts) else ""
            out[field].append(value)

    return out


def get_bigwig(data_accessor: DataAccessor, item: AccessItem, start: int, end: int):
    """
    Retireve raw datapoints from a bigwig file. As every datapoint in the file within
    the region is returned, this is only useful when very zoomed in.

    Args:
        data_accessor (DataAccessor): singleton business object containing location-resolution code
        item (AccessItem): some data source in bigbed format
        start (int): start coordinate of region of interest
        end (int): end coordinate of region of interest

    Returns:
        tuple(list, int, int): first value is the actual datapoints, other two are the
        actual start and end used, which will have expanded a little over the start and
        end given. This is needed to assume that there are no edge-effects from the
        binning opertaion.
    """
    accessor = data_accessor.resolver.get(item)
    chromosome = data_accessor.data_model.stick(item.stick())
    if accessor is None:
        return [], start, end
    if accessor.file is not None:
        return _get_bigwig_data(accessor.file, chromosome, start, end)
    elif accessor.url is not None:
        return _get_bigwig_data(accessor.url, chromosome, start, end)
    else:
        raise RequestException("cannot use accessor to get data")


def get_bigwig_stats(data_accessor: DataAccessor, item: AccessItem, start: int, end: int, consolidation: str = "mean",
                     nBins: int = 1000):
    """
    Retrieve binned data from bigwig file. Binned data is data where each data-point
    represents a summary of more than one data-point in the file, consolidtaed with
    the given function, and so is useful at more zoomed-out levels.

    This low-level, file-internal function is only used by get_bigwig_stats further down
    this file.

    Args:
        data_accessor (DataAccessor): singleton business object containing location-resolution code
        item (AccessItem): some data source in bigbed format
        start (int): start coordinate of region of interest
        end (int): end coordinate of region of interest
        consolidation (str): method to use to reduce multiple values to single datapoint
        nBins (int): number of bins (ie datapoints) to produce

    Returns:
        tuple(list, int, int): first value is the actual datapoints, other two are the
        actual start and end used, which will have expanded a little over the start and
        end given. This is needed to assume that there are no edge-effects from the
        binning opertaion.
    """
    accessor = data_accessor.resolver.get(item)
    chromosome = data_accessor.data_model.stick(item.stick())
    if accessor is None:
        raise RequestException("Cannot resolve item")
    if accessor.file is not None:
        return _get_bigwig_stats_data(accessor.file, chromosome, start, end, consolidation=consolidation, nBins=nBins)
    elif accessor.url is not None:
        return _get_bigwig_stats_data(accessor.url, chromosome, start, end, consolidation=consolidation, nBins=nBins)
    else:
        raise RequestException("cannot use accessor to get data")
