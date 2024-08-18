## Simple HTTP Server


### About
this is a small project that works much like pythons http server.
It can be used to serve files and also be used to upload files.

### Notice
This should not be used in a professional setting but for quick file sharing


### CLI
```bash

Usage: simplehttpserver [OPTIONS]

Options:
      --port <PORT>          change the port number. Default is 8888 [default: 8888]
  -a, --authenticate         Authenticate the user
  -p, --password <PASSWORD>  set password for authentication (required for authenticate flag) [default: ]
  -h, --help                 Print help
  -V, --version              Print version

```

### future
- [x] cli
- [ ] better error handling
- [ ] authentication
