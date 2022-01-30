import { sleep } from 'k6'
import http from 'k6/http'

/*
00000000  a3 67 63 68 61 6e 6e 65  6c 82 00 71 2f 61 70 69 |.gchannel..q / api|
00000010  2f 62 72 6f 77 73 65 72  2f 64 61 74 61 67 76 65 | /browser/datagve |
00000020  72 73 69 6f 6e a1 63 65  67 73 06 68 72 65 71 75 | rsion.cegs.hrequ |
00000030  65 73 74 73 81 83 00 00  f6 | ests.....|
*/

let request = "£gchannel\u0082\u0000q/api/browser/datagversion¡cegs\u0006hrequests\u0081\u0083\u0000\u0000ö";

let data = [...request].map(c => c.charCodeAt(0));
let request2 = new Uint8Array(data);
console.log(request2);
// See https://k6.io/docs/using-k6/options
export const options = {
  stages: [
    { duration: '1m', target: 5 },
  ],
  ext: {
    loadimpact: {
      distribution: {
        'amazon:us:ashburn': { loadZone: 'amazon:us:ashburn', percent: 100 },
      },
    },
  },
}

export default function main() {
  let response = http.post('http://localhost:3333/api/data/hi?stamp=k6',
    request2.buffer,    
{
      headers: {
        'content-type': 'application/cbor',
      }
    }
  )
  sleep(1)
}
