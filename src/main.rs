use clap::{Parser, Subcommand};
use coap::CoAPClient;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Args {
    /// COAP resource URL
    url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Retrieves a representation of a resource
    Get,

    /// Requests that the submitted data be processed
    Post {
        /// Resource data
        #[arg(short, long)]
        data: Option<String>,
        /// Path to file containing resource data
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Requests that the resource be updated or created with the submitted data
    Put {
        /// Resource data
        #[arg(short, long)]
        data: Option<String>,
        /// Path to file containing resource data
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Requests that the resource be deleted
    Delete,
}

fn coap_get(url: &str) -> Result<()> {
    eprintln!("GET {}", url);

    let response = CoAPClient::get(&url)?;
    let content = String::from_utf8(response.message.payload).unwrap();
    eprintln!("{}", response.message.header.get_code());
    println!("{}", content);

    Ok(())
}

fn coap_post(url: &str, data: &str) -> Result<()> {
    eprintln!("POST {}", url);

    let response = CoAPClient::post(&url, data.as_bytes().to_vec())?;
    let content = String::from_utf8(response.message.payload).unwrap();
    eprintln!("{}", response.message.header.get_code());
    println!("{}", content);

    Ok(())
}

fn coap_put(url: &str, data: &str) -> Result<()> {
    eprintln!("PUT {}", url);

    let response = CoAPClient::put(&url, data.as_bytes().to_vec())?;
    let content = String::from_utf8(response.message.payload).unwrap();
    eprintln!("{}", response.message.header.get_code());
    println!("{}", content);

    Ok(())
}

fn coap_delete(url: &str) -> Result<()> {
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
    match &cli.command {
        Commands::Get => coap_get(&cli.url),
        Commands::Post { data, file } => {
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

            coap_post(&cli.url, &data)
        }
        Commands::Put { data, file } => {
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

            coap_put(&cli.url, &data)
        }
        Commands::Delete => coap_delete(&cli.url),
    }
}

fn main() {
    let cli = Args::parse();

    if let Err(err) = execute_command(&cli) {
        eprintln!("ERROR: {}", err);
    }
}
