{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-23.11";

    flake-utils.url = "github:numtide/flake-utils";

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
    naersk,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk {
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
          default = naersk'.buildPackage {
            name = name;
            version = version;

            src = ./.;

            nativeBuildInputs = nativeBuildInputs;
            buildInputs = buildInputs;
          };

          clippy = naersk'.buildPackage {
            mode = "clippy";
            src = ./.;

            nativeBuildInputs = nativeBuildInputs;
            buildInputs = buildInputs;
          };

          test = naersk'.buildPackage {
            mode = "test";
            src = ./.;

            nativeBuildInputs = nativeBuildInputs;
            buildInputs = buildInputs;

            # Ensure detailed test output appears in nix build log
            cargoTestOptions = x: x ++ ["1>&2"];
          };
        };
      }
    );
}
