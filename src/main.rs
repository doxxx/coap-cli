mod coap_helper;

use clap::{Parser, Subcommand};
use coap::UdpCoAPClient;
use coap_lite::RequestType;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;
use std::time::Duration;

use coap_helper::*;

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

async fn coap_get(client: &mut UdpCoAPClient, args: &Args, accept: &[String]) -> Result<()> {
    eprintln!("GET {}", args.url);

    let accept_cf = accept.iter().map(|a| parse_content_format(a)).collect::<Result<Vec<_>>>()?;
    let request = build_coap_request_for_url(&args.url, RequestType::Get, None, None, Some(accept_cf))?;
    let response = client.send(request).await?;

    let content = String::from_utf8_lossy(&response.message.payload);
    eprintln!("{}", response.message.header.get_code());
    println!("{}", content);

    Ok(())
}

async fn coap_post(
    client: &mut UdpCoAPClient,
    args: &Args,
    accept: &[String],
    content_format: Option<&str>,
    data: &str,
) -> Result<()> {
    eprintln!("POST {}", args.url);

    let cf = content_format.map(parse_content_format).transpose()?;
    let accept_cf = accept.iter().map(|a| parse_content_format(a)).collect::<Result<Vec<_>>>()?;
    let request = build_coap_request_for_url(&args.url, RequestType::Post, Some(data.as_bytes().to_vec()), cf, Some(accept_cf))?;
    let response = client.send(request).await?;

    let content = String::from_utf8_lossy(&response.message.payload);
    eprintln!("{}", response.message.header.get_code());
    println!("{}", content);

    Ok(())
}

async fn coap_put(
    client: &mut UdpCoAPClient,
    args: &Args,
    accept: &[String],
    content_format: Option<&str>,
    data: &str,
) -> Result<()> {
    eprintln!("PUT {}", args.url);

    let cf = content_format.map(parse_content_format).transpose()?;
    let accept_cf = accept.iter().map(|a| parse_content_format(a)).collect::<Result<Vec<_>>>()?;
    let request = build_coap_request_for_url(&args.url, RequestType::Put, Some(data.as_bytes().to_vec()), cf, Some(accept_cf))?;
    let response = client.send(request).await?;

    let content = String::from_utf8_lossy(&response.message.payload);
    eprintln!("{}", response.message.header.get_code());
    println!("{}", content);

    Ok(())
}

async fn coap_delete(client: &mut UdpCoAPClient, args: &Args, accept: &[String]) -> Result<()> {
    eprintln!("DELETE {}", args.url);

    let accept_cf = accept.iter().map(|a| parse_content_format(a)).collect::<Result<Vec<_>>>()?;
    let request = build_coap_request_for_url(&args.url, RequestType::Delete, None, None, Some(accept_cf))?;
    let response = client.send(request).await?;

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

async fn create_coap_client(args: &Args) -> Result<UdpCoAPClient> {
    let (host, port, _, _) = parse_coap_url(&args.url)?;
    let mut client = UdpCoAPClient::new_udp((host, port.unwrap_or(5683))).await?;
    client.set_receive_timeout(Duration::new(args.timeout, 0));
    Ok(client)
}

async fn execute_command(args: &Args) -> Result<()> {
    let mut client = create_coap_client(args).await?;

    match &args.command {
        Commands::Get { accept } => coap_get(&mut client, args, accept).await,
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

            coap_post(&mut client, args, accept, content_format.as_deref(), &data).await
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

            coap_put(&mut client, args, accept, content_format.as_deref(), &data).await
        }
        Commands::Delete { accept } => coap_delete(&mut client, args, accept).await,
    }
}

#[tokio::main]
async fn main() {
    let cli = Args::parse();

    if let Err(err) = execute_command(&cli).await {
        eprintln!("ERROR: {}", err);
    }
}
