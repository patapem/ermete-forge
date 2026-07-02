Name:           ermete-system-config
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS ermete-system-config
License:        MIT
URL:            https://github.com/patapem/ermete-forge
BuildArch:      noarch

%description
Provides ermete-system-config for Ermete OS.

%prep
# Nothing to prep

%build
# Nothing to build

%install
mkdir -p %{buildroot}/usr/share/ermete-system-config
mkdir -p %{buildroot}/usr/lib/systemd/system-preset
mkdir -p %{buildroot}/usr/lib/tmpfiles.d
mkdir -p %{buildroot}/etc/greetd
cp -a SOURCES/usr/lib/systemd/system-preset/* %{buildroot}/usr/lib/systemd/system-preset/ || true
cp -a SOURCES/usr/lib/tmpfiles.d/* %{buildroot}/usr/lib/tmpfiles.d/ || true
cp -a SOURCES/etc/greetd/config.toml %{buildroot}/etc/greetd/config.toml || true

%post

%files
%dir /usr/share/ermete-system-config
/usr/lib/systemd/system-preset/99-Ermete.preset
/usr/lib/tmpfiles.d/10-ermete-home.conf
/usr/lib/tmpfiles.d/10-ermete-greetd.conf
%config(noreplace) /etc/greetd/config.toml

%changelog
* Wed Jul 01 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial Bedrock encapsulation
