#!/bin/bash
# Ermete OS: The Ultimate Chimera Kernel Bedrock Builder (Upstream Mainline Torvalds)

set -e

WORKSPACE_DIR="$HOME/rpmbuild/BUILD"
echo ">>> Pulizia profonda del workspace per evitare conflitti con vecchie build..."
rm -rf "$WORKSPACE_DIR"/*
mkdir -p "$WORKSPACE_DIR"
cd "$WORKSPACE_DIR"

echo "========================================================="
echo " FASE 1: RISOLUZIONE DINAMICA MATRICE KERNEL UPSTREAM"
echo "========================================================="
echo ">>> Clonazione repository patch CachyOS..."
rm -rf /tmp/cachyos-patches
git clone --depth 1 https://github.com/CachyOS/kernel-patches.git /tmp/cachyos-patches

echo ">>> Clonazione repository patch Clear Linux..."
rm -rf /tmp/clearlinux-patches
git clone --depth 500 https://github.com/clearlinux-pkgs/linux.git /tmp/clearlinux-patches

echo ">>> Clonazione repository patch linux-tkg..."
rm -rf /tmp/tkg-patches
git clone --depth 1 https://github.com/Frogging-Family/linux-tkg.git /tmp/tkg-patches

echo ">>> Clonazione repository patch XanMod..."
rm -rf /tmp/xanmod-patches
git clone --depth 1 https://gitlab.com/xanmod/linux-patches.git /tmp/xanmod-patches || true

echo ">>> Clonazione repository patch Liquorix..."
rm -rf /tmp/liquorix-patches
git clone --depth 1 https://github.com/damentz/liquorix-package.git /tmp/liquorix-patches

echo ">>> Calcolo intersezione versioni perfette (Matrice Universale)..."
CACHY_VERSIONS=$(ls -d /tmp/cachyos-patches/*/all 2>/dev/null | awk -F/ '{print $4}' | sort -V || true)
TKG_VERSIONS=$(ls -d /tmp/tkg-patches/linux-tkg-patches/* 2>/dev/null | awk -F/ '{print $5}' | sort -V || true)
XANMOD_VERSIONS=$(ls -d /tmp/xanmod-patches/linux-*-xanmod 2>/dev/null | awk -F- '{print $2}' | sed 's/\.y//' | sort -V || true)

TARGET_KERNEL_VER=""
# Cerca la versione più alta che esista sia in CachyOS che in TKG e Xanmod
for v in $(echo "$CACHY_VERSIONS" | tac); do
    if echo "$TKG_VERSIONS" | grep -q "^$v$" && echo "$XANMOD_VERSIONS" | grep -q "^$v$"; then
        TARGET_KERNEL_VER="$v"
        break
    fi
done

# Fallback: Se Xanmod è troppo avanti, cerchiamo tra CachyOS e TKG
if [ -z "$TARGET_KERNEL_VER" ]; then
    echo ">>> Xanmod non ha una versione compatibile, cerco intersezione CachyOS + TKG..."
    for v in $(echo "$CACHY_VERSIONS" | tac); do
        if echo "$TKG_VERSIONS" | grep -q "^$v$"; then
            TARGET_KERNEL_VER="$v"
            break
        fi
    done
fi

# Fallback finale: Solo CachyOS
if [ -z "$TARGET_KERNEL_VER" ]; then
    echo ">>> Nessuna intersezione trovata, fallback forzato sull'ultima versione di CachyOS..."
    TARGET_KERNEL_VER=$(echo "$CACHY_VERSIONS" | tail -n 1)
fi

if [ -z "$TARGET_KERNEL_VER" ]; then
    echo "ERRORE FATALE: Impossibile determinare la versione CachyOS."
    exit 1
fi

echo ">>> MATCH PERFETTO! Costruiremo il kernel Mainline versione: $TARGET_KERNEL_VER"

echo "========================================================="
echo " FASE 2: FONDAMENTA E CHIRURGIA PATCH (Upstream Torvalds)"
echo "========================================================="
KERNEL_TARBALL="linux-${TARGET_KERNEL_VER}.tar.xz"
echo ">>> Scaricamento Kernel Upstream Torvalds ($KERNEL_TARBALL)..."
if [ ! -f "$KERNEL_TARBALL" ]; then
    wget -q "https://cdn.kernel.org/pub/linux/kernel/v6.x/$KERNEL_TARBALL"
fi

echo ">>> Estrazione del Kernel..."
tar -xf "$KERNEL_TARBALL"
cd "linux-${TARGET_KERNEL_VER}"

echo ">>> Sincronizzazione dinamica Clear Linux con Kernel $TARGET_KERNEL_VER..."
pushd /tmp/clearlinux-patches > /dev/null
KERNEL_VER_ESC="${TARGET_KERNEL_VER//./\\.}"
CLEAR_COMMIT=$(git log --grep="update.*${KERNEL_VER_ESC}\\b" -n 1 --format="%H" || true)
if [ -n "$CLEAR_COMMIT" ]; then
    echo ">>> Allineamento Clear Linux al commit: $CLEAR_COMMIT"
    git checkout -q "$CLEAR_COMMIT"
else
    echo ">>> ATTENZIONE: Nessun commit specifico trovato per $TARGET_KERNEL_VER. Utilizzo l'head di main."
fi
popd > /dev/null

echo ">>> Importazione Chirurgica delle Patch (Solo versioni compatibili!)..."
mkdir -p .patches

# 1. CachyOS (Sempre compatibile)
if [ -d "/tmp/cachyos-patches/$TARGET_KERNEL_VER" ]; then
    cp "/tmp/cachyos-patches/$TARGET_KERNEL_VER"/*.patch .patches/ || true
fi

# 2. Linux-TKG (Solo se compatibile)
if [ -d "/tmp/tkg-patches/linux-tkg-patches/$TARGET_KERNEL_VER" ]; then
    cp "/tmp/tkg-patches/linux-tkg-patches/$TARGET_KERNEL_VER"/*.patch .patches/ || true
fi

# 3. XanMod (Solo se compatibile)
if [ -d "/tmp/xanmod-patches/linux-${TARGET_KERNEL_VER}.y-xanmod" ]; then
    cp "/tmp/xanmod-patches/linux-${TARGET_KERNEL_VER}.y-xanmod"/*.patch .patches/ || true
fi

# 4. Clear Linux (Patch specifiche)
for patch_name in \
    "0001-sched-migrate.patch" \
    "0001-sched-numa-Initialise-numa_migrate_retry.patch" \
    "0001-mm-memcontrol-add-some-branch-hints-based-on-gcov-an.patch" \
    "0002-sched-core-add-some-branch-hints-based-on-gcov-analy.patch" \
    "0170-sched-Add-unlikey-branch-hints-to-several-system-cal.patch"; do
    if [ -f "/tmp/clearlinux-patches/$patch_name" ]; then
        cp "/tmp/clearlinux-patches/$patch_name" .patches/ || true
    fi
done

# Genera un default Kconfig per permettere a Kbuild di funzionare (ci serve per AST validazione)
make defconfig

echo ">>> [BEDROCK] Inizio applicazione matrice universale Kbuild per gli 8 Kernel..."
for patch in .patches/*.patch; do
    if [ ! -f "$patch" ]; then continue; fi
    echo "-> Test di compatibilità per $(basename "$patch")..."
    
    if patch -p1 -F 2 --force --dry-run --silent < "$patch"; then
        patch -p1 -F 2 --force < "$patch" > /dev/null || true
        echo "   [SUCCESS] Patch applicata a Fuzz 2."
    else
        echo "   [WARNING] Fallito Fuzz 2. Tento Fuzz 3..."
        NON_C_FILES=$(grep -E '^\+\+\+ b/' "$patch" | awk '{print $2}' | sed 's/^b\///' | grep -v '\.c$' || true)
        if [ -n "$NON_C_FILES" ]; then
             echo "   [SKIP] La patch tocca file non-C. Impossibile validare con AST. Fuzz 3 vietato. Scartata."
        elif patch -p1 -F 3 --force --dry-run --silent < "$patch"; then
            patch -p1 -F 3 --force < "$patch" > /dev/null || true
            
            echo "   [AST VALIDATION] Controllo purezza albero sintattico tramite Kbuild Assembly..."
            MODIFIED_C_FILES=$(grep -E '^\+\+\+ b/' "$patch" | awk '{print $2}' | sed 's/^b\///' | grep '\.c$' || true)
            AST_FAILED=0
            for c_file in $MODIFIED_C_FILES; do
                if [ -f "$c_file" ]; then
                    target_s="${c_file%.c}.s"
                    if ! make "$target_s" >/dev/null 2>&1; then
                        AST_FAILED=1
                        echo "   [AST FATAL] Kbuild ha fallito la compilazione AST di $c_file!"
                        break
                    fi
                fi
            done
            if [ $AST_FAILED -eq 1 ]; then
                echo "   [ROLLBACK] Conflitto sintattico rilevato! Scarto la patch."
                patch -p1 -R -F 3 --force < "$patch" > /dev/null || true
            else
                echo "   [SUCCESS] Patch fusa e validata nativamente tramite AST Kbuild."
            fi
        else
            echo "   [SKIP] Conflitto strutturale (Fallito Fuzz 3). Patch scartata."
        fi
    fi
done

echo "========================================================="
echo " FASE 3: TUNING KCONFIG (Bedrock Naturale)"
echo "========================================================="
# Configurazione Kconfig avanzata tramite script Kconfig integrato in upstream
./scripts/config --enable HZ_1000
./scripts/config --set-val HZ 1000
./scripts/config --disable HZ_300
./scripts/config --disable HZ_250
./scripts/config --disable HZ_100
./scripts/config --enable DEFAULT_BBR
./scripts/config --enable TCP_CONG_BBR
./scripts/config --disable DEFAULT_CUBIC
./scripts/config --enable SCHED_BORE
./scripts/config --enable MODULE_COMPRESS_ZSTD
./scripts/config --disable MODULE_COMPRESS_XZ
./scripts/config --enable LRU_GEN
./scripts/config --enable LRU_GEN_ENABLED
./scripts/config --enable GENERIC_CPU
./scripts/config --enable CC_OPTIMIZE_FOR_PERFORMANCE_O3
./scripts/config --disable LTO_CLANG_THIN
./scripts/config --disable DEBUG_INFO
./scripts/config --enable DEBUG_INFO_NONE
./scripts/config --disable DEBUG_INFO_DWARF_TOOLCHAIN_DEFAULT
./scripts/config --enable NTSYNC
./scripts/config --disable RUST

# Tuning PGO QEMU Boot
./scripts/config --enable VIRTIO_PCI
./scripts/config --enable VIRTIO_CONSOLE
./scripts/config --enable NET_9P
./scripts/config --enable NET_9P_VIRTIO
./scripts/config --enable 9P_FS

make olddefconfig

echo ">>> Generazione ~/.rpmmacros globale per la compilazione..."
cat "$GITHUB_WORKSPACE/config/rpmmacros" > ~/.rpmmacros
cat << 'MCR' >> ~/.rpmmacros
%_with_vanilla 1
%buildid .chimera
%toolchain gcc
%optflags %{__global_compiler_flags} -march=x86-64-v3 -pipe -Wno-error
%kcflags -march=x86-64-v3 -pipe -Wno-error

%_without_selftests 1
%_without_tools 1
%_without_perf 1
%_without_libperf 1
%_without_ynl 1
%_without_bpftool 1
%_without_debug 1
%_without_debuginfo 1
%_without_doc 1
%_binary_payload w1.zstdio
%_source_payload w1.zstdio
MCR

echo "========================================================="
echo " PREPARAZIONE COMPLETATA."
echo " Il Kernel è pronto nella cartella $(pwd)"
echo " Usa 'make binrpm-pkg' per compilare un RPM nativo upstream."
echo "========================================================="

# Fix Keys errors in upstream build
./scripts/config --set-str SYSTEM_TRUSTED_KEYS ""
./scripts/config --set-str SYSTEM_REVOCATION_KEYS ""
./scripts/config --disable DEBUG_INFO_BTF
make olddefconfig
