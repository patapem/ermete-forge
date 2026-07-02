Name:           ermete-ags-config
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS ermete-ags-config
License:        MIT
URL:            https://github.com/patapem/ermete-forge
BuildArch:      noarch

%description
Provides ermete-ags-config for Ermete OS.

%prep
# Nothing to prep

%build
# Nothing to build

%install
mkdir -p %{buildroot}/usr/share/ermete-ags-config
mkdir -p %{buildroot}/etc/skel/.config/ags
cp -a SOURCES/etc/skel/.config/ags/* %{buildroot}/etc/skel/.config/ags/ || true

%files
%dir /usr/share/ermete-ags-config
/etc/skel/.config/ags/config.js
/etc/skel/.config/ags/style.css

%changelog
* Wed Jul 01 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial Bedrock encapsulation
