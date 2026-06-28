#!/bin/bash
# Ermete Kernel Chimera Builder
# Integrazione:
# 1. CachyOS (Scheduler BORE, Network optimization)
# 2. ClearLinux (Power management, AVX512/x86-64-v3 optimization)
# 3. Gentoo (-O3, ThinLTO, Clang compiler flags)

set -e

mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

echo ">>> Preparing Chimera Kernel Bedrock..."

# 1. Recupero base CachyOS
echo ">>> Fetching CachyOS kernel tree from COPR SRPM..."
curl -Lo /etc/yum.repos.d/bieszczaders-kernel-cachyos.repo "https://copr.fedorainfracloud.org/coprs/bieszczaders/kernel-cachyos/repo/fedora-${FEDORA_VERSION:-43}/bieszczaders-kernel-cachyos-fedora-${FEDORA_VERSION:-43}.repo"

# DNF setup and download
dnf install -y dnf-plugins-core
dnf download --source kernel-cachyos

echo ">>> Populating rpmbuild directories..."
rpm -ivh kernel-cachyos*.src.rpm
rm -f kernel-cachyos*.src.rpm

# Assicura che ci sia sempre un kernel-cachyos.spec per il workflow
SPEC_FILE=$(ls ~/rpmbuild/SPECS/*.spec | head -n 1)
if [ "$(basename "$SPEC_FILE")" != "kernel-cachyos.spec" ]; then
    mv "$SPEC_FILE" ~/rpmbuild/SPECS/kernel-cachyos.spec
fi

# 2. Iniezione patch Clear Linux (Es. Ottimizzazioni schedulatore e memoria)
echo ">>> Injecting Clear Linux patches..."
curl -sL https://raw.githubusercontent.com/clearlinux-pkgs/linux/master/0001-sched-migrate.patch -o ~/rpmbuild/SOURCES/0001-sched-migrate.patch
curl -sL https://raw.githubusercontent.com/clearlinux-pkgs/linux/master/0001-sched-numa-Initialise-numa_migrate_retry.patch -o ~/rpmbuild/SOURCES/0001-sched-numa-Initialise-numa_migrate_retry.patch

# Modifica il file spec per includere le patch
sed -i '/^Patch[0-9]*:.*/a Patch10001: 0001-sched-migrate.patch\nPatch10002: 0001-sched-numa-Initialise-numa_migrate_retry.patch' ~/rpmbuild/SPECS/kernel-cachyos.spec

# 3. Configurazione Compilatore (Gentoo Style)
echo ">>> Setting up Gentoo-style Clang/LTO parameters in the spec file..."
sed -i 's/%define buildid .*/%define buildid .chimera/' ~/rpmbuild/SPECS/kernel-cachyos.spec

# Append Clang and LTO flags for Extreme Integration
cat << 'EOF' >> ~/rpmbuild/SPECS/kernel-cachyos.spec
%global toolchain clang
%global _lto_cflags -flto=thin
%global optflags %{optflags} -O3 -march=x86-64-v3
EOF

echo ">>> Chimera Kernel preparation complete."
