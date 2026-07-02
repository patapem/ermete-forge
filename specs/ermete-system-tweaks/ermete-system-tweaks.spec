%global debug_package %{nil}
Name:           ermete-system-tweaks
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS ermete-system-tweaks
License:        MIT
URL:            https://github.com/patapem/ermete-forge
BuildArch:      noarch

%description
Provides ermete-system-tweaks for Ermete OS.

%prep
# Nothing to prep

%build
# Nothing to build

%install
mkdir -p %{buildroot}/usr/share/ermete-system-tweaks

%post

%files
%dir /usr/share/ermete-system-tweaks

%changelog
* Wed Jul 01 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial Bedrock encapsulation
