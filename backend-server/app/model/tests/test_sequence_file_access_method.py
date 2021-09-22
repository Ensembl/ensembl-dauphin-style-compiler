
from model.datalocator import AccessItem, FileAccessMethod

# Specify genome, chromosome and base_path as appropriate
GENOME = 'homo_sapiens_GCA_000001405.28'
CHROMOSOME = '11eeaa801f6b0e2e36a1138616b8ee9a'
BASE_PATH = '/usr/data/newsite/dev/genome_browser/s3-data/'

# Adjust offset, size as appropriate depending on GENOME, CHROMOSOME etc
OFFFSET = 30000000
SIZE = 256


ai = AccessItem('seqs', genome=GENOME, chromosome=CHROMOSOME)
 
fam = FileAccessMethod(BASE_PATH, ai)
print('Reading File : ', fam.file)

out = fam.get(OFFSET, SIZE)
# Prints the sequences
print(out)
