from functools import cmp_to_key

def build_map(data):
    mapping = { v: i for (i,v) in enumerate(data) }
    return (data,mapping)

def increase(data):
    prev = 0
    out = []
    for item in data:
        out.append(item-prev)
        prev = item
    return out

def immute(data):
    if isinstance(data,list):
        return tuple([True] + [immute(x) for x in data])
    elif isinstance(data,dict):
        keys = sorted(data.keys())
        items = [(k,immute(data[k])) for k in keys]
        return tuple([False] + items)
    else:
        return data

def remute(data):
    if isinstance(data,tuple):
        if data[0]:
            return data[1:]
        else:
            return { x[0]: x[1] for x in data[1:] }
    else:
        return data

def cmp_immute(a,b):
    a_tuple = isinstance(a,tuple)
    b_tuple = isinstance(b,tuple)
    if a_tuple and not b_tuple:
        return -1
    if b_tuple and not a_tuple:
        return 1
    if a_tuple and b_tuple:
        if len(a) < len(b):
            return -1
        if len(a) > len(b):
            return 1
        for (x,y) in zip(a,b):
            c = cmp_immute(x,y)
            if c != 0:
                return c
        return 0
    else:
        if str(a) < str(b):
            return -1
        if str(a) > str(b):
            return 1
        return 0

def immute_key():
    return cmp_to_key(cmp_immute)
