Contents
========
[Unreleased](#unreleased)
[0.3.0](#0.3.0)
[0.2.0](#0.2.0)

# Unreleased
- Ability to override config file location on the command line
- Bind to optional second address to support ipv4 and ipv6 at the same time

# 0.3.0
- ScriptAlias support
- Add REMOTE_ADDR to CGI environment
- Add config option to support binding to second ip, for dual ipv4/ipv6
- Send a redirect if the client requests a directory without including a trailing
  slash, to prevent potential breakage for relative url's.

# 0.2.0
Initial release

