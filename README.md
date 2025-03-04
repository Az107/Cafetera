# CAFETERA ☕️  
[![Test](https://github.com/Az107/Cafetera/actions/workflows/test.yml/badge.svg)](https://github.com/Az107/Cafetera/actions/workflows/test.yml)

**Cafetera** (/kafeˈteɾa/) is a simple HTTP mock server made with [HTEAPOT](https://github.com/az107/hteapot), designed for mocking API endpoints for testing purposes. It allows you to define custom responses for different HTTP methods and routes through a TOML configuration file.

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

The server's behavior is defined by a TOML configuration file. Below is an example of the configuration file structure:

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

[[db]]
path = "/db/users"
data = '''
{
    "users": [
        {
            "name": "Jhon",
            "surname": "Smith",
            "age": 35
        }
    ],
    "last_id": "3bed620f-6020-444b-b825-d06240bfa632"
}
'''
```
## Usage

After starting the server, it will listen for HTTP requests on the specified port. The server matches incoming requests against the paths defined in the configuration file and responds with the corresponding status code and body.

### DB mode
The database consists of simple JSON structures that can be accessed and modified using GET, POST, PATCH, and DELETE requests

#### Read Data
```HTTP
GET /db/users/users/0 HTTP/1.1
```
This request retrieves the JSON object at the specified path
```JSON
{
  "age": 35,
  "name": "Jhon",
  "surname": "Smith"
}
```
You can also filter results using query parameters. For example:
for example
```HTTP
GET /db/users/users?name="Jhon" HTTP/1.1
```
This request returns all users matching the specified criteria.


#### Create

```HTTP
POST /db/users/users HTTP/1.1

{
  "age": 19,
  "name": "Sarah",
  "surname": "Brown"
}
```
This request adds a new entry to the users array.

Additionally, you can add new attributes to existing objects dynamically.

#### UPDATE

```HTTP
PATCH /db/users/users/1 HTTP/1.1

{"name":"Sara"}
```
This request updates the user at index 1, changing the name from "Sarah" to "Sara".

Using PATCH, only the specified attributes will be modified, while the rest of the object remains unchanged. If the provided attribute does not exist, it will not be added.

#### DELETE

```HTTP
DELETE /db/users/users/1 HTTP/1.1
```

This request removes the user at index 1 from the database.




Available wildcard variables:
- [x] {{path}}: The path of the request
- [ ] {{query}}: The query string of the request
- [x] {{rand}}: A random number between 0 and 100
- [x] {{arg.\<name\>}}: The value of the query parameter with the specified name
- [ ] {{header.\<name\>}}: The value of the header with the specified name
- [x] {{\<name\>}}: The value of the path parameter at the specified index

## Contributions

Contributions are welcome. Please feel free to submit pull requests or open issues to suggest improvements or add new features.
