"""BED feature intervals and scores for transcriptomic block tracks."""

from collections import Counter
from hashlib import sha256
import math

from command.coremodel import DataHandler, DataAccessor, Panel
from command.exceptionres import DataException
from data.v16.dataalgorithm import data_algorithm
from model.bigbed import get_bigbed


SHADE_COUNT = 8
SUMMARY_BINS = 256
SUMMARY_MIN_WIDTH = 1 << 16


def score_shade(score: str) -> int:
    """Use fixed BED score bands so shades remain comparable across panels."""
    try:
        value = float(score)
    except (TypeError, ValueError):
        return 0
    if not math.isfinite(value) or value <= 0:
        return 0
    return round((SHADE_COUNT - 1) * math.log1p(min(value, 1000)) / math.log1p(1000))


def bed_score(rest: str) -> str:
    parts = rest.split("\t", 2)
    if len(parts) < 2:
        raise DataException("RNA-seq features require BED name and score columns")
    return parts[1]


def summarize_features(rows, panel_start: int, panel_end: int, chrom_size: int):
    """Bin overlapping features into one band, retaining the darkest score per bin."""
    width = panel_end - panel_start
    step = max(1, (width + SUMMARY_BINS - 1) // SUMMARY_BINS)
    bin_count = (width + step - 1) // step
    # Difference arrays avoid visiting every covered bin for each long feature.
    changes = [[0] * (bin_count + 1) for _ in range(SHADE_COUNT)]
    for start, end, rest in rows:
        start, end = max(0, panel_start, start), min(chrom_size, panel_end, end)
        if start >= end:
            continue
        first = max(0, (start - panel_start) // step)
        last = min(bin_count - 1, (end - 1 - panel_start) // step)
        shade = score_shade(bed_score(rest))
        changes[shade][first] += 1
        changes[shade][last + 1] -= 1

    fields = {key: [] for key in ("summary_start", "summary_end", "summary_shade")}
    active = [0] * SHADE_COUNT
    for index in range(bin_count):
        for shade in range(SHADE_COUNT):
            active[shade] += changes[shade][index]
        shade = next(
            (shade for shade in reversed(range(SHADE_COUNT)) if active[shade]), None
        )
        if shade is None:
            continue
        start = max(0, panel_start + index * step)
        end = min(chrom_size, panel_end, panel_start + (index + 1) * step)
        if (
            fields["summary_end"]
            and fields["summary_end"][-1] == start
            and fields["summary_shade"][-1] == shade
        ):
            fields["summary_end"][-1] = end
        else:
            fields["summary_start"].append(start)
            fields["summary_end"].append(end)
            fields["summary_shade"].append(shade)
    return fields


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
        fields = {
            key: []
            for key in (
                "chr",
                "start",
                "end",
                "score",
                "id",
                "shade",
                "summary_start",
                "summary_end",
                "summary_shade",
            )
        }
        if panel.end - panel.start >= SUMMARY_MIN_WIDTH:
            fields.update(summarize_features(rows, panel.start, panel.end, chrom.size))
        else:
            occurrences = Counter()
            for start, end, rest in sorted(rows):
                score = bed_score(rest)
                # Include the full record so otherwise identical intervals remain distinct.
                identity = sha256(
                    f"{chrom.name}\0{start}\0{end}\0{rest}".encode()
                ).hexdigest()
                ordinal = occurrences[identity]
                occurrences[identity] += 1
                fields["chr"].append(chrom.name)
                fields["start"].append(start)
                fields["end"].append(end)
                fields["score"].append(score)
                fields["id"].append(f"{identity}-{ordinal}")
                fields["shade"].append(score_shade(score))
        return {
            key: data_algorithm(
                "SZ" if key in ("chr", "score", "id") else "NDZRL", values
            )
            for key, values in fields.items()
        }
