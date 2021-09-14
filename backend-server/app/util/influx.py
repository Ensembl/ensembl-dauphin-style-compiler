from logging import log
import logging
import socket
import collections
from core.config import TELEGRAF_HOST, TELEGRAF_PORT

class ResponseMetrics:
    def __init__(self,priority):
        self.cache_hits = 0
        self.cache_misses = 0
        self.cache_hits_bytes = 0
        self.cache_misses_bytes = 0
        self.count_packets = 0
        self.runtime_num = collections.defaultdict(float) # [(name,scale)] time
        self.runtime_denom = collections.defaultdict(float) # [(name,scale)] count
        self.priority = priority

    def send(self):
        if TELEGRAF_HOST == None:
            return
        cache_total = self.cache_misses + self.cache_hits
        cache_total_bytes= self.cache_misses_bytes + self.cache_hits_bytes
        lines = ""
        lines += "memcache,priority={0} hits={1},misses={2},ratio_memcached={3},hits_bytes={4},misses_bytes={5},ratio_memcached_bytes={6}\n".format(
            self.priority,
            self.cache_hits,self.cache_misses,
            self.cache_hits/cache_total if cache_total > 0 else 0,
            self.cache_hits_bytes,self.cache_misses_bytes,
            self.cache_hits_bytes/cache_total_bytes if cache_total_bytes > 0 else 0,
        )
        lines += "packets-per-request,priority={0} count={1}\n".format(self.priority,self.count_packets)
        for (key,count) in self.runtime_denom.items():
            avg_time = self.runtime_num[key] / count
            lines += "be-runtime,name={0},scale={1} runtime={2}\n".format(key[0],key[1],avg_time)
        send_to_telegraf(lines)

def send_to_telegraf(lines):
    if TELEGRAF_HOST == None:
        return
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.connect((TELEGRAF_HOST,int(TELEGRAF_PORT)))
        s.settimeout(1)
        s.sendall(lines.encode("utf-8"))
        s.close()
    except:
        logging.warn("discarded stats: could not contact telegraf")
