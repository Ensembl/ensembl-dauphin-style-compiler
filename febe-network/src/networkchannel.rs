use js_sys::Date;
use peregrine_data::{ChannelIntegration, PacketPriority, MaxiRequest, ChannelSender, BackendNamespace, ChannelMessageDecoder, MaxiResponse, null_payload };
use peregrine_toolkit::cbor::{cbor_into_drained_map, cbor_into_bytes};
use peregrine_toolkit::error::Error;
use serde_cbor::Deserializer;
use serde::de::{DeserializeSeed};
use crate::ajax::PgAjax;
use peregrine_toolkit::url::Url;
use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::{ Arc };
use inflate::inflate_bytes_zlib;

/* Network URL syntax specifies one or two http or https endpoint. If two are specified, then
 * they are for high and low priority requests. A backend namespace can also be specified.
 * If a single URL is specified it can be a bare url. After the fragment identifier is an
 * optional backend namespace formatted as two colon-separated strings.
 * 
 * If two URLs are given they are specified as a space-separated pair.
 */

pub struct NetworkChannelSender {
    cache_buster: String,
    url_hi: String,
    url_lo: String
}

impl ChannelSender for NetworkChannelSender {
    fn get_sender(&self, prio: &PacketPriority, data: MaxiRequest, decoder: ChannelMessageDecoder) -> Pin<Box<dyn Future<Output=Result<MaxiResponse,Error>>>> {
        let url = match prio {
            PacketPriority::RealTime => &self.url_hi,
            PacketPriority::Batch => &self.url_lo
        };
        Box::pin(send_wrap(url.clone(),prio.clone(),data,Some(30.),self.cache_buster.clone(),decoder))
    }

    fn deserialize_data(&self, _payload: &dyn Any, bytes: Vec<u8>) -> Result<Option<Vec<(String,Vec<u8>)>>,String> {
        let bytes = inflate_bytes_zlib(&bytes).map_err(|e| format!("cannot uncompress: {}",e))?;
        let value = serde_cbor::from_slice(&bytes).map_err(|e| format!("corrupt payload/A: {}",e))?;
        let mut value = cbor_into_drained_map(value).map_err(|e| format!("corrupt payload/B: {}",e))?;
        let value = value.drain(..).map(|(k,v)| Ok((k,cbor_into_bytes(v)?))).collect::<Result<Vec<_>,String>>()?;
        Ok(Some(value))
    }
}

pub struct NetworkChannel {
    cache_buster: String,
}

impl NetworkChannel {
    pub fn new() -> NetworkChannel {
        let cache_buster = format!("{:016x}",Date::now() as u64);
        NetworkChannel {
            cache_buster,
        }
    }
}

fn parse_backend_namespace(name: &str) -> BackendNamespace {
    let mut authority = "";
    let mut name = name;
    if let Some(first_colon) = name.find(":") {
        (authority,name) = name.split_at(first_colon);
    }
    BackendNamespace::new(authority,name)
}

fn split_fragment(url: &str) -> (&str,Option<&str>) {
    if let Some(hash_pos) = url.find("#") {
        let (a,b) = url.split_at(hash_pos);
        (a,Some(b))
    } else {
        (url,None)
    }
}

impl ChannelIntegration for NetworkChannel {
    fn make_channel(&self, name: &str) -> Option<(Arc<dyn ChannelSender>,Option<BackendNamespace>)> {
        if !name.starts_with("http:") && !name.starts_with("https:") {
            return None;
        }
        /* separate into hi/lo if present. */
        let name = name.trim();
        let (url_hi,url_lo) = if let Some(space_pos) = name.find(char::is_whitespace) {
            name.split_at(space_pos)
        } else {
            (name,name)
        };
        /* split off fragment if present */
        let (url_hi,frag_hi) = split_fragment(url_hi);      
        let (url_lo,frag_lo) = split_fragment(url_lo);   
        let ns_hi = frag_hi.map(|x| parse_backend_namespace(x));
        let ns_lo = frag_lo.map(|x| parse_backend_namespace(x));
        /* If both have namespaces, check they match */
        if let (Some(frag_hi),Some(frag_lo)) = (&ns_hi,&ns_lo) {
            if frag_hi != frag_lo {
                return None; // XXX error todo!()
            }
        }
        /* build */
        let sender = NetworkChannelSender {
            cache_buster: self.cache_buster.clone(),
            url_hi: url_hi.to_string(),
            url_lo: url_lo.to_string()
        };
        Some((Arc::new(sender),ns_hi))
    }
}

fn add_priority(a: &Url, prio: &PacketPriority, cache_buster: &str) -> Url {
    let z = a.add_path_segment(match prio {
        PacketPriority::RealTime => "hi",
        PacketPriority::Batch => "lo"
    });
    z.add_query_parameter(&format!("stamp={}",cache_buster))
}

async fn send(url: &Url, prio: PacketPriority, data: Vec<u8>, timeout: Option<f64>, cache_buster: &str) -> Result<Vec<u8>,Error> {
    let mut ajax = PgAjax::new("POST",&add_priority(url,&prio,cache_buster));
    if let Some(timeout) = timeout {
        ajax.set_timeout(timeout);
    }
    ajax.set_body(data);
    ajax.get_cbor().await
}

async fn send_wrap(url_str: String, prio: PacketPriority, packet: MaxiRequest, timeout: Option<f64>, cache_buster: String, decoder: ChannelMessageDecoder) -> Result<MaxiResponse,Error> {
    let url = Error::oper_r(Url::parse(&url_str),&format!("bad_url {}",url_str))?;
    let data = Error::oper_r(serde_cbor::to_vec(&packet),"packet error")?;
    let data = send(&url,prio,data,timeout,&cache_buster).await?;
    let mut deserializer = Deserializer::from_slice(&data);
    let deserialize = decoder.serde_deserialize_maxi(null_payload());
    let response = Error::oper_r(deserialize.deserialize(&mut deserializer),"packet error")?;
    Error::oper_r(deserializer.end(),"packet error")?;
    Ok(response)
}
