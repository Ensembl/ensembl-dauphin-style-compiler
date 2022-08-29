from command.coremodel import DataHandler, Panel, DataAccessor
from command.response import Response
from model.bigbed import get_bigbed
from model.transcriptfile import TranscriptFileLine
from ..numbers import lesqlite2, compress
from .genedata import extract_gene_overview_data
from model.datalocator import AccessItem
from ncd import NCDRead

def _get_approx_location(data_accessor: DataAccessor, genome: str, id):
    key = "focus:{}:{}".format(genome,id)
    accessor = data_accessor.resolver.get(AccessItem("jump"))
    jump_ncd = NCDRead(accessor.ncd())
    value = jump_ncd.get(key.encode("utf-8"))
    if value != None:
        parts = value.decode('utf-8').split("\t")
        if len(parts) == 3:
            on_stick = "{}:{}".format(genome,parts[0])
            return (on_stick,int(parts[1]),int(parts[2]))
    return None

def _remove_version(id: str):
    return id.rsplit('.',1)[0]

def _get_exact_location(data_accessor: DataAccessor, stick, gene_id, approx_start, approx_end):
    chrom = data_accessor.data_model.stick(data_accessor,stick)
    if chrom == None:
        return Response(1,"Unknown chromosome {0}".format(stick))
    item = chrom.item_path("transcripts")
    data = get_bigbed(data_accessor,item,approx_start,approx_end)
    for line in data:
        line = TranscriptFileLine(line)
        line_gene_id = _remove_version(line.gene_id)
        if line_gene_id == gene_id:
            return (line.gene_start,line.gene_end,1 if line.strand == '+' else 0)
    return None

class GeneLocationHandler8(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel,scope) -> Response:
        genome = scope.get("genome",[])
        id = scope.get("id",[])
        out = []
        stick = None
        location = None
        if len(id) > 0:
            approx = _get_approx_location(data_accessor,genome[0],id[0])
            if approx is not None:
                stick = approx[0]
                exact = _get_exact_location(data_accessor,approx[0],id[0],approx[1],approx[2])
                if exact is not None:
                    location = exact
        chrom = data_accessor.data_model.stick(data_accessor,panel.stick)
        if location is not None:
            out = extract_gene_overview_data(data_accessor,chrom,exact[0],exact[1],True)
        else:
            out = extract_gene_overview_data(data_accessor,chrom,0,0,True)
            location = []
        out["location"] = compress(lesqlite2(location))
        if stick is not None:
            out["stick"] = compress(stick)
        return Response(5,{ 'data': out })
