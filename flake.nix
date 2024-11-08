{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsFor = nixpkgs.legacyPackages;
    in {
      packages = forAllSystems (system: let
        pkgs = pkgsFor.${system};
        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
      in {
        #HACK: this is not ever supposed to be built, I just couldn't get `shell.nix` to bring the correct `openssl` path in
        default = pkgs.rustPlatform.buildRustPackage rec {
          pname = manifest.name;
          version = manifest.version;

          buildInputs = with pkgs; [
            openssl
            openssl.dev
          ];
          nativeBuildInputs = with pkgs; [ pkg-config ];
          env.PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

          cargoLock.lockFile = ./Cargo.lock;
          src = pkgs.lib.cleanSource ./.;
        };
      });
    };
}