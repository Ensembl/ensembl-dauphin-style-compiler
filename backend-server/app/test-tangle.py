#! /usr/bin/env python3

import json
import sys, os
import unittest

# to allow tests in this file desipte relative imports
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from tangle.tangle import TangleFactory

def test_data(filename):
    with open(os.path.join(os.path.dirname(__file__),"tangle","testdata",filename),"r") as f:
        return f.read()

class Processor:
    def inc(self, value):
        return value + 1

class TangleTestCase(unittest.TestCase):
    def test_smoke(self):
        self.maxDiff = 10000
        tangle_factory = TangleFactory()
        test_config = test_data("smoke-config.toml")
        data_in = json.loads(test_data("smoke-in.json"))
        data_out = json.loads(test_data("smoke-out.json"))
        tangle = tangle_factory.make_from_toml(test_config,["on"],Processor())
        out = {}
        tangle.run(out,data_in,to_bytes=False,compress=False)
        self.assertEqual(out,data_out)

if __name__ == '__main__':
    unittest.main()
