#! /usr/bin/env python3

import argparse
import logging
import sys
from logging.handlers import SysLogHandler
from logging import StreamHandler
from pymemcache.client import base
import time
import urllib.request
from pathlib import Path

# Extract and reflect config

parser = argparse.ArgumentParser(description='Monitor bump values to allow cache clearing')

parser.add_argument('--syslog', '-s', dest='syslog', default=None)
parser.add_argument('--restart','-r',dest='restart_file', default=None)
parser.add_argument('--check-interval','-i',dest='check_interval',type=float,default=30.0)
parser.add_argument('--memcached-server','-m',dest='memcached_server', default=None)
parser.add_argument('--memcached-key','-k',dest='memcached_key', default="bump")
parser.add_argument('bump_file',type=str)
args = parser.parse_args()

def parse_host_port(value):
    if value != None:
        try:
            value = value.split(':',1)
            if len(value) < 2:
                value.append("514")
            value[1] = int(value[1])
        except Exception as e:
            raise Exception("Bad argument: {0}".format(e)) from None
    return value

args.syslog = parse_host_port(args.syslog)
args.memcached_server = parse_host_port(args.memcached_server)

# Configure logging

logger = logging.getLogger()
logger.setLevel(logging.DEBUG)
if args.syslog != None:
    logger.addHandler(SysLogHandler(address=(args.syslog[0],args.syslog[1])))
else:
    logger.addHandler(StreamHandler(sys.stderr))

# Show configuration

logger.info('bump-detector starting bump-file: "{0}", memcached: "{1}" ({2}) restart-file: "{3}"'
                .format(args.bump_file,args.memcached_server,args.memcached_key,args.restart_file))

# Get value

def get_bump():
    try:
        if "//" in args.bump_file:
            with urllib.request.urlopen(args.bump_file) as response:
                return response.readline()
        else:
            with open(args.bump_file) as f:
                return f.readline()
    except:
        logging.error("Failed to retrieve bump file")
        return None

# Loop and update when necessary

bump = None
while True:
    new_bump = get_bump()
    if bump != new_bump and new_bump != None:
        bump = new_bump
        if bump != None:
            logging.info("bump file changed was '{0}' now '{1}'".format(bump,new_bump))
            if args.restart_file:
                logging.info("scheduling nginx restart")
                with open(args.restart_file,"w+b") as f:
                    pass
            if args.memcached_server:
                logging.info("updating memcached with bump value {0}".format(bump))
    if args.memcached_server:
        mc = base.Client(args.memcached_server)
        mc.set(args.memcached_key, bump)
        mc.quit()
    time.sleep(args.check_interval)
