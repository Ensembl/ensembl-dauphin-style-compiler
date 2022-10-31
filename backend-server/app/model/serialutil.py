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
