#!/bin/sh
set -eu

APP_NAME="bashers"
GITHUB_REPO="${BASHERS_GITHUB_REPO:-yourusername/bashers}"

detect_arch() {
    local arch
    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64)
            echo "x86_64"
            ;;
        *)
            echo "Unsupported architecture: $arch (only x86_64 is supported)" >&2
            exit 1
            ;;
    esac
}

get_latest_version() {
    if command -v curl >/dev/null 2>&1; then
        curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | \
            grep '"tag_name":' | \
            sed -E 's/.*"([^"]+)".*/\1/' | \
            sed 's/^v//'
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | \
            grep '"tag_name":' | \
            sed -E 's/.*"([^"]+)".*/\1/' | \
            sed 's/^v//'
    else
        echo "Error: curl or wget is required" >&2
        exit 1
    fi
}

download_binary() {
    local version="$1"
    local url
    local filename
    local bin_dir="${BASHERS_INSTALL_DIR:-$HOME/.local/bin}"
    
    url="https://github.com/$GITHUB_REPO/releases/download/v${version}/bashers-linux-x86_64.tar.gz"
    filename="bashers-linux-x86_64.tar.gz"
    
    echo "Downloading $APP_NAME $version for linux-x86_64..."
    
    mkdir -p "$bin_dir"
    
    if command -v curl >/dev/null 2>&1; then
        curl -fL "$url" -o "/tmp/$filename"
    elif command -v wget >/dev/null 2>&1; then
        wget "$url" -O "/tmp/$filename"
    else
        echo "Error: curl or wget is required" >&2
        exit 1
    fi
    
    echo "Extracting..."
    tar -xzf "/tmp/$filename" -C "/tmp"
    
    mv "/tmp/bashers" "$bin_dir/bashers"
    chmod +x "$bin_dir/bashers"
    rm -f "/tmp/$filename"
    
    echo ""
    echo "✓ $APP_NAME $version installed to $bin_dir/bashers"
    echo ""
    
    if ! echo "$PATH" | grep -q "$bin_dir"; then
        echo "⚠️  $bin_dir is not in your PATH"
        echo "Add this to your shell configuration:"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
    fi
}

main() {
    local version="${1:-}"
    
    if [ -z "$version" ]; then
        version="$(get_latest_version)"
    fi
    
    detect_arch >/dev/null  # Check architecture is supported
    
    download_binary "$version"
}

main "$@"
