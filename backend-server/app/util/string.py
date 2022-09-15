def split_all(pat,input):
    start = 0
    while True:
        idx = input.find(pat,start)
        if idx != -1:
            yield (input[:idx],input[idx:])
            start = idx+1
        else:
            break
    yield (input,"")