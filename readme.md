## Simple HTTP Server


### About
this is a small project that works much like pythons http server.
It can be used to serve files and also be used to upload files.

### Notice
This should not be used in a professional setting but for quick file sharing


### SSL Usage
To use SSL you need to have openssl installed on your system.
run the following command to generate the required files
```bash
openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'
```

### CLI
```bash

Simple HTTP Server refer to https://github.com/jackrizza/http-server for more information

Usage: main [OPTIONS]

Options:
      --port <PORT>            change the port number. Default is 8888 [default: 8888]
  -a, --authenticate           Authenticate the user openssl needed for authentication (Cookies won\'t work without it)
  -p, --password <PASSWORD>    set password for authentication (required for authenticate flag) [default: ]
      --pem-file <PEM_FILE>    [default: key.pem]
      --cert-file <CERT_FILE>  [default: cert.pem]
  -h, --help                   Print help
  -V, --version                Print version


```

### future
- [x] cli
- [ ] better error handling
- [x] authentication
