fn need_bytes(data: &[u8], i: &mut usize, n : usize) -> Result<(),()> {
    if *i+n > data.len() {
        return Err(())
    }
    *i += n;
    Ok(())
}

pub fn lesqlite2_decode(data: &[u8]) -> Result<Vec<f64>,()> {
    let mut out = vec![];
    let mut i = 0;
    while i < data.len() {
        if data[i] < 178 {
            need_bytes(data,&mut i,1)?;
            out.push(data[i-1] as f64);
        } else if data[i] < 242 {
            need_bytes(data,&mut i,2)?;
            out.push((((data[i-2] as u64-178)<<8) + (data[i-1] as u64) + 178_u64) as f64);
        } else if data[i] < 250 {
            need_bytes(data,&mut i,3)?;
            let v : u64 = 
                ((data[i-3] as u64-242_u64)<<16_u64) +
                ((data[i-1] as u64) << 8_u64) +
                (data[i-2] as u64) +
                16562_u64
            ;
            out.push(v as f64);
        } else {
            need_bytes(data,&mut i,1)?;
            let n = (data[i-1] - 247) as usize;
            need_bytes(data,&mut i,n)?;
            let mut v = 0;
            let mut m = 0;
            for j in 0..n {
                v += (data[i-n+j] as u64) << m;
                m += 8;
            }
            out.push(v as f64);
        }
    }
    Ok(out)
}
