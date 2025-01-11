{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane = {
      url = "github:ipetkov/crane";
    };
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, crane, flake-utils, fenix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
          craneLib = crane.lib.${system}.overrideToolchain
            fenix.packages.${system}.minimal.toolchain;
          pkgs = nixpkgs.legacyPackages.${system};

        nativeBuildInputs = with pkgs; [
          pkg-config
          openssl
          maturin
          fenix.packages.${system}.latest.toolchain
        ];

        buildInputs = with pkgs; [ libz gcc ];

        core = craneLib.buildPackage {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          inherit nativeBuildInputs buildInputs;
        };

      in {
        packages.default = core;
        apps.default = core;
        devShells.default =
          pkgs.mkShell { inherit nativeBuildInputs buildInputs ;  
          LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath buildInputs;
           };
      });
}


