#! /usr/bin/env python3

import argparse
import logging
import sys
from logging.handlers import SysLogHandler
from logging import StreamHandler
import subprocess
import time
import os
import shutil
import signal

# script.py [-s syslog-host:syslog-port] [-r restart_file] [-p name-of-pid-file] [-i check-interval] [-c cache-path] path-to-nginx [nginx args]

# Extract and reflect config

parser = argparse.ArgumentParser(description='Monitor and restart nginx when necessary to clear cache')

parser.add_argument('--syslog', '-s', dest='syslog', default=None)
parser.add_argument('--pid-file','-p',dest='pid_file', default=None)
parser.add_argument('--restart','-r',dest='restart_file', default=None)
parser.add_argument('--check-interval','-i',dest='check_interval',type=float,default=30.0)
parser.add_argument('--cache-path','-c',dest='cache_path', default=None)
parser.add_argument('nginx', nargs=argparse.REMAINDER)
args = parser.parse_args()

if args.syslog != None:
    try:
        args.syslog = args.syslog.split(':',1)
        if len(args.syslog) < 2:
            args.syslog.append("514")
        args.syslog[1] = int(args.syslog[1])
    except Exception as e:
        raise Exception("Bad syslog argument: {0}".format(e)) from None

# Configure logging

logger = logging.getLogger()
logger.setLevel(logging.DEBUG)
if args.syslog != None:
    logger.addHandler(SysLogHandler(address=(args.syslog[0],args.syslog[1])))
else:
    logger.addHandler(StreamHandler(sys.stderr))

# Show configuration

logger.info('nginx-wrapper starting nginx-path: "{0}", pid-file: "{1}" restart-file: "{2}"'
                .format(" ".join(args.nginx),args.pid_file,args.restart_file))

# Run nginx

def run():
    global pid
    global process

    process = subprocess.Popen(args.nginx)

    if args.pid_file == None:
        process.wait()
        sys.exit(0)

    # Extract pid etc
    pid = None
    for _ in range(0,30):
        try:
            with open(args.pid_file) as f:
                pid = int(f.readline())
        except:
            pass
        time.sleep(1.)

    if pid == None:
        logger.error("Could not find nginx pidfile at {0}".format(args.pid_file))

# Loop and restart when necessary

def check_pid(pid):
    try:
        os.kill(pid, 0)
    except OSError:
        return False
    else:
        return True

if args.cache_path != None:
    shutil.rmtree(args.cache_path,ignore_errors=True)
run()
while True:
    if not check_pid(pid):
        logger.error("pid has disappeard, nginx has exited!")
        sys.exit(1)
    if args.restart_file != None and os.path.exists(args.restart_file):
        logger.error("restarting nginx")
        os.remove(args.restart_file)
        if os.path.exists(args.restart_file):
            logger.error("cannot delete restart file!")
            sys.exit(1)
        if args.cache_path != None:
            shutil.rmtree(args.cache_path,ignore_errors=True)
        os.kill(pid,signal.SIGTERM)
        process.wait()
        logger.info("exited, good")
        for _ in range(0,30):
            while check_pid(pid):
                logger.warn("old pid={0} still exists".format(pid))
                time.sleep(1.)
        run()
        logger.info("restarted, good pid={0}".format(pid))
    else:
        time.sleep(args.check_interval)
