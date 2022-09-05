from typing import Any, List, Tuple
from .numbers import delta, zigzag, lesqlite2, compress, classify


def classified_numbers(result: dict, data: List[str], name: str):
    """

    Args:
        result (dict):
        data (List[str]):
        name (str):

    Returns:

    """
    (keys, values) = classify(data)
    result[name + "_keys"] = compress("\0".join(keys))
    result[name + "_values"] = compress(lesqlite2(values))


def starts_and_ends(result: dict, sizes: List[Tuple[int, int]], name: str):
    """

    Args:
        result (dict):
        sizes (List[Tuple[int, int]]):
        name (str):

    Returns:

    """
    if name:
        name += "_"
    else:
        name = ""
    result[name + 'starts'] = compress(lesqlite2(zigzag(delta([x[0] for x in sizes]))))
    result[name + 'lengths'] = compress(lesqlite2(zigzag(delta([x[1] - x[0] for x in sizes]))))


def starts_and_lengths(result: dict, sizes: List[Tuple[int, int]], name: str):
    """

    Args:
        result (dict):
        sizes (List[Tuple[int, int]]):
        name (str):

    Returns:

    """
    if name:
        name += "_"
    else:
        name = ""
    result[name + 'starts'] = compress(lesqlite2(zigzag(delta([x[0] for x in sizes]))))
    result[name + 'lengths'] = compress(lesqlite2(zigzag(delta([x[1] for x in sizes]))))


def domino_series_expand(data: List[Any], distance: int) -> List[Any]:
    """

    Args:
        data (List[Any]):
        distance (int):

    Returns:
        List[Any]
    """
    out = data[:]
    for (i, x) in enumerate(data):
        if x > 0:
            for delta in range(1, distance + 1):
                if i + delta >= len(out):
                    break
                if data[i + delta] == 0 and out[i + delta] == 0:
                    out[i + delta] = x
                else:
                    break
            for delta in range(1, distance + 1):
                if i - delta < 0:
                    break
                if data[i - delta] == 0 and out[i - delta] == 0:
                    out[i - delta] = x
                else:
                    break
    return out
