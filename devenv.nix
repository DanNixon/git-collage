{pkgs, ...}: {
  packages = with pkgs; [
    # Rust toolchain
    rustup
    cargo-cross

    # Code formatting tools
    treefmt
    alejandra
    rustfmt

    # Rust dependency linting
    cargo-deny

    # Container image tools
    buildah
    skopeo

    # Dependencies
    pkg-config
    openssl
  ];
}
