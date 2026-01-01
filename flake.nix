{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks.url = "github:cachix/git-hooks.nix/ca5b894d3e3e151ffc1db040b6ce4dcc75d31c37";
    v-utils.url = "github:valeratrades/.github?ref=v1.3";
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
          inherit pkgs pname; lastSupportedVersion = "nightly-2025-10-12";
          langs = [ "rs" ];
          jobsErrors = [ "rust-tests" ];
          jobsWarnings = [
            { name = "rust-doc"; args = { package = "v_utils"; }; }
            "rust-clippy"
            "rust-machete"
            "rust-sorted"
            "rust-unused-features"
            "rust-sorted-derives"
            "tokei"
          ];
          jobsOther = [ "loc-badge" ];
        };
        readme = v-utils.readme-fw { inherit pkgs pname; lastSupportedVersion = "nightly-1.92"; rootDir = ./.; licenses = [{ name = "Blue Oak 1.0.0"; outPath = "LICENSE"; }]; badges = [ "msrv" "crates_io" "docs_rs" "loc" "ci" ]; };

        rs = v-utils.rs {
          inherit pkgs;
          build = {
            enable = true;
            deny = true;
            workspace = {
              "./v_utils" = [ "git_version" "log_directives" { deprecate = "v3.0.0"; } ];
              "./v_utils_macros" = [{ deprecate = "v3.0.0"; }];
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
            ''
              cp -f ${v-utils.files.licenses.blue_oak} ./LICENSE

              cp -f ${(v-utils.files.treefmt) {inherit pkgs;}} ./.treefmt.toml

              cp -f ${readme} ./README.md
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

