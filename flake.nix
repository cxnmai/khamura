{
  description = "Khamura camera app and development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          khamura = import ./nix/package.nix { inherit pkgs; };
        in { inherit khamura; default = khamura; });

      devShells = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          dependencies = import ./nix/dependencies.nix { inherit pkgs; };
        in {
          default = pkgs.mkShell (dependencies.environment // {
            nativeBuildInputs = dependencies.nativeBuildInputs ++ (with pkgs; [
              cargo rustc ffmpeg-full pulseaudio
            ]);
            buildInputs = dependencies.runtimeLibraries;
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath dependencies.runtimeLibraries;
          });
        });
    };
}
