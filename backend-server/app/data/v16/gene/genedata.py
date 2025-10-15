import os.path

from ncd import NCDRead

from command.coremodel import DataHandler, Panel, DataAccessor
from data.v16.dataalgorithm import data_algorithm
from data.v16.gene.transcriptfilter import filter_lines_by_criteria
from data.v16.gene.transcriptorder import sort_data_by_transcript_priority
from model.bigbed import get_bigbed
from model.datalocator import AccessItem
from model.transcriptfile import TranscriptFileLine
from tangle.tangle import TangleFactory

# We might be asked for very zoomed-in views even when zoomed out for example if we are zoomed
# in and want info on a whole transcipt object. Even in this case we don't want to emit very big
# data sets such as sequences.
MAX_SEQ_SCALE = 10

def accept_to_tangling_config(accept:str) -> dict[str,bool]:
    """Converts the accept parameter (from incoming requests) to a tangle config (for outgoing requests)

    Args:
        accept (str): param from incoming requests.
        Possible values: "dump"|"uncompressed"|"release" (default)

    Returns:
        dict[str,bool]: config for data tangler
    """

    compress = True
    to_bytes = True
    if accept == "dump":
        compress = False
        to_bytes = False
    elif accept == "uncompressed":
        compress = False
    return { 'compress': compress, 'to_bytes': to_bytes }

class TangleProcessor:
    def plus_strand(self, data):
        return int(data=="+")

processor = TangleProcessor()

TANGLE_FACTORY = TangleFactory()

TR_TANGLE_PATH = os.path.join(os.path.dirname(__file__),"transcript-tangle.toml")
TANGLE_NO_EXON = TANGLE_FACTORY.make_from_tomlfile(TR_TANGLE_PATH,["block"],processor)
TANGLE_EXON = TANGLE_FACTORY.make_from_tomlfile(TR_TANGLE_PATH,["exon"],processor)

OV_TANGLE_PATH = os.path.join(os.path.dirname(__file__),"overview-tangle.toml")
TANGLE_OVERVIEW = TANGLE_FACTORY.make_from_tomlfile(OV_TANGLE_PATH,[],processor)
TANGLE_OVERVIEW_WITH_IDS = TANGLE_FACTORY.make_from_tomlfile(OV_TANGLE_PATH,["ids"],processor)

def get_approx_location(data_accessor: DataAccessor, genome_id: str, id: str):
    species = data_accessor.data_model.species(genome_id)
    if species != None:
        key = "focus:gene:{}:{}".format(genome_id,id)
        accessor = data_accessor.resolver.get(AccessItem("jump",genome_id))
        jump_ncd = NCDRead(accessor.ncd())
        value = jump_ncd.get(key.encode("utf-8"))
        if value != None:
            parts = value.decode('utf-8').split("\t")
            if len(parts) == 3:
                on_stick = "{}:{}".format(genome_id,parts[0])
                return (on_stick,int(parts[1]),int(parts[2]))
    return (None,None,None)

# We need to return all the data for the focus gene wherever we are (except for the sequence) as
# transcript configuration, ordering, etc is still relevant.
def update_panel_from_id(data_accessor: DataAccessor, panel: Panel, for_id: tuple[str,str]):
    (stick,start,end) = get_approx_location(data_accessor,for_id[0],for_id[1])
    if stick is not None:
        panel.stick = stick
        panel.start = start
        panel.end = end
    else:
        # can't find one but expected one. This data will be junked on the FE, so keep it small
        panel.end = panel.start + 1

# For non-focus genes we need to make sure we include all the transcripts even ones which
# start&end completely off-panel.
def extract_data_for_lines(data, for_id: tuple[str,str]|None, expanded: list[str]) -> list:
    lines = [ TranscriptFileLine(row) for row in data ]

    # sort the data
    lines = sort_data_by_transcript_priority(lines)
    max_tr = 5 if for_id is None else None

    # filter the data
    lines = filter_lines_by_criteria(lines, for_id, max_tr, expanded)
    return lines

def extract_gene_data(
        data_accessor: DataAccessor, panel: Panel, include_exons: bool, for_id: tuple[str,str]|None, expanded: list[str], accept: str
    ) -> dict[str, bytearray]:
    # fix location
    if for_id is not None:
        update_panel_from_id(data_accessor, panel, for_id)
    chrom = panel.get_chrom(data_accessor)
    # get the data
    item = chrom.item_path("transcripts")
    # serialize the data
    tangle = TANGLE_EXON if include_exons else TANGLE_NO_EXON
    data = get_bigbed(data_accessor, item, panel.start, panel.end)
    lines = extract_data_for_lines(data,for_id, expanded)
    out = tangle.run2({},{ "tr_bigbed": lines },**accept_to_tangling_config(accept))
    out["stick"] = ("SZ",[panel.stick])
    # flag as invariant if by id
    out = { k: data_algorithm(v[0],v[1]) for (k,v) in out.items() }
    if for_id is not None:
        out['__invariant'] = True
    return out

def extract_gene_overview_data(data_accessor: DataAccessor, panel: Panel, with_ids: bool, accept: str) -> dict[str, bytearray]:
    item = panel.get_chrom(data_accessor).item_path("transcripts")
    data = get_bigbed(data_accessor, item, panel.start, panel.end)
    tangle = TANGLE_OVERVIEW_WITH_IDS if with_ids else TANGLE_OVERVIEW
    lines = [ TranscriptFileLine(x) for x in data ]
    out = tangle.run2({}, { "tr_bigbed": lines }, **accept_to_tangling_config(accept))
    out = { k: data_algorithm(v[0],v[1]) for (k,v) in out.items() }
    return out

def for_id(scope):
    genome_id = scope.get("genome",[""])[0]
    obj_id = scope.get("id",[""])[0]
    if genome_id and obj_id:
        return (genome_id,obj_id)
    else:
        return None

class TranscriptDataHandler16(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope: dict, accept:str) -> dict:
        return extract_gene_data(data_accessor, panel, True, for_id(scope), scope.get("expanded",[]), accept)

class GeneDataHandler16(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope: dict, accept:str) -> dict:
        return extract_gene_data(data_accessor, panel, False, for_id(scope), scope.get("expanded",[]), accept)

class GeneOverviewDataHandler16(DataHandler):
    def process_data(self, data_accessor: DataAccessor, panel: Panel, scope: dict, accept:str) -> dict:
        return extract_gene_overview_data(data_accessor, panel, scope.get("with_ids",[False])[0], accept)
