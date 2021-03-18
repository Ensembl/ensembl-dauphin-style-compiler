#! /usr/bin/env python3

import sys
import cbor2
import requests # use pip/pip3 if you don't have it!

url = sys.argv[1]
if not '://' in url:
    print("must supply URL of server eg http://localhost:3333/api/data/hi")
    sys.exit(1)

request_data = cbor2.dumps({ "channel": "self()", "requests": [[0,0,None]]} )
request = requests.post(url,data=request_data)
content = cbor2.loads(request.content)
good = False
if "responses" in content:
    for response in content["responses"]:
        if len(response) == 3:
            if response[0] == 0 and response[1] == 0:
                good = True
if good:
    print("got boot response. (len={0} bytes). good.".format(len(request.content)))
else:
    print("no boot response. something wrong. Supplied should have form http://my.host.name:3333/api/data/hi or similar")
    sys.exit(2)
