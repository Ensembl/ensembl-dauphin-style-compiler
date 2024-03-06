from command.coremodel import DataHandler, Panel, DataAccessor
from data.v16.dataalgorithm import data_algorithm
from model.bigbed import get_bigbed

def get_compara_details(
        data_accessor: DataAccessor, panel: Panel, filename: str
    ) -> dict[str,bytearray]:
    chrom = panel.get_chrom(data_accessor)
    data = get_bigbed(data_accessor, chrom.item_path(filename), panel.start, panel.end)
    chrs = []
    starts = []
    lengths = []
    ids = []
    scores = []
    pvalues = []
    for (start, end, rest) in data:
        (name, score, pvalue) = rest.split("\t")
        chrs.append(chrom.name)
        starts.append(start)
        lengths.append(end - start)
        ids.append(name)
        scores.append(int(float(score)))
        pvalues.append(int(float(pvalue)))

    return {
        "chr": data_algorithm("SZ", chrs),
        "start": data_algorithm("NDZRL", starts),
        "length": data_algorithm("NDZRL", lengths),
        "id": data_algorithm("SZ", ids),
        "score": data_algorithm("NZRL", scores),
        "pvalue": data_algorithm("NZRL", pvalues),
    }



class ComparaDataHandler(DataHandler):
    """
    Handle a request for compara bigbed data (conserved elements).

    Args:
        data_accessor (DataAccessor): The means of accessing data
        panel (Panel): The panel (ie genomic location, scale) we want
        scope: extra scope args (here used for datafile name)

    Returns: A data dict (payload for Response object)
    """
    def process_data(
        self, data_accessor: DataAccessor, panel: Panel, scope: dict, accept: str
    ) -> dict[str,bytearray]:
        return get_compara_details(data_accessor, panel, self.get_datafile(scope))
