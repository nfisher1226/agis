## Description
Agis will be a [Spartan protocol](https://portal.mozz.us/spartan/spartan.mozz.us/)
server written in Rust. It is currently under active development but is not yet
functional.

## Planned Features
- [x] Multithreaded worker pool
- [x] Static files
- [x] Virtual hosts (name based)
- [x] CGI
- [x] Redirects
- [x] Aliases
- [x] indexes

## Configuration
The configuration file is in [Ron](https://github.com/ron-rs/ron) format, which
should be very simple to grasp if you are used to any programming languages with
braces (such as C). There is an example config file with plenty of comments in
`conf/config.ron`. This file can be copied to `/etc/agis/config.ron` and edited
to match your actual desired configuration.
