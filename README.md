# CAFETERA
[![Test](https://github.com/Az107/Cafetera/actions/workflows/test.yml/badge.svg)](https://github.com/Az107/Cafetera/actions/workflows/test.yml)

## Description

This is a simple HTTP mock server, designed for mocking API endpoints for testing purposes. It allows you to define custom responses for different HTTP methods and routes through a TOML configuration file.

Requirements
- Rust

## Setup

To get the server running, follow these steps:

Clone the repository to your local machine.
Ensure you have Rust installed. If not, install Rust using rustup.
Navigate to the root directory of the project.
Build the project using Cargo:
```shell
cargo build
```

Run the server with the following command, replacing <port> with your desired port number and <config_path> with the path to your configuration JSON file:
```shell
cargo run <port> <config_path>
```
OR 

```shell
CAFETERA <port> <config_path>
```

## Configuration

The server's behavior is defined by a JSON configuration file. Below is an example of the configuration file structure:

```toml
[[endpoints.GET]]
path = "/health"
status = 200
body = "API is up and running"

[[endpoints.GET]]
path = "/users"
status = 200
body = '''
[
  {
    "id": "{{rand}}", <-- this will be replaced with a random number
    "name": "{{arg.name}}",
    "email": "",
    "path": "{{path}}"
  }
]
'''

[[endpoints.GET]]
path = "/users/{{name}}"
status = 200
body = '''
{
  "id": 1,
  "name": "{{name}}",
  "email": ""
}
'''

[[endpoints.GET]]
path = "/users/{{name}}/{{id}}"
status = 200
body = '''
{
  "id": "{{id}}",
  "name": "{{name}}",
  "email": ""
}
'''

[[endpoints.POST]]
path = "/users"
status = 201
body = '''
{
  "id": "{{rand}}",
  "name": "Jane Doe",
  "email": ""
}
'''
```
## Usage

After starting the server, it will listen for HTTP requests on the specified port. The server matches incoming requests against the paths defined in the configuration file and responds with the corresponding status code and body.

Available wildcard variables:
- [x] {{path}}: The path of the request
- [ ] {{query}}: The query string of the request
- [x] {{rand}}: A random number between 0 and 100
- [x] {{arg.\<name\>}}: The value of the query parameter with the specified name
- [ ] {{header.\<name\>}}: The value of the header with the specified name
- [x] {{\<name\>}}: The value of the path parameter at the specified index

## Contributions

Contributions are welcome. Please feel free to submit pull requests or open issues to suggest improvements or add new features.
