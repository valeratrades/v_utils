{
  inputs = {
    v_flakes.url = "github:valeratrades/v_flakes?ref=v1.6";
  };

  outputs = { self, v_flakes }:
    let
      inherit (v_flakes) flake-utils pre-commit-hooks;
    in
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import v_flakes.default_nixpkgs { inherit system; };
        rust = v_flakes.rs.default_nightly system;

        pre-commit-check = pre-commit-hooks.lib.${system}.run (v_flakes.files.preCommit { inherit pkgs; });
        manifest = (pkgs.lib.importTOML ./v_utils/Cargo.toml).package;
        pname = manifest.name;
        stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv;

        rs = v_flakes.rs {
          inherit pkgs rust;
          build = {
            enable = true;
            deny = true;
            workspace = let deprecate_by = "v3.0.0"; in {
              # Pinned here rather than in v_flakes because src/lwc/*.js is written against this exact API.
              "./v_utils" = [ "git_version" "log_directives" { lightweight_charts = { version = "5.2.0"; sha256 = "66ac22df1b08de08ec2fae2b401b0f9731a4653a28e18a9837c7c3553c33dbe2"; }; } { deprecate = { by_version = deprecate_by; force = true; }; } ];
              "./v_utils_macros" = [{ deprecate = { by_version = deprecate_by; force = true; }; }];
            };
          };
        };
        github = v_flakes.github {
          inherit pkgs pname rs;
          lastSupportedVersion = "nightly-${v_flakes.rs.nightly_version}";
          enable = true;
          jobs = {
            default = true;
            warnings.augment = [
              { name = "rust-doc"; args = { package = "v_utils"; }; }
            ];
            warnings.exclude = [ "rust-doc" ];
          };
        };
        readme = v_flakes.readme-fw { inherit pkgs pname; defaults = true; lastSupportedVersion = "nightly-1.98"; rootDir = ./.; badges = [ "msrv" "crates_io" "docs_rs" "loc" "ci" ]; };
        combined = v_flakes.utils.combine { inherit rust; modules = [ rs github readme ]; };
      in
      {
        devShells.default = with pkgs; mkShell {
          inherit stdenv;
          shellHook =
            pre-commit-check.shellHook
            + combined.shellHook
            + ''
              cp -f ${(v_flakes.files.treefmt) {inherit pkgs;}} ./.treefmt.toml
            '';

          buildInputs = [
            mold
            openssl
            pkg-config
            rust
          ] ++ pre-commit-check.enabledPackages ++ combined.enabledPackages;
        };
      }
    );
}

