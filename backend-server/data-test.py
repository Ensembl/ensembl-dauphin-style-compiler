#! /usr/bin/env python3

import sys, re, getopt
import cbor2, zlib
import requests # use pip/pip3 if you don't have it!

use_rich = sys.stdout.isatty() 

(optlist,args) = getopt.getopt(sys.argv[1:],[],['rich','no-rich'])
for (option,value) in optlist:
    if option == '--rich':
        use_rich = True
    elif option == '--no-rich':
        use_rich = False

CODES = {
    'X': '\033[2J\033[H'
}

COLOUR_CODES = { 'r': '91;1', 'g': '32;1', 'y': '93;1', '-': '39;0', 'b': '34;1', 'c': '96;1' }
CODES |= { k: "\033[{0}m".format(v) for (k,v) in COLOUR_CODES.items() }

if not use_rich:
    CODES = { k: '' for k in CODES.keys() }

def rich(line):
    return re.sub(r'\0(.)',lambda x: CODES[x[1]],line)

def ask(question,default):
    print(rich("\0y{}\0-? ".format(question)),end=' ',flush=True)
    out = sys.stdin.readline().strip()
    if out == '':
        out = default
    return out

endpoint = ask('Endpoint name','transcript')
print(rich("Ok, using endpoint \0y{}\0-").format(endpoint))

ask_chrom = ask("Enter stick name (eg \0ghomo_sapiens_GCA_000001405_28:4\0y)","homo_sapiens_GCA_000001405_28:4")
if ask_chrom.strip() != '':
    chrom = ask_chrom

bp_in_data = int(ask("Roughly how many bp in data",100000))
scale = 1
for try_scale in range(0,40):
    base_bp = 2**try_scale
    if base_bp * 1.41 < bp_in_data:
        scale = try_scale
print(rich("Ok, nearest is scale \0y{}\0- which has \0y{}\0- bp".format(scale,2**scale)))

bp_centre = int(ask("Approx centre in bp",10000000))
index = max(0,bp_centre//(2**scale))
print(rich("Ok, nearest is index \0y{}\0- which has bp range \0y{}\0- - \0y{}\0-".format(
    index, index*(2**scale), (index+1)*(2**scale)-1
)))

url = sys.argv[1]
if not '://' in url:
    print("must supply URL of server eg http://localhost:3333/api/data/hi")
    sys.exit(1)

def to_bytes(input):
    if isinstance(input, str):
        return input.encode("utf-8")
    else:
        return input

def compressed_size(input):
    return len(zlib.compress(to_bytes(input)))

def heading(text):
    print("\n{}\n{}\n\n".format(text.upper(),"=" * len(text)))

def calc_sizes_one(sizes,keys,data):
    prev_len = compressed_size("")
    current_str = b''
    for key in keys:
        current_str += to_bytes(data[key])
        new_len = compressed_size(current_str)
        sizes[key] += new_len-prev_len
        prev_len = new_len

def calc_sizes(data):
    keys = sorted(list(data))
    sizes = { k: 0 for k in keys }
    calc_sizes_one(sizes,keys,data)
    calc_sizes_one(sizes,reversed(keys),data)
    return { k: v/2 for (k,v) in sizes.items() }

def summarize_data(data,sizes):
    heading("raw dump")
    keys = sorted(list(data))
    for key in keys:
        print(key,data[key])
    heading("approx sizes")
    key_width = max([len(k) for k in keys])
    total_size = sum([sizes[k] for k in sizes])
    rows = []
    for key in keys:
        value = "{:{w}} {:>6} {:>3}%".format(key,round(sizes[key]),round(sizes[key]*100/total_size),w=key_width)
        rows.append((-sizes[key],value))
    rows.sort()
    for (_,value) in rows:
        print(value)

def make_request(accept):
    request_data = [0,4,["self()",endpoint,[chrom,scale,index],{},accept]]

    request_data = cbor2.dumps({ 
        "channel": "self()", 
        "requests": [request_data],
        "version": { "egs": 14 }
    })
    request = requests.post(url,data=request_data)
    content = cbor2.loads(request.content)

    for response in content['responses']:
        (msgid,(resp,payload)) = response
        if msgid == 0 and resp == 5:
            return cbor2.loads(zlib.decompress(payload['data']))

sizes = calc_sizes(make_request("uncompressed"))
summarize_data(make_request("dump"),sizes)
