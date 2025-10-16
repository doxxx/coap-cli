use coap::request::RequestBuilder;
use coap_lite::option_value::OptionValueU16;
use coap_lite::{CoapOption, CoapRequest, ContentFormat, RequestType};
use regex::Regex;
use std::io::{Error, ErrorKind, Result};
use std::net::SocketAddr;
use url::Url;

pub fn parse_coap_url(url: &str) -> Result<(String, Option<u16>, String, Option<String>)> {
    let url_params = match Url::parse(url) {
        Ok(url_params) => url_params,
        Err(_) => return Err(Error::new(ErrorKind::InvalidInput, "url error")),
    };

    let host = match url_params.host_str() {
        Some("") => return Err(Error::new(ErrorKind::InvalidInput, "host error")),
        Some(h) => h,
        None => return Err(Error::new(ErrorKind::InvalidInput, "host error")),
    };
    let host = Regex::new(r"^\[(.*?)]$")
        .unwrap()
        .replace(&host, "$1")
        .to_string();

    let port = url_params.port();

    let path = url_params.path().to_string();

    let query = url_params.query().map(|q| q.to_string());

    return Ok((host, port, path, query));
}

pub fn parse_content_format(s: &str) -> Result<ContentFormat> {
    if let Ok(num) = s.parse::<usize>() {
        ContentFormat::try_from(num).map_err(|_| {
            Error::new(
                ErrorKind::InvalidInput,
                format!("invalid content format number: {}", s),
            )
        })
    } else {
        match s {
            "text/plain" => Ok(ContentFormat::TextPlain),
            "application/json" => Ok(ContentFormat::ApplicationJSON),
            "application/xml" => Ok(ContentFormat::ApplicationXML),
            "application/cbor" => Ok(ContentFormat::ApplicationCBOR),
            "application/octet-stream" => Ok(ContentFormat::ApplicationOctetStream),
            // TODO: more content formats
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("unsupported content format string: {}", s),
            )),
        }
    }
}

fn content_format_as_u16(cf: ContentFormat) -> u16 {
    let num = usize::from(cf);
    let num: u16 = num.try_into().unwrap();
    num
}

pub fn build_coap_request_for_url(
    url: &str,
    method: RequestType,
    payload: Option<Vec<u8>>,
    content_format: Option<ContentFormat>,
    accept: Option<Vec<ContentFormat>>,
) -> Result<CoapRequest<SocketAddr>> {
    let (host, _, path, query) = parse_coap_url(url)?;
    let mut rb = RequestBuilder::new(&path, method);
    rb = rb.domain(host);
    if let Some(q) = query {
        rb = rb.queries(vec![q.as_bytes().to_vec()]);
    }
    rb = rb.data(payload);
    let mut options = vec![];
    if let Some(cf) = content_format {
        options.push((
            CoapOption::ContentFormat,
            OptionValueU16(content_format_as_u16(cf)).into(),
        ));
    }
    if let Some(a) = accept {
        for a in a {
            options.push((
                CoapOption::Accept,
                OptionValueU16(content_format_as_u16(a)).into(),
            ));
        }
    }
    rb = rb.options(options);
    Ok(rb.build())
}
