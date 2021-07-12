from typing import List, Tuple
from .numbers import delta, zigzag, lesqlite2, compress, classify

def classified_numbers(result: dict, data: List[str], name: str):
    (keys,values) = classify(data)
    result[name+"_keys"] = compress("\0".join(keys))
    result[name+"_values"] = compress(lesqlite2(values))

def starts_and_ends(result: dict, sizes: List[Tuple[int,int]], name: str):
    if name:
        name += "_"
    else:
        name = ""
    result[name+'starts'] = compress(lesqlite2(zigzag(delta([ x[0] for x in sizes ]))))
    result[name+'lengths'] = compress(lesqlite2(zigzag(delta([ x[1]-x[0] for x in sizes ]))))

def starts_and_lengths(result: dict, sizes: List[Tuple[int,int]], name: str):
    if name:
        name += "_"
    else:
        name = ""
    result[name+'starts'] = compress(lesqlite2(zigzag(delta([ x[0] for x in sizes ]))))
    result[name+'lengths'] = compress(lesqlite2(zigzag(delta([ x[1] for x in sizes ]))))
