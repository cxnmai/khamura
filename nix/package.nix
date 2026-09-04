{ pkgs }:
let
  dependencies = import ./dependencies.nix { inherit pkgs; };
  manifest = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in pkgs.rustPlatform.buildRustPackage (dependencies.environment // {
  pname = manifest.package.name;
  version = manifest.package.version;
  src = pkgs.lib.cleanSource ../.;
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = dependencies.nativeBuildInputs ++ [ pkgs.makeWrapper ];
  buildInputs = dependencies.runtimeLibraries;
  nativeCheckInputs = [ pkgs.ffmpeg-full ];
  KHAMURA_VIDEO_ENCODER = "software";

  postInstall = ''
    wrapProgram "$out/bin/khamura" \
      --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath dependencies.runtimeLibraries}" \
      --prefix PATH : "${pkgs.lib.makeBinPath [ pkgs.xdg-utils ]}"
    install -Dm644 assets/khamura.svg "$out/share/icons/hicolor/scalable/apps/khamura.svg"
    mkdir -p "$out/share/applications"
    cat > "$out/share/applications/khamura.desktop" <<DESKTOP
    [Desktop Entry]
    Type=Application
    Name=Khamura
    Comment=Take photos and record videos
    Exec=$out/bin/khamura
    Icon=khamura
    Terminal=false
    Categories=AudioVideo;Video;
    Keywords=camera;photo;video;webcam;
    StartupWMClass=khamura
    DESKTOP
  '';

  meta = {
    description = manifest.package.description;
    homepage = manifest.package.repository;
    platforms = pkgs.lib.platforms.linux;
    mainProgram = "khamura";
  };
})
