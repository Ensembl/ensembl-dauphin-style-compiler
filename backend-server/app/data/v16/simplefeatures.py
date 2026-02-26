from command.coremodel import DataHandler, Panel, DataAccessor
from data.v16.dataalgorithm import data_algorithm
from model.bigbed import get_bigbed_fields


def get_features(
    data_accessor: DataAccessor, panel: Panel, filename: str
) -> dict[str, bytearray]:
    fields = get_bigbed_fields(
        data_accessor, panel, filename,
        ["strand", "analysis"]
    )

    return {
        "chr": data_algorithm("SZ", fields["chr"]),
        "start": data_algorithm("NDZRL", fields["start"]),
        "end": data_algorithm("NDZRL", fields["end"]),
        "strand": data_algorithm("SZ", fields["strand"]),
        "analysis": data_algorithm("SZ", fields["analysis"]),
    }


class SimpleFeaturesDataHandler(DataHandler):
    """
    Handle a request for fetching data (simple features) from bigbed file.

    Args:
        data_accessor (DataAccessor): The means of accessing data
        panel (Panel): The panel (ie genomic location, scale) we want
        scope: extra scope args (here used for datafile name)

    Returns: A data dict (payload for Response object)
    """

    def process_data(
        self, data_accessor: DataAccessor, panel: Panel, scope: dict, accept: str
    ) -> dict[str, bytearray]:
        return get_features(data_accessor, panel, self.get_datafile(scope))
