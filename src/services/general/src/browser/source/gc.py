from ..data import get_bigwig_data

POINTS = 40

class BAISPercGC(object):
    def __init__(self,gc_file):
        self.gc_file = gc_file
    
    def gc(self,leaf):
        steps = 500
        y = get_bigwig_data(self.gc_file,leaf.chrom,leaf.start,leaf.end,steps)
        y = [ int((y or 0)*POINTS/100) for y in y ]
        return [[leaf.start,leaf.end],y,[0.5],[1/POINTS]]
