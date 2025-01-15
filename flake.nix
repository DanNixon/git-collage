{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = pkgs.cargo;
          rustc = pkgs.rustc;
        };

        cargoMetadata = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        name = cargoMetadata.package.name;
        version = cargoMetadata.package.version;

        nativeBuildInputs = with pkgs; [pkg-config];
        buildInputs = with pkgs; [openssl];
      in {
        devShell = pkgs.mkShell {
          nativeBuildInputs = nativeBuildInputs;
          buildInputs = buildInputs;

          packages = with pkgs; [
            # Rust toolchain
            cargo
            rustc

            # Code analysis tools
            clippy
            rust-analyzer

            # Code formatting tools
            treefmt
            alejandra
            rustfmt

            # Rust dependency linting
            cargo-deny
          ];

          RUSTFLAGS = "-D unused-crate-dependencies";
        };

        packages = rec {
          default = rustPlatform.buildRustPackage {
            pname = name;
            version = version;

            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = nativeBuildInputs;
            buildInputs = buildInputs;
          };

          container-image = pkgs.dockerTools.buildImage {
            name = name;
            tag = "latest";
            created = "now";

            runAsRoot = ''
              #!${pkgs.runtimeShell}
              mkdir -p /config
              mkdir -p /data
            '';

            config = {
              Entrypoint = ["${default}/bin/git-collage"];
              Env = [
                "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              ];
            };
          };
        };
      }
    );
}
