#! /usr/bin/env python3

import sys, cbor2, json

import sys
import cbor2
import requests # use pip/pip3 if you don't have it!

data = sys.stdin.read()
request_data = bytes([ ord(x) for x in data ])

url = sys.argv[1]
if not '://' in url:
    print("must supply URL of server eg http://localhost:3333/api/data/hi")
    sys.exit(1)

request = requests.post(url,data=request_data)
content = cbor2.loads(request.content)

# Remove byte arrays from JSON

def fix(value):
    if isinstance(value,dict):
        out = {}
        for (k,v) in value.items():
            out[k] = fix(v)
        return out
    elif isinstance(value,list):
        out = []
        for v in value:
            out.append(fix(v))
        return out
    elif isinstance(value,bytes):
        return list(value)
    else:
        return value

content = fix(content)

print(json.dumps(content, indent = 1))
