use clap::{Parser, Subcommand};
use coap::CoAPClient;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    port: Option<u16>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// The GET method retrieves a representation for the information that currently corresponds to the resource identified by the request URI.
    Get { host: String, path: String },
    /// The POST method requests that the representation enclosed in the request be processed. Either a data string or file path must be provided.
    Post {
        /// COAP server hostname
        host: String,
        /// COAP resource path
        path: String,
        /// Resource data
        #[arg(short, long)]
        data: Option<String>,
        /// Path to file containing resource data
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// The PUT method requests that the resource identified by the request URI be updated or created with the enclosed representation.
    Put {
        /// COAP server hostname
        host: String,
        /// COAP resource path
        path: String,
        /// Resource data
        #[arg(short, long)]
        data: Option<String>,
        /// Path to file containing resource data
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// The DELETE method requests that the resource identified by the request URI be deleted.
    Delete { host: String, path: String },
}

fn coap_get(host: &str, path: &str, port: u16) -> Result<()> {
    let url = format!("coap://{}:{}/{}", host, port, path);
    eprintln!("GET {}", url);

    let response = CoAPClient::get(&url)?;
    let content = String::from_utf8(response.message.payload).unwrap();
    eprintln!("{}", response.message.header.get_code());
    println!("{}", content);

    Ok(())
}

fn coap_post(host: &str, path: &str, port: u16, data: &str) -> Result<()> {
    let url = format!("coap://{}:{}/{}", host, port, path);
    eprintln!("POST {}", url);

    let response = CoAPClient::post(&url, data.as_bytes().to_vec())?;
    let content = String::from_utf8(response.message.payload).unwrap();
    eprintln!("{}", response.message.header.get_code());
    println!("{}", content);

    Ok(())
}

fn coap_put(host: &str, path: &str, port: u16, data: &str) -> Result<()> {
    let url = format!("coap://{}:{}/{}", host, port, path);
    eprintln!("PUT {}", url);

    let response = CoAPClient::put(&url, data.as_bytes().to_vec())?;
    let content = String::from_utf8(response.message.payload).unwrap();
    eprintln!("{}", response.message.header.get_code());
    println!("{}", content);

    Ok(())
}

fn coap_delete(host: &str, path: &str, port: u16) -> Result<()> {
    let url = format!("coap://{}:{}/{}", host, port, path);
    eprintln!("DELETE {}", url);

    let response = CoAPClient::delete(&url)?;
    let content = String::from_utf8(response.message.payload).unwrap();
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

fn execute_command(cli: &Args) -> Result<()> {
    let port = cli.port.unwrap_or(5683);

    match &cli.command {
        Commands::Get { host, path } => coap_get(host, path, port),
        Commands::Post {
            host,
            path,
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

            coap_post(host, path, port, &data)
        }
        Commands::Put {
            host,
            path,
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

            coap_put(host, path, port, &data)
        }
        Commands::Delete { host, path } => coap_delete(host, path, port),
    }
}

fn main() {
    let cli = Args::parse();

    if let Err(err) = execute_command(&cli) {
        eprintln!("ERROR: {}", err);
    }
}
