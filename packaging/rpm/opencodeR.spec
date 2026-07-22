Name: opencodeR
Version: 0.1.0
Release: 1%{?dist}
Summary: OpenCode AI coding agent (Rust port)

License: MIT
URL: https://github.com/opencode-r/opencodeR
Source0: opencodeR-x86_64-unknown-linux-gnu.tar.gz
BuildArch: x86_64

%description
AI-powered development tool with HTTP API server
and CLI client for interacting with AI coding agents.
Includes opencodeR (combined binary), opencodeR-server,
and opencodeR-client.

%prep
%setup -q -n opencodeR-x86_64-unknown-linux-gnu

%install
mkdir -p %{buildroot}/usr/bin
install -m 755 opencodeR-amd64 %{buildroot}/usr/bin/opencodeR
install -m 755 opencodeR-server-amd64 %{buildroot}/usr/bin/opencodeR-server
install -m 755 opencodeR-client-amd64 %{buildroot}/usr/bin/opencodeR-client

%files
/usr/bin/opencodeR
/usr/bin/opencodeR-server
/usr/bin/opencodeR-client

%changelog
* $(date '+%a %b %d %Y') opencodeR <dev@opencode.ai>
- Initial release
