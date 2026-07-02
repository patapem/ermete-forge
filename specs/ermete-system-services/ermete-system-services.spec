Name:           ermete-system-services
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS ermete-system-services
License:        MIT
URL:            https://github.com/patapem/ermete-forge
BuildArch:      noarch

%description
Provides ermete-system-services for Ermete OS.

%prep
# Nothing to prep

%build
# Nothing to build

%install
mkdir -p %{buildroot}/usr/share/ermete-system-services

%post

%files
%dir /usr/share/ermete-system-services

%changelog
* Wed Jul 01 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial Bedrock encapsulation
