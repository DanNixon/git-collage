{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";

    flake-utils.url = "github:numtide/flake-utils";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

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

        toolchain = fenix.packages.${system}.toolchainOf {
          channel = "1.76";
          date = "2024-02-08";
          sha256 = "e4mlaJehWBymYxJGgnbuCObVlqMlQSilZ8FljG9zPHY=";
        };

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain.rust;
          rustc = toolchain.rust;
        };

        cargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        name = cargo.package.name;
        version = cargo.package.version;

        nativeBuildInputs = with pkgs; [pkg-config];
        buildInputs = with pkgs; [openssl];
      in {
        devShell = pkgs.mkShell {
          nativeBuildInputs = nativeBuildInputs;
          buildInputs = buildInputs;

          packages = with pkgs; [
            # Rust toolchain
            toolchain.toolchain

            # Code formatting tools
            alejandra
            treefmt

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
