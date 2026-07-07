%global debug_package %{nil}
Name:           ermete-ags-config
Version:        1.0.1
Release:        1%{?dist}
Summary:        Ermete OS ermete-ags-config
License:        MIT
URL:            https://github.com/patapem/ermete-forge
BuildArch:      noarch
Requires:       aylurs-gtk-shell2

%description
Provides default Astal (AGS v3) GTK4 desktop configuration for Ermete OS.

%prep
# Nothing to prep

%build
# Nothing to build

%install
mkdir -p %{buildroot}/usr/share/ermete-ags-config
mkdir -p %{buildroot}/etc/skel/.config/ags
cp -a %{_sourcedir}/etc/skel/.config/ags/* %{buildroot}/etc/skel/.config/ags/ || true

%files
%dir /usr/share/ermete-ags-config
/etc/skel/.config/ags/app.ts
/etc/skel/.config/ags/style.css

%changelog
* Tue Jul 07 2026 Ermete Forge <forge@ermete.os> - 1.0.1-1
- Transition default config from legacy AGS v1 to Astal v3 GTK4 (app.ts)
- Implement dynamic Catppuccin Mocha glassmorphism UI
