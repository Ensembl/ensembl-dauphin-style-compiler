from command.coremodel import DataHandler, Panel, DataAccessor
from data.v16.dataalgorithm import data_algorithm
from model.bigbed import get_bigbed_fields


def get_compara_details(
        data_accessor: DataAccessor, panel: Panel, filename: str
    ) -> dict[str,bytearray]:
    fields = get_bigbed_fields(
        data_accessor, panel, filename,
        ["id", "score", "pvalue"]
    )
    scores = [int(float(score)) for score in fields["score"]]
    pvalues = [int(float(pvalue)) for pvalue in fields["pvalue"]]
    lengths = [end - start for start, end in zip(fields["start"], fields["end"])]

    return {
        "chr": data_algorithm("SZ", fields["chr"]),
        "start": data_algorithm("NDZRL", fields["start"]),
        "length": data_algorithm("NDZRL", lengths),
        "id": data_algorithm("SZ", fields["id"]),
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
