%global debug_package %{nil}
Name:           ermete-niri-session
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS ermete-niri-session
License:        MIT
URL:            https://github.com/patapem/ermete-forge
BuildArch:      noarch

%description
Provides ermete-niri-session for Ermete OS.

%prep
# Nothing to prep

%build
# Nothing to build

%install
mkdir -p %{buildroot}/usr/share/ermete-niri-session
mkdir -p %{buildroot}/etc/skel/.config/niri
cp -a %{_sourcedir}/etc/skel/.config/niri/config.kdl %{buildroot}/etc/skel/.config/niri/config.kdl || true

%files
%dir /usr/share/ermete-niri-session
/etc/skel/.config/niri/config.kdl

%changelog
* Wed Jul 01 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial Bedrock encapsulation
