# coap-cli

This is a simple command-line tool to send CoAP requests via UDP and display the
responses.

## Usage

```
Usage: coap-cli <URL> <COMMAND>

Commands:
  get     Retrieves a representation of a resource
  post    Requests that the submitted data be processed
  put     Requests that the resource be updated or created with the submitted data
  delete  Requests that the resource be deleted
  help    Print this message or the help of the given subcommand(s)

Arguments:
  <URL>  COAP resource URL

Options:
  -h, --help  Print help
```

Some commands have their own options:

```
Usage: coap-cli.exe <URL> post [OPTIONS]

Options:
  -d, --data <DATA>  Resource data
  -f, --file <FILE>  Path to file containing resource data
  -h, --help         Print help
```

```
Usage: coap-cli.exe <URL> put [OPTIONS]

Options:
  -d, --data <DATA>  Resource data
  -f, --file <FILE>  Path to file containing resource data
  -h, --help         Print help
```

## Examples

```
$ coap-cli coap://10.1.2.3/version get
GET coap://10.1.2.3/version
2.05
{"version":"1.2.3.4"}
```

```
$ coap-cli coap://10.1.2.3/some/resource post -f path/to/data
POST coap://10.1.2.3/some/resource
2.04
```

```
$ coap-cli coap://10.1.2.3/some/resource put -d '{"name":"stuff"}'
PUT coap://10.1.2.3/some/resource
2.04
```
