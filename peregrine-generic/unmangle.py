#! /usr/bin/env python3

import sys

data = sys.stdin.read()

print(", ".join([hex(ord(c)) for c in data]))

