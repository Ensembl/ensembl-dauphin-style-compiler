from typing import List, ByteString, Union, Tuple, Dict
import zlib


# TODO generators to help sequencing if it's a speed improvement

def compress(input: Union[ByteString, str]) -> ByteString:
    """

    Args:
        input (Union[ByteString, str]):

    Returns:
        ByteString
    """
    if isinstance(input, str):
        input = input.encode("utf-8")
    return input


def delta(input: List[int]) -> List[int]:
    """

    Args:
        input (List[int]):

    Returns:
        List[int]
    """
    value: int = 0
    output: List[int] = []
    for item in input:
        output.append(item - value)
        value = item
    return output


def zigzag(input: List[int]) -> List[int]:
    """

    Args:
        input (List[int]):

    Returns:
        List[int]
    """
    output: List[int] = []
    for item in input:
        if item >= 0:
            output.append(item * 2)
        else:
            output.append(-item * 2 - 1)
    return output


# props to stoklund
def lesqlite2(input: List[int]) -> ByteString:
    """

    Args:
        input (List[int]):

    Returns:
        ByteString
    """
    output: ByteString = bytearray()
    for item in input:
        if item < 178:
            output.append(item)
        elif item < 16562:
            (a, b) = divmod(item - 178, 256)
            output.append(178 + a)
            output.append(b)
        elif item < 540850:
            (a, b) = divmod(item - 16562, 65536)
            (b, c) = divmod(b, 256)
            output.append(242 + a)
            output.append(c)
            output.append(b)
        else:
            pos = len(output)
            output.append(247)
            while item > 0:
                output[pos] += 1
                (item, rem) = divmod(item, 256)
                output.append(rem)
    return output


def classify(input: List[str]) -> Tuple[List[str], List[int]]:
    """

    Args:
        input (List[str]):

    Returns:
        Tuple[List[str], List[int]]
    """
    mapping: Dict[str, int] = {}
    keys: List[str] = []
    values: List[int] = []
    for item in input:
        value = mapping.get(item)
        if value is None:
            value = len(mapping)
            mapping[item] = value
            keys.append(item)
        values.append(value)
    return keys, values
