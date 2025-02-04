# bak01

## Summary

The git repository to work on the university module BAK01. The repository contains a sample
server application to be load balanced, that can be used as HTTP, TCP or UDP server. The
second application is the actual load balancer.

## Local development

### Starting sample backend servers

First you should start as many instances of the sample server as you like:

```shell
cargo watch -w example-server -x "run --bin example-server -- --server-type http --port 8080"
```

NOTE: You can choose a different server type, but make sure all instances use the same type.
Furthermore you should provide a different port to each instance.

### Starting the load balancer

After you started the backends, you should start the load balancer:

```shell
cargo watch -w load-balancer -x "run --bin load-balancer -- --port 3000 --proxy-type tcp --servers 127.0.0.1:8080"
```

NOTE: Depending on the number of started backends, the `--servers` argument will be different.
