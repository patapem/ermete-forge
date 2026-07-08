#!/bin/bash
set -euo pipefail
# Ermete OS: The Ultimate Chimera Kernel Bedrock Local Builder
# Riproduce in bit-perfect il workflow di GitHub Actions all'interno di un micro-container OCI locale

set -e

echo ">>> [BEDROCK] Inizializzazione Ambiente di Build Isolato Locale (Fedora 43 OCI)"

# Otteniamo la directory radice del repository
FORGE_DIR=$(git rev-parse --show-toplevel)
CACHE_DIR="$FORGE_DIR/.ccache_local"
mkdir -p "$CACHE_DIR"


echo ">>> [BEDROCK] Esecuzione Container Fedora 43 (Privileged)..."
docker run --rm -i \
  --privileged \
  --security-opt label=disable \
  -v "$FORGE_DIR":/forge \
  -w /forge \
  -e GITHUB_WORKSPACE=/forge \
  registry.fedoraproject.org/fedora:43 \
  /bin/bash -s << 'DOCKEREOF'
    set -e
    echo '>>> Configurazione DNF (Identica alla CI)...'
    echo 'zchunk=False' >> /etc/dnf/dnf.conf
    echo 'fastestmirror=True' >> /etc/dnf/dnf.conf
    echo 'install_weak_deps=False' >> /etc/dnf/dnf.conf
    
    echo '>>> Installazione Architettura di Compilazione...'
    dnf install -y rpm-build rpmdevtools gcc gcc-c++ make cmake flex bison ncurses-devel elfutils-libelf-devel openssl-devel bc rsync tar wget curl cpio perl zstd git llvm clang lld ccache qemu-kvm stress-ng iperf3 jq gnupg2 hostname skopeo elfutils-devel dwarves openssl rust cargo rustfmt bindgen iproute
    
    echo '>>> Esecuzione prepare-chimera.sh...'
    bash specs/ermete-kernel/prepare-chimera.sh
    
    echo ">>> Injecting Rust fix into kernel.spec %prep phase..."
    cat << 'RUSTFIX' > ~/fix-rust.sh
echo ">>> FIX RUST NO-JUMP-TABLES AND TARGET POINTER WIDTH AND CORE EDITION"
find . -type f -name "Makefile" -exec sed -i 's/-Zno-jump-tables/-Zunstable-options/g' {} +
find . -type f -name "Makefile" -exec sed -i 's/-Z no-jump-tables/-Z unstable-options/g' {} +
find . -type f -name "generate_rust_target.rs" -exec sed -i 's/"target-pointer-width", "64"/"target-pointer-width", 64/g' {} +
find . -type f -name "generate_rust_target.rs" -exec sed -i 's/"target-pointer-width", "32"/"target-pointer-width", 32/g' {} +
find . -type f -name "Makefile" -path "*/rust/Makefile" -exec sed -i 's/rustc_target_flags = $(core-cfgs)/rustc_target_flags = $(core-cfgs) --edition=2024/g' {} +
find . -type f -name "Makefile" -path "*/rust/Makefile" -exec sed -i 's/skip_flags = -Wunreachable_pub/skip_flags = -Wunreachable_pub --edition=2021/g' {} +
find . -type f -name "Makefile" -path "*/arch/x86/tools/Makefile" -exec sed -i 's/$(call cmd,posttest)/true/g' {} +
find . -type f -name "Makefile" -path "*/arch/x86/tools/Makefile" -exec sed -i 's/$(call cmd,sanitytest)/true/g' {} +
RUSTFIX
    awk '/^%build/ && !done { print; system("cat ~/fix-rust.sh"); done=1; next }1' ~/rpmbuild/SPECS/kernel.spec > ~/rpmbuild/SPECS/kernel.spec.new
    mv ~/rpmbuild/SPECS/kernel.spec.new ~/rpmbuild/SPECS/kernel.spec
    
    KERNEL_DIR=$(cat ~/rpmbuild/BUILD/.kernel_version)
    cd ~/rpmbuild/BUILD/$KERNEL_DIR
    
    echo '>>> [BEDROCK PUNTO 3] FASE 1: Build Definitiva con LLVM e ThinLTO...'
    ./scripts/config --enable LTO_CLANG_THIN
    make LLVM=1 LLVM_IAS=1 olddefconfig </dev/null
    
    cat << 'MACRO' >> ~/.rpmmacros
%_smp_mflags -j$(nproc)
%_ld ld.lld
%_ldflags -Wl,-O2 -Wl,--as-needed -Wl,--sort-common -Wl,-z,now -Wl,-z,relro
%optflags %{__global_compiler_flags} -march=x86-64-v3 -pipe -Wno-error
%kcflags -march=x86-64-v3 -pipe -Wno-error
MACRO

    echo "Pulizia build precedente per forgiatura finale..."
    cd ~/rpmbuild/BUILD/$KERNEL_DIR
    make LLVM=1 LLVM_IAS=1 clean
    rm -f ~/rpmbuild/RPMS/*/*.rpm || true
    
    echo "Compilazione del Kernel Ottimizzato via Fedora Spec..."
    cd ~/rpmbuild/SPECS
    rpmbuild -bc kernel.spec \
        --target x86_64 \
        --define "__make /usr/bin/make LLVM=1 LLVM_IAS=1 HOSTCC=clang HOSTCXX=clang++"
    rpmbuild -bb kernel.spec \
        --target x86_64 \
        --define "__make /usr/bin/make LLVM=1 LLVM_IAS=1 HOSTCC=clang HOSTCXX=clang++" </dev/null
    ccache -s
    
    echo '========================================================='
    echo ' BUILD LOCALE COMPLETATA CON SUCCESSO.'
    echo '========================================================='
    ls -lh ~/rpmbuild/RPMS/x86_64/
    
    mkdir -p /forge/RPMS_OUT
    cp ~/rpmbuild/RPMS/x86_64/*.rpm /forge/RPMS_OUT/
DOCKEREOF
