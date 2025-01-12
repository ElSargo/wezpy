{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
    };
  };

  outputs = {
    nixpkgs,
    flake-utils,
    fenix,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      toolchain = fenix.packages.${system}.latest.toolchain;

      nativeBuildInputs = with pkgs; [
        maturin
        pkg-config
        toolchain
        pythonVersion
      ];

      buildInputs = with pkgs; [libz gcc openssl];

      craneLib =
        (crane.mkLib pkgs).overrideToolchain toolchain;
      projectName =
        (craneLib.crateNameFromCargoToml {cargoToml = ./Cargo.toml;}).pname;
      projectVersion =
        (craneLib.crateNameFromCargoToml {
          cargoToml = ./Cargo.toml;
        })
        .version;

      pythonVersion = pkgs.python311;
      wheelName = "wezpy-0.1.0-cp311-cp311-linux_x86_64.whl";

      txtFilter = path: _type: builtins.match ".*wezterm-src/.*(txt|pest|)$" path != null;
      txtOrCargo = path: type:
        (txtFilter path type) || (craneLib.filterCargoSources path type);

      crateCfg = {
        src = pkgs.lib.cleanSourceWith {
          src = ./.; 
          filter = txtOrCargo;
          name = "source"; # Be reproducible, regardless of the directory name
        };
        inherit nativeBuildInputs buildInputs;
      };
      crateWheel =
        (craneLib.buildPackage (crateCfg
          // {
            pname = projectName;
            version = projectVersion;
          }))
        .overrideAttrs (old: {
          doNotPostBuildInstallCargoBinaries = true;
          nativeBuildInputs = old.nativeBuildInputs ;
          buildPhase = #bash
            ''
              runHook preBuild
              maturin build --offline --target-dir ./target 

              cd target/wheels/
              ${pkgs.unzip}/bin/unzip ${wheelName} -d ext
              cp ${pkgs.libz}/lib/* ext/wezpy.libs/

              runHook postBuild
            '';
          installPhase =
            ''
              mkdir $out
              cd ext
              ${pkgs.zip}/bin/zip -r $out/${wheelName} .
            '';
        });
    in rec {
      lib = {
        pythonPackage = ps:
          ps.buildPythonPackage {
            pname = projectName;
            format = "wheel";
            version = projectVersion;
            src = "${crateWheel}/${wheelName}";
            doCheck = false;
          };
      };
      formatter = pkgs.alejandra;
      packages = {
        default = crateWheel;
        python = pkgs.python311.withPackages (ps: [ (lib.pythonPackage ps) ]);
      };
      devShells.default = pkgs.mkShell {
        inherit nativeBuildInputs buildInputs;
        LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath buildInputs;
      };
    });
}
