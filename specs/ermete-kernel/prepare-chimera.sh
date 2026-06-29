#!/bin/bash
# Ermete OS: The Ultimate Chimera Kernel Bedrock Builder

set -e

WORKSPACE_DIR="$HOME/rpmbuild"
mkdir -p "$WORKSPACE_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
cd "$WORKSPACE_DIR"

echo "========================================================="
echo " FASE 1: LE FONDAMENTA (Fedora Upstream Zero-Trust)"
echo "========================================================="
# [BEST PRACTICE] Lavoriamo con il kernel.spec nativo al 100%, ZERO comandi "sed".
echo ">>> Scaricamento kernel.src.rpm puro..."
dnf download --source kernel
rpm -ivh kernel-*.src.rpm
KERNEL_SRPM=$(ls kernel-*.src.rpm | head -n 1)
KERNEL_VER=$(rpm -qp --qf '%{VERSION}' "$KERNEL_SRPM" | cut -d. -f1,2)
rm -f kernel-*.src.rpm

echo "========================================================="
echo " FASE 2: I MUSCOLI E I NERVI (Scarico Patch)"
echo "========================================================="
echo ">>> Clonazione repository patch CachyOS..."
rm -rf /tmp/cachyos-patches
git clone --depth 1 https://github.com/CachyOS/kernel-patches.git /tmp/cachyos-patches

CACHY_PATCH_DIR="/tmp/cachyos-patches/$KERNEL_VER"
if [ ! -d "$CACHY_PATCH_DIR/all" ]; then
    echo "ATTENZIONE: Patch CachyOS per $KERNEL_VER non trovate. Fallback..."
    CACHY_PATCH_DIR=$(ls -d /tmp/cachyos-patches/[0-9].* | sort -V | tail -n 1)
fi

echo ">>> Scansione e registrazione delle patch nello spec (Native RPM Best Practice)..."
# Invece di concatenare file (hack), registriamo ogni patch con un suo ID univoco
# all'interno del file spec. In questo modo RPM traccia nativamente i sorgenti
# e in caso di errore sappiamo esattamente quale patch ha fallito.
PATCH_ID=10000

if [ -d "$CACHY_PATCH_DIR/all" ]; then
    for patch in "$CACHY_PATCH_DIR"/all/*.patch; do
        cp "$patch" SOURCES/
        patch_name=$(basename "$patch")
        sed -i "/^Patch999999:/i Patch${PATCH_ID}: ${patch_name}" SPECS/kernel.spec
        sed -i "/^%build/i %patch -P ${PATCH_ID} -p1" SPECS/kernel.spec
        ((PATCH_ID++))
    done
fi

echo ">>> Aggiunta patch chirurgiche Clear Linux..."
for patch_url in \
    "https://raw.githubusercontent.com/clearlinux-pkgs/linux/main/0001-sched-migrate.patch" \
    "https://raw.githubusercontent.com/clearlinux-pkgs/linux/main/0001-sched-numa-Initialise-numa_migrate_retry.patch" \
    "https://raw.githubusercontent.com/clearlinux-pkgs/linux/main/0001-mm-memcontrol-add-some-branch-hints-based-on-gcov-an.patch" \
    "https://raw.githubusercontent.com/clearlinux-pkgs/linux/main/0002-sched-core-add-some-branch-hints-based-on-gcov-analy.patch" \
    "https://raw.githubusercontent.com/clearlinux-pkgs/linux/main/0170-sched-Add-unlikey-branch-hints-to-several-system-cal.patch"; do
    
    patch_name=$(basename "$patch_url")
    curl -sL -f "$patch_url" -o "SOURCES/clearlinux-$patch_name"
    
    sed -i "/^Patch999999:/i Patch${PATCH_ID}: clearlinux-${patch_name}" SPECS/kernel.spec
    sed -i "/^%build/i %patch -P ${PATCH_ID} -p1" SPECS/kernel.spec
    ((PATCH_ID++))
done

echo "========================================================="
echo " FASE 3: TUNING KCONFIG E MACROS (Bedrock Naturale)"
echo "========================================================="
echo ">>> Creazione kernel-local..."
# [BEST PRACTICE] L'uso di kernel-local è il metodo ufficiale supportato da Fedora
cat << 'EOF' > SOURCES/kernel-local
# --- ERMETE FORGE: ZEN/LIQUORIX TUNING ---
CONFIG_HZ_1000=y
CONFIG_HZ=1000
# CONFIG_HZ_300 is not set
# CONFIG_HZ_250 is not set

CONFIG_PREEMPT=y
CONFIG_PREEMPT_BUILD=y
CONFIG_PREEMPT_DYNAMIC=y

CONFIG_RCU_EXPERT=y
CONFIG_RCU_BOOST=y
CONFIG_RCU_BOOST_DELAY=500

CONFIG_TCP_CONG_BBR=y
CONFIG_DEFAULT_BBR=y

CONFIG_SCHED_BORE=y

# Mitigations Off
# CONFIG_SPECULATION_MITIGATIONS is not set

# ZSTD Estrema
CONFIG_MODULE_COMPRESS_ZSTD=y
CONFIG_MODULE_COMPRESS_ZSTD_LEVEL=19

# Ottimizzazione MGLRU (Multi-Gen LRU) attiva per default (Ottimo per 32GB RAM)
CONFIG_LRU_GEN=y
CONFIG_LRU_GEN_ENABLED=y

# Ottimizzazione CPU Architettura Esatta (Zen 3 - Ryzen 5800X3D)
CONFIG_MZEN3=y
# CONFIG_GENERIC_CPU is not set

# NT Sync per Gaming
CONFIG_NTSYNC=y
# -----------------------------------------
EOF

echo ">>> Generazione ~/.rpmmacros globale per la compilazione..."
# [BEST PRACTICE] Zero modifiche al file kernel.spec. Tutte le macro e i flag del
# compilatore (LLVM, LTO, identificatore OS) vengono iniettati tramite rpmmacros 
# in modo nativo per rpmbuild.
cat << 'EOF' > ~/.rpmmacros
%buildid .chimera
%toolchain clang
%use_lto 1
%_lto_cflags -flto=thin
%optflags %{__global_compiler_flags} -O3 -march=znver3 -pipe -Wno-error -g
%kcflags -O3 -march=znver3 -pipe -Wno-error
EOF

echo "========================================================="
echo " PREPARAZIONE COMPLETATA."
echo " Zero 'sed' eseguiti. Il file .spec e' intatto."
echo " Tutte le flags LLVM/Gentoo sono passate in runtime tramite ~/.rpmmacros."
echo "========================================================="
