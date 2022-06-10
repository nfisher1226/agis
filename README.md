Contents
========
- [Description](#description)
- [Features](#features)
- [Building](#building)
- [Configuration](#configuration)
- [Running](#running)
- [CGI](#cgi)
- [Script Alias](#script-alias)

## Description
Agis will be a [Spartan protocol](https://portal.mozz.us/spartan/spartan.mozz.us/)
server written in Rust. It is currently under active development but is not yet
functional.

## Building
Agis is written in Rust and requires the Cargo build tool.
```Sh
# Build a release binary
cargo build --release
```
## Features
- [x] Multithreaded worker pool
- [x] Static files
- [x] Virtual hosts (name based)
- [x] CGI
- [x] CGI ScriptAlias
- [x] Redirects
- [x] Aliases
- [x] indexes

## Configuration
The configuration file is in [Ron](https://github.com/ron-rs/ron) format, which
should be very simple to grasp if you are used to any programming languages with
braces (such as C). There is an example config file with plenty of comments in
`conf/config.ron`. This file can be copied to `/etc/agis/config.ron` and edited
to match your actual desired configuration.

### Fields (Global)
- address - The ip address to bind to. Change this to your server's public ip
  address.
- port - The port to listen on. Spartan specifies port 300, so only change this
  if you have a specific use case for it.
- user - The user which this server will run as. Agis must be started as root in
  order to bind to one of the lower ports, but will drop priviledges as soon as
  it is initialized.
- group - The group which the server will run as.
- threads - The number of threads to be started to handle requests. It is unlikely
  that you will have enough traffic to warrant increasing this.
- access_log - If this is set to `None`, access will be logged to stdout. If it
  is set to `Some(path)` access will be logged to that file.
- error_log - See access_log for specifics. Logs errors either to stderr or file.
- vhosts - One or more name based virtual hosts.

### Fields (per Vhost)
Each vhost is looked up by a key, which is the domain name it will serve.
- name - The domain name for which to serve requests.
- root - The path to the root directory of this server's files.
- directories - Path specific directives.

### Directives
Each directive is looked up via a key, which is the path which it applies to.
- Allow(bool) - whether or not to allow access to this path. If not set, all files
  in the document tree under the server root are allowed. If set to false, all
  files under this path are disallowed.
- Alias(path) - Serves files requested for this path from a different path. This is
  handled by the server transparently to the client.
- Redirect - Any request for this specific path will be sent a redirect to the
  new location, to be handled by the client.
- Cgi - Any requests under this directory will be passed to the cgi program which
  is the direct child of the directory. If the Cgi directive is given the path
  '/cgi-bin/', and a client requests '/cgi-bin/foo/bar/baz.gmi?fizzbuzz=true'
  then the program located at '/server-root/cgi-bin/foo' will be run and given
  the rest of the path and query as environment variables. This implementation
  is a subset of CGI 1.1 with http specific environment vars removed.
- ScriptAlias(path) - Any requests under this path will be interpreted via the
  CGI program specified by <path>. The <path> is given as an absolute path, with
  the path to the server root stripped from it. Thus, if the server root is
  `/srv/spartan` and the CGI program resides at `/srv/spartan/cgi-bin/hello`,
  then <path> would be given as `/cgi-bin/hello`.

The default configuration runs the server as user 'agis' and group 'agis'. You
will need to create that user and group on your system or Agis will not run.
```Sh
useradd -r -s /sbin/nologin agis
```
## Running
If you are running Linux with Systemd init, there is a unit file included in
the conf/ subdirectory. It can be copied into /etc/systemd/system and then
started and stopped like any other service.

If you are on a Linux system that does not use systemd, or bsd, it should be
straitforward to write your own init script. The default location for the
configuration file is `/etc/agis/config.ron` but can be overridden on the command
line with the `-c` or `--config` flag. This is currently the only command line
option which is supported, making startup quite straightforward.

## CGI
A CGI program can be written in any language and receives it's input via
environment variables. The program's output should present it's mime type in
plain text, followed by a carriage return and newline, and then any data which
is representable via a sequence of u8 bytes. This can be plain text but does not
have to be.
### CGI environment vars
| Var | Meaning |
| --- | --- |
| DOCUMENT_ROOT | The root directory of your server |
| QUERY_STRING | The query string |
| REMOTE_ADDR | The IP address of the client |
| REQUEST_URI | The interpreted pathname of the requested document or CGI (relative to the document root) |
| SCRIPT_FILENAME | The full pathname of the current CGI |
| SCRIPT_NAME | The interpreted pathname of the current CGI (relative to the document root) |
| SERVER_NAME | Your server's fully qualified domain name (e.g. www.cgi101.com) |
| SERVER_PORT | The port number your server is listening on |
| SERVER_SOFTWARE | The server software you're using |
| REQUEST_BODY | The path to a temporary file containing any content uploaded to the server |

## ScriptAlias
The ScriptAlias directive allows passing requests to a CGI program without the
cgi-bin directory or program name appearing in the url. In this way, dynamic
content can be served without revealing to the client that a CGI program is being
run or what the nature of that program is. This might be desireable if, for instance,
one is using php scripting and doesn't wish to make that readily known to potential
attackers.
