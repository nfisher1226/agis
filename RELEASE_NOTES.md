Contents
========
[0.5.0](#0.5.0)
[0.4.0](#0.4.0)
[0.3.0](#0.3.0)
[0.2.0](#0.2.0)

# 0.5.0
- Only allow worker threads to panick during shutdown, otherwise log any message
passing errors in the error log
- Don't use `unwrap()` on incoming streams. Instead, if the incoming stream is in
an error condition, log it in the error log
- Add `version` command line option
- Print any errors in parsing cli options to stderr and print `usage`

# 0.4.0
- Ability to override config file location on the command line
- Bind to optional second address to support ipv4 and ipv6 at the same time
- Handle percent encoded url's properly
- Fall back to stdout/stderr if logs cannot be opened for writing
- Make logging less verbose
- Share worker pool among both tcp listeners if bound to a second address
- Shut down worker pool cleanly rather than just exiting when termination signal
  is received
- Log startup and shutdown messages rather than printing to stdout

# 0.3.0
- ScriptAlias support
- Add REMOTE_ADDR to CGI environment
- Add config option to support binding to second ip, for dual ipv4/ipv6
- Send a redirect if the client requests a directory without including a trailing
  slash, to prevent potential breakage for relative url's.

# 0.2.0
Initial release

