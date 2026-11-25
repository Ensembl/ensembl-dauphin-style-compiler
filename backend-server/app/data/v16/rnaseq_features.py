"""BED feature intervals and scores for transcriptomic block tracks."""

from collections import Counter
from hashlib import sha256

from command.coremodel import DataHandler, DataAccessor, Panel
from command.exceptionres import DataException
from data.v16.dataalgorithm import data_algorithm
from model.bigbed import get_bigbed


class RNASeqFeaturesDataHandler(DataHandler):
    def process_data(
        self, data_accessor: DataAccessor, panel: Panel, scope: dict, accept: str
    ) -> dict:
        filename = self.get_datafile(scope)
        chrom = panel.get_chrom(data_accessor)
        start, end = max(0, panel.start), min(panel.end, chrom.size)
        rows = (
            get_bigbed(data_accessor, chrom.item_filepath(filename), start, end)
            if start < end
            else []
        )
        fields = {key: [] for key in ("chr", "start", "end", "score", "id")}
        occurrences = Counter()
        for start, end, rest in sorted(rows):
            parts = rest.split("\t")
            if len(parts) < 2:
                raise DataException(
                    "RNA-seq features require BED name and score columns"
                )
            # Include the full record so otherwise identical intervals remain distinct.
            identity = sha256(
                f"{chrom.name}\0{start}\0{end}\0{rest}".encode()
            ).hexdigest()
            ordinal = occurrences[identity]
            occurrences[identity] += 1
            fields["chr"].append(chrom.name)
            fields["start"].append(start)
            fields["end"].append(end)
            fields["score"].append(parts[1])
            fields["id"].append(f"{identity}-{ordinal}")
        return {
            key: data_algorithm("NDZRL" if key in ("start", "end") else "SZ", values)
            for key, values in fields.items()
        }
