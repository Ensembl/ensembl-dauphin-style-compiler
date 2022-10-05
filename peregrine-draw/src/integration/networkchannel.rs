use js_sys::Date;
use peregrine_data::{ChannelIntegration, PacketPriority, RequestPacket, ResponsePacket, ChannelSender, BackendNamespace };
use serde_cbor::Value as CborValue;
use crate::util::ajax::PgAjax;
use peregrine_toolkit::url::Url;
use std::future::Future;
use std::pin::Pin;
use std::sync::{ Arc, Mutex };
use crate::util::message::Message;
use peregrine_data::DataMessage;

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
    fn get_sender(&self, prio: &PacketPriority, data: RequestPacket) -> Pin<Box<dyn Future<Output=Result<ResponsePacket,DataMessage>>>> {
        let url = match prio {
            PacketPriority::RealTime => &self.url_hi,
            PacketPriority::Batch => &self.url_lo
        };
        Box::pin(send_wrap(url.clone(),prio.clone(),data,Some(30.),self.cache_buster.clone()))
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

fn add_priority(a: &Url, prio: &PacketPriority, cache_buster: &str) -> Result<Url,Message> {
    let z = a.add_path_segment(match prio {
        PacketPriority::RealTime => "hi",
        PacketPriority::Batch => "lo"
    }).map_err(|_| Message::CodeInvariantFailed(format!("cannot manipulate URL")))?;
    let z = z.add_query_parameter(&format!("stamp={}",cache_buster))
        .map_err(|_| Message::CodeInvariantFailed(format!("cannot manipulate URL")))?;
    Ok(z)
}

async fn send(url: &Url, prio: PacketPriority, data: CborValue, timeout: Option<f64>, cache_buster: &str) -> Result<CborValue,Message> {
    let mut ajax = PgAjax::new("POST",&add_priority(url,&prio,cache_buster)?);
    if let Some(timeout) = timeout {
        ajax.set_timeout(timeout);
    }
    ajax.set_body_cbor(&data)?;
    ajax.get_cbor().await
}

async fn send_wrap(url_str: String, prio: PacketPriority, packet: RequestPacket, timeout: Option<f64>, cache_buster: String) -> Result<ResponsePacket,DataMessage> {
    let url = Url::parse(&url_str).map_err(|e| DataMessage::PacketError(url_str.to_string(),format!("bad url {} '{:?}'",url_str.to_string(),e)))?;
    let data = packet.encode();
    let data = send(&url,prio,data,timeout,&cache_buster).await.map_err(|e| DataMessage::TunnelError(Arc::new(Mutex::new(e))))?;
    let response = ResponsePacket::decode(data).map_err(|e| DataMessage::PacketError(url_str.to_string(),e))?;
    Ok(response)
}
