{
  description = "Exa CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "exa";
          version = "0.1.1";
          src = self;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          meta = {
            mainProgram = "exa";
            description = "Exa CLI";
            homepage = "https://github.com/radjathaher/exa-cli";
            license = pkgs.lib.licenses.mit;
          };
        };
      }
    );
}
