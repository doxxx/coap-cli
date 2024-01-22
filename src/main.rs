use clap::{Parser, Subcommand};
use coap::CoAPClient;
use coap_lite::option_value::OptionValueU16;
use coap_lite::{CoapOption, CoapRequest, CoapResponse, ContentFormat, RequestType};
use regex::Regex;
use std::io::{Error, ErrorKind, Result};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

const DEFAULT_RECEIVE_TIMEOUT: u64 = 1;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// COAP resource URL
    url: String,

    /// Receive timeout in seconds
    #[arg(global = true, long, default_value_t = DEFAULT_RECEIVE_TIMEOUT)]
    timeout: u64,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Retrieves a representation of a resource
    Get {
        /// Acceptable content formats (comma-separated) for the response
        #[arg(long, value_delimiter = ',')]
        accept: Vec<String>,
    },

    /// Requests that the submitted data be processed
    Post {
        /// Acceptable content formats (comma-separated) for the response
        #[arg(long, value_delimiter = ',')]
        accept: Vec<String>,
        /// Content format of the submitted data
        #[arg(long)]
        content_format: Option<String>,
        /// Resource data
        #[arg(short, long)]
        data: Option<String>,
        /// Path to file containing resource data
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Requests that the resource be updated or created with the submitted data
    Put {
        /// Acceptable content formats (comma-separated) for the response
        #[arg(long, value_delimiter = ',')]
        accept: Vec<String>,
        /// Content format of the submitted data
        #[arg(long)]
        content_format: Option<String>,
        /// Resource data
        #[arg(short, long)]
        data: Option<String>,
        /// Path to file containing resource data
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Requests that the resource be deleted
    Delete {
        /// Acceptable content formats (comma-separated) for the response
        #[arg(long, value_delimiter = ',')]
        accept: Vec<String>,
    },
}

fn parse_coap_url(url: &str) -> Result<(String, Option<u16>, String, Option<String>)> {
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

fn parse_content_format(s: &str) -> Result<ContentFormat> {
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

trait CoapRequestHelper {
    fn set_domain(&mut self, domain: &str);
    fn set_query(&mut self, query: &str);
    fn add_accept(&mut self, accept: ContentFormat);
    fn set_content_format(&mut self, content_format: ContentFormat);
    fn set_data(&mut self, data: Vec<u8>);
}

impl<Endpoint> CoapRequestHelper for CoapRequest<Endpoint> {
    fn set_domain(&mut self, domain: &str) {
        self.message
            .set_option(CoapOption::UriHost, [domain.as_bytes().to_vec()].into());
    }

    fn set_query(&mut self, query: &str) {
        self.message
            .set_option(CoapOption::UriQuery, [query.as_bytes().to_vec()].into());
    }

    fn add_accept(&mut self, accept: ContentFormat) {
        self.message.add_option_as(
            CoapOption::Accept,
            OptionValueU16(content_format_as_u16(accept)),
        );
    }

    fn set_content_format(&mut self, content_format: ContentFormat) {
        self.message.set_options_as(
            CoapOption::ContentFormat,
            [OptionValueU16(content_format_as_u16(content_format))].into(),
        );
    }

    fn set_data(&mut self, data: Vec<u8>) {
        self.message.payload = data;
    }
}

fn coap_request_for_url(url: &str) -> Result<(String, Option<u16>, CoapRequest<SocketAddr>)> {
    let (host, port, path, query) = parse_coap_url(url)?;
    let mut request = CoapRequest::new();
    request.set_domain(&host);
    request.set_path(&path);
    if let Some(query) = query {
        request.set_query(&query);
    }
    Ok((host, port, request))
}

fn coap_send(
    host: &str,
    port: Option<u16>,
    timeout: u64,
    mut request: CoapRequest<SocketAddr>,
) -> Result<CoapResponse> {
    let mut client = CoAPClient::new((host, port.unwrap_or(5683)))?;
    client.set_receive_timeout(Some(Duration::new(timeout, 0)))?;
    client.send2(&mut request)?;
    client.receive2(&mut request)
}

fn coap_get(args: &Args, accept: &[String]) -> Result<()> {
    eprintln!("GET {}", args.url);

    let (host, port, mut request) = coap_request_for_url(&args.url)?;
    request.set_method(RequestType::Get);

    for cf in accept {
        request.add_accept(parse_content_format(cf)?);
    }

    let response = coap_send(&host, port, args.timeout, request)?;

    let content = String::from_utf8_lossy(&response.message.payload);
    eprintln!("{}", response.message.header.get_code());
    println!("{}", content);

    Ok(())
}

fn coap_post(
    args: &Args,
    accept: &[String],
    content_format: Option<&str>,
    data: &str,
) -> Result<()> {
    eprintln!("POST {}", args.url);

    let (host, port, mut request) = coap_request_for_url(&args.url)?;
    request.set_method(RequestType::Post);

    for cf in accept {
        request.add_accept(parse_content_format(cf)?);
    }

    if let Some(cf) = content_format {
        request.set_content_format(parse_content_format(cf)?);
    }

    request.set_data(data.as_bytes().to_vec());

    let response = coap_send(&host, port, args.timeout, request)?;

    let content = String::from_utf8_lossy(&response.message.payload);
    eprintln!("{}", response.message.header.get_code());
    println!("{}", content);

    Ok(())
}

fn coap_put(
    args: &Args,
    accept: &[String],
    content_format: Option<&str>,
    data: &str,
) -> Result<()> {
    eprintln!("PUT {}", args.url);

    let (host, port, mut request) = coap_request_for_url(&args.url)?;
    request.set_method(RequestType::Put);

    for cf in accept {
        request.add_accept(parse_content_format(cf)?);
    }

    if let Some(cf) = content_format {
        request.set_content_format(parse_content_format(cf)?);
    }

    request.set_data(data.as_bytes().to_vec());

    let response = coap_send(&host, port, args.timeout, request)?;

    let content = String::from_utf8_lossy(&response.message.payload);
    eprintln!("{}", response.message.header.get_code());
    println!("{}", content);

    Ok(())
}

fn coap_delete(args: &Args, accept: &[String]) -> Result<()> {
    eprintln!("DELETE {}", args.url);

    let (host, port, mut request) = coap_request_for_url(&args.url)?;
    request.set_method(RequestType::Delete);

    for cf in accept {
        request.add_accept(parse_content_format(cf)?);
    }

    let response = coap_send(&host, port, args.timeout, request)?;

    let content = String::from_utf8_lossy(&response.message.payload);
    eprintln!("{}", response.message.header.get_code());
    println!("{}", content);

    Ok(())
}

fn load_data_file(file: &PathBuf) -> Result<String> {
    if !file.is_file() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Error: path must be file: {}", file.to_str().unwrap()),
        ));
    }

    let data = std::fs::read_to_string(&file)?;
    Ok(data)
}

fn execute_command(args: &Args) -> Result<()> {
    match &args.command {
        Commands::Get { accept } => coap_get(args, accept),
        Commands::Post {
            accept,
            content_format,
            data,
            file,
        } => {
            let data = {
                if let Some(data) = data {
                    Some(data.to_owned())
                } else if let Some(file) = file {
                    Some(load_data_file(file)?)
                } else {
                    None
                }
            };

            let data = data.ok_or(Error::new(
                ErrorKind::InvalidInput,
                "must specify either data string or file path",
            ))?;

            coap_post(args, accept, content_format.as_deref(), &data)
        }
        Commands::Put {
            accept,
            content_format,
            data,
            file,
        } => {
            let data = {
                if let Some(data) = data {
                    Some(data.to_owned())
                } else if let Some(file) = file {
                    Some(load_data_file(file)?)
                } else {
                    None
                }
            };

            let data = data.ok_or(Error::new(
                ErrorKind::InvalidInput,
                "must specify either data string or file path",
            ))?;

            coap_put(args, accept, content_format.as_deref(), &data)
        }
        Commands::Delete { accept } => coap_delete(args, accept),
    }
}

fn main() {
    let cli = Args::parse();

    if let Err(err) = execute_command(&cli) {
        eprintln!("ERROR: {}", err);
    }
}
