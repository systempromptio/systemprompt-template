{
  description = "systemprompt — AI governance gateway";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default;
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        version = cargoToml.workspace.package.version;
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "systemprompt";
          inherit version;
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
          nativeBuildInputs = with pkgs; [ pkg-config clang mold ];
          buildInputs = with pkgs; [ openssl postgresql ];
          SQLX_OFFLINE = "true";
          doCheck = false;
          meta = with pkgs.lib; {
            description = "AI governance gateway for Claude, OpenAI, and Gemini";
            homepage = "https://systemprompt.io";
            # Template package is MIT (this flake builds the template's source).
            # The compiled artifact links systemprompt-core which is BSL-1.1;
            # see CONFLICT IN BINARY DISTRIBUTION below — production users need
            # a commercial licence for systemprompt-core, but Nix's `meta.license`
            # tracks source licence only, so MIT here is correct.
            license = licenses.mit;
            platforms = platforms.unix;
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            openssl
            postgresql
            just
            sqlx-cli
          ];
        };

        checks.build = self.packages.${system}.default;
      });
}
