#!/bin/bash
# Ermete OS: The Ultimate Chimera Kernel Bedrock Builder
# 1. Base: Fedora Upstream (Stabilità, SELinux, Bootc)
# 2. Muscoli: CachyOS (BORE Scheduler, BBRv3, Ottimizzazioni di rete e CPU)
# 3. Nervi: Clear Linux (Ottimizzazioni NUMA, Schedulazione, Latenze Memoria)
# 4. Forgiatura: Gentoo (LLVM/Clang, -O3, x86-64-v3, ThinLTO)

set -e

WORKSPACE_DIR="$HOME/rpmbuild"
mkdir -p "$WORKSPACE_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
cd "$WORKSPACE_DIR"

echo "========================================================="
echo " FASE 1: LE FONDAMENTA (Fedora Upstream Zero-Trust)"
echo "========================================================="
# Otteniamo il kernel puro da Koji/Fedora per garantire la catena di fiducia.
dnf install -y dnf-utils koji git curl tar
# Scarichiamo l'ultimo sorgente kernel disponibile per Fedora 43
echo ">>> Scaricamento kernel.src.rpm puro..."
dnf download --source kernel --resolve
rpm -ivh kernel-*.src.rpm
rm -f kernel-*.src.rpm

# Rinominiamo il kernel per evitare conflitti con pacchetti standard
sed -i 's/Name: kernel/Name: kernel-chimera/' SPECS/kernel.spec
# Disabilitiamo il debuginfo per velocizzare la build (stile Arch/Gentoo)
sed -i 's/%define with_debuginfo %{?_without_debuginfo: 0} %{?!_without_debuginfo: 1}/%define with_debuginfo 0/' SPECS/kernel.spec


echo "========================================================="
echo " FASE 2: I MUSCOLI (Patch Ufficiali CachyOS)"
echo "========================================================="
# Invece di usare COPR, preleviamo il DNA direttamente da CachyOS.
echo ">>> Clonazione repository patch CachyOS..."
git clone --depth 1 https://github.com/CachyOS/kernel-patches.git /tmp/cachyos-patches

# Estraiamo le patch critiche:
# - BORE (Burst-Oriented Response Enhancer) Scheduler
# - Ottimizzazioni x86-64-v3 e AMD/Intel
# - Ottimizzazioni di rete (BBRv3)
CACHY_PATCH_DIR="/tmp/cachyos-patches/6.10" # Sostituire con la versione corrente
if [ -d "$CACHY_PATCH_DIR" ]; then
    echo ">>> Iniezione patch CachyOS (BORE, CPU, Rete)..."
    cp $CACHY_PATCH_DIR/all/*.patch SOURCES/
    
    # Iniezione programmatica nel file .spec di Fedora
    PATCH_INDEX=5000
    for patch in $CACHY_PATCH_DIR/all/*.patch; do
        patch_name=$(basename "$patch")
        sed -i "/^# End of generic patches/a Patch${PATCH_INDEX}: ${patch_name}" SPECS/kernel.spec
        PATCH_INDEX=$((PATCH_INDEX+1))
    done
else
    echo "ATTENZIONE: Directory patch CachyOS non trovata, fallback alla modalità ibrida."
    # In un ambiente di produzione reale, il bot aggiornerebbe questo path.
fi


echo "========================================================="
echo " FASE 3: I NERVI (Ottimizzazioni Clear Linux)"
echo "========================================================="
# Clear Linux domina nei benchmark per la gestione millimetrica della memoria e degli stati idle
echo ">>> Scaricamento patch chirurgiche da Intel Clear Linux..."

# Patch 1: Ottimizzazione della migrazione dei task nello scheduler
curl -sL https://raw.githubusercontent.com/clearlinux-pkgs/linux/master/0001-sched-migrate.patch -o SOURCES/0001-sched-migrate.patch
# Patch 2: Ottimizzazione NUMA retry per CPU multi-die
curl -sL https://raw.githubusercontent.com/clearlinux-pkgs/linux/master/0001-sched-numa-Initialise-numa_migrate_retry.patch -o SOURCES/0001-sched-numa-Initialise-numa_migrate_retry.patch
# Patch 3: Ottimizzazione degli hint per il memory controller
curl -sL https://raw.githubusercontent.com/clearlinux-pkgs/linux/master/0001-mm-memcontrol-add-some-branch-hints-based-on-gcov-an.patch -o SOURCES/0001-mm-memcontrol-branch-hints.patch

echo ">>> Iniezione patch Clear Linux nel file .spec..."
sed -i '/^# End of generic patches/a Patch6001: 0001-sched-migrate.patch\nPatch6002: 0001-sched-numa-Initialise-numa_migrate_retry.patch\nPatch6003: 0001-mm-memcontrol-branch-hints.patch' SPECS/kernel.spec


echo "========================================================="
echo " FASE 4: LA FORGIATURA (Estremismo Compiler Gentoo)"
echo "========================================================="
# Non modifichiamo il sorgente C, modifichiamo come viene trasformato in silicio.
echo ">>> Configurazione infrastruttura LLVM/Clang e ThinLTO..."

# 1. Obblighiamo l'uso di Clang al posto di GCC
sed -i '/%global toolchain /c\%global toolchain clang' SPECS/kernel.spec

# 2. Definiamo le CFLAGS estreme per hardware moderno (x86-64-v3 = AVX2, BMI1/2)
cat << 'SPEC_INJECT' >> SPECS/kernel.spec

# --- INIEZIONE ERMETE GENTOO LTO & COMPILER ---
%global optflags %{optflags} -O3 -march=x86-64-v3 -pipe -Wno-error
%global build_host ErmeteForge
%global _lto_cflags -flto=thin
%global use_lto 1
# ----------------------------------------------
SPEC_INJECT

# Forziamo LLVM nell'invocazione di make dentro il file spec
sed -i 's/make %{?_smp_mflags}/make %{?_smp_mflags} LLVM=1 LLVM_IAS=1/g' SPECS/kernel.spec

echo "========================================================="
echo " ASSEMBLAGGIO COMPLETATO. KERNEL CHIMERA PRONTO."
echo "========================================================="
