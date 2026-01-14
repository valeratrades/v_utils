{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks.url = "github:cachix/git-hooks.nix";
    v-utils.url = "github:valeratrades/.github?ref=v1.4";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, pre-commit-hooks, v-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = builtins.trace "flake.nix sourced" [ (import rust-overlay) ];
        rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" "rust-analyzer" "rust-docs" "rustc-codegen-cranelift-preview" ];
        });
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        pre-commit-check = pre-commit-hooks.lib.${system}.run (v-utils.files.preCommit { inherit pkgs; });
        manifest = (pkgs.lib.importTOML ./v_utils/Cargo.toml).package;
        pname = manifest.name;
        stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv;

        github = v-utils.github {
          inherit pkgs pname;
          lastSupportedVersion = "nightly-2025-10-12";
          langs = [ "rs" ];
          jobs = {
            default = true;
            warnings.augment = [
              { name = "rust-doc"; args = { package = "v_utils"; }; }
            ];
            warnings.exclude = [ "rust-doc" ];
          };
        };
        readme = v-utils.readme-fw { inherit pkgs pname; defaults = true; lastSupportedVersion = "nightly-1.92"; rootDir = ./.; badges = [ "msrv" "crates_io" "docs_rs" "loc" "ci" ]; };

        rs = v-utils.rs {
          inherit pkgs rust;
          build = {
            enable = true;
            deny = true;
            workspace = let deprecate_by = "v3.0.0"; in {
              "./v_utils" = [ "git_version" "log_directives" { deprecate = { by_version = deprecate_by; force = true; }; } ];
              "./v_utils_macros" = [{ deprecate = { by_version = deprecate_by; force = true; }; }];
            };
          };
        };
      in
      {
        devShells.default = with pkgs; mkShell {
          inherit stdenv;
          shellHook =
            pre-commit-check.shellHook +
            github.shellHook +
            rs.shellHook +
            readme.shellHook +
            ''
              cp -f ${(v-utils.files.treefmt) {inherit pkgs;}} ./.treefmt.toml
            '';

          buildInputs = [
            mold
            openssl
            pkg-config
            rust
          ] ++ pre-commit-check.enabledPackages ++ github.enabledPackages;
        };
      }
    );
}

