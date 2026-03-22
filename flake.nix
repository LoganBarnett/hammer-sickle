{
  description = "hammer-sickle - Run commands across Foreman-managed hosts";
  inputs = {
    # LLM: Do NOT change this URL unless explicitly directed. This is the
    # correct format for nixpkgs stable (25.11 is correct, not nixos-25.11).
    nixpkgs.url = "github:NixOS/nixpkgs/25.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, rust-overlay, crane }@inputs: let
    forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;
    overlays = [
      (import rust-overlay)
    ];
    pkgsFor = system: import nixpkgs {
      inherit system;
      overlays = overlays;
    };

    workspaceCrates = {
      cli = {
        name = "hammer-sickle-cli";
        binary = "hammer-sickle";
        description = "Run commands across Foreman-managed hosts";
      };
    };

    devPackages = pkgs: let
      rust = pkgs.rust-bin.stable.latest.default.override {
        extensions = [
          "rust-src"
          "rust-analyzer"
          "rustfmt"
        ];
      };
    in [
      rust
      pkgs.cargo-sweep
      pkgs.pkg-config
      pkgs.openssl
      pkgs.jq
    ];
  in {

    devShells = forAllSystems (system: let
      pkgs = pkgsFor system;
    in {
      default = pkgs.mkShell {
        buildInputs = devPackages pkgs;
        shellHook = ''
          echo "hammer-sickle development environment"
          echo ""
          echo "Available Cargo packages (use 'cargo build -p <name>'):"
          cargo metadata --no-deps --format-version 1 2>/dev/null | \
            jq -r '.packages[].name' | \
            sort | \
            sed 's/^/  • /' || echo "  Run 'cargo build' to get started"

          # Symlink cargo-husky hooks into .git/hooks/ using paths relative
          # to .git/hooks/ so the repo stays valid after moves or copies.
          _git_root=$(git rev-parse --show-toplevel 2>/dev/null)
          if [ -n "$_git_root" ] && [ -d ".cargo-husky/hooks" ]; then
            for _hook in .cargo-husky/hooks/*; do
              [ -x "$_hook" ] || continue
              _name=$(basename "$_hook")
              _dest="$_git_root/.git/hooks/$_name"
              _target=$(${pkgs.coreutils}/bin/realpath --relative-to="$_git_root/.git/hooks" "$(pwd)/$_hook")
              if [ ! -L "$_dest" ] || [ "$(readlink "$_dest")" != "$_target" ]; then
                ln -sf "$_target" "$_dest"
                echo "Installed git hook: $_name -> $_target"
              fi
            done
          fi
        '';
      };
    });

    packages = forAllSystems (system: let
      pkgs = pkgsFor system;
      craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.stable.latest.default);

      commonArgs = {
        src = craneLib.cleanCargoSource ./.;
        # LLM: Do NOT add darwin.apple_sdk.frameworks here - they were removed
        # in nixpkgs 25.11+. Use libiconv for Darwin builds instead.
        buildInputs = with pkgs; [
          openssl
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin; [
          libiconv
        ]);
        nativeBuildInputs = with pkgs; [
          pkg-config
        ];
        # Run only unit tests, skip integration tests that may need external
        # services unavailable in the Nix sandbox. Run all tests locally with
        # 'cargo test --all'.
        cargoTestExtraArgs = "--lib --bins";
      };

      cratePackages = pkgs.lib.mapAttrs (key: crate:
        let pkgFile = ./. + "/nix/packages/${key}.nix";
        in if builtins.pathExists pkgFile
          then import pkgFile { inherit craneLib commonArgs; }
          else craneLib.buildPackage (commonArgs // {
            pname = crate.name;
            cargoExtraArgs = "-p ${crate.name}";
          })
      ) workspaceCrates;

    in cratePackages // {
      default = craneLib.buildPackage (commonArgs // { pname = "hammer-sickle"; });
    });

    apps = forAllSystems (system: let
      pkgs = pkgsFor system;
    in pkgs.lib.mapAttrs (key: crate: {
      type = "app";
      program = "${self.packages.${system}.${key}}/bin/${crate.binary}";
    }) workspaceCrates);

  };
}
