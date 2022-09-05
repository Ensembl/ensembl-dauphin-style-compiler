# Introduction

app/data contains the most important files in the genome browser backend. They convert an
incoming data request into a response in the format expected by the genome browser. There
are other types of requests as well as data requests which are handled elsewhere in the
codebase (for bootstrapping, etc and vaious housekeeping reasons), but the data request type is the one to return all of the information to be displayed.

The genome browser accepts data values in a hash with string keys. The values are binary
blobs. Each binary blob encodes a compressed form of a one-dimensional data sequence of
a single simple type.

For exmaple to encode a gene there might be a stream for starts, a stream for ends, a stream for labels, and so on. Each stream is compressed into an efficient form depending on what contents it holds. For exmaple, start co-ordinates could be stored as offsets from the previous start, and so on.

So the methods in these files typically follow the following pattern:

1. based on the parameters find some file or url (or, in the future, API, etc) which has this data;

2. break this data down into a group of one-dimensional streams (eg in python lists);

3. exploit the redundency in the data to make it more compressible (by calling functions called things like lesqlite2, zigzag, etc) which take one such stream and generate a more compressible one;

4. call compress to actually compress the stream;

5. build these streams into a response.

gc.py is a good one to look at to find out what's going on. (genedata.py is the most important but a horrible mess that needs lots of TLC). Note how _get_gc gets the contents of some bigwig files manipulates the values a little and then generates two streams: the actual data in a stream called "values" and the start and end coordinates in a stream called "range". These are transformed in various ways and then passed to compress.

The transformations are in numbers.py (bad name) and need documenting: lesqlite2 converts small positive numbers into a very compact set of bytes (the algorithm is called this because it was first defined in lesqlite2); zigzag maps small +ve or -ve numbers into small +ve numbers; delta replaces numbers with their difference from the previous (great for start coords to make the numbers smaller); compress makes the resulting smaller using the zlib (gzip) algorithm; classify converts strings which represent small sets of values (eg biotypes) into a list of strings containing all used string values and a list of ints giving which should be used where, eg ("A","A","B","A","B") -> ("A","B"), (0,0,1,0,1). These can be sent separately and be much smaller than the original and also easier to manipulate on the other end.
