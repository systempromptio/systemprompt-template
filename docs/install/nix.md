# Install the gateway via Nix

Builds the `systemprompt-gateway` server from the flake at the root of this repo. For the cowork client CLI, see [../cowork/](../cowork/).

The repo ships a flake at the root, so no external registry — you consume it straight from GitHub.

## Run once (no install)

```bash
nix run github:systempromptio/systemprompt-template -- --help
```

## Install into your profile

```bash
nix profile install github:systempromptio/systemprompt-template
systemprompt --version
```

## Pin a version

```bash
nix run github:systempromptio/systemprompt-template/v0.2.2 -- --version
```

## NixOS module (flake input)

In your `flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systemprompt.url = "github:systempromptio/systemprompt-template";
  };

  outputs = { self, nixpkgs, systemprompt, ... }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ({ pkgs, ... }: {
          environment.systemPackages = [
            systemprompt.packages.${pkgs.system}.default
          ];
        })
      ];
    };
  };
}
```

## Development shell

The flake also exposes a dev shell with the full toolchain:

```bash
nix develop github:systempromptio/systemprompt-template
```

Gives you `cargo`, `rustc`, `pkg-config`, `openssl`, `postgresql`, `just`, and `sqlx-cli` on `$PATH`.

## Build locally

```bash
git clone https://github.com/systempromptio/systemprompt-template
cd systemprompt-template
nix build
./result/bin/systemprompt --version
```

Docs: https://systemprompt.io/documentation/?utm_source=nix&utm_medium=install_doc
