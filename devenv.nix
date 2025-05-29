{pkgs, ...}: {
  packages = with pkgs; [
    # Rust toolchain
    rustup

    # Code formatting tools
    treefmt
    alejandra
    rustfmt

    # Rust dependency linting
    cargo-deny
  ];
}
