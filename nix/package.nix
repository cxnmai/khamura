{ pkgs }:
let
  dependencies = import ./dependencies.nix { inherit pkgs; };
  manifest = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in pkgs.stdenvNoCC.mkDerivation {
  pname = manifest.package.name;
  version = "0.1.0";
  src = pkgs.fetchurl {
    url = "https://github.com/cxnmai/khamura/releases/download/v0.1.0/khamura-x86_64-unknown-linux-gnu-v0.1.0.tar.gz";
    hash = "sha256-EothgxC+b3MXuTIryx5+jhlXmPcPndx6EQSwdHGv0PU=";
  };

  nativeBuildInputs = [ pkgs.autoPatchelfHook pkgs.makeWrapper ];
  buildInputs = dependencies.runtimeLibraries ++ [ pkgs.stdenv.cc.cc.lib ];
  dontConfigure = true;
  dontBuild = true;
  installPhase = ''
    runHook preInstall
    install -Dm755 khamura "$out/bin/khamura"
    install -Dm644 LICENSE "$out/share/licenses/khamura/LICENSE"
    runHook postInstall
  '';

  postInstall = ''
    wrapProgram "$out/bin/khamura" \
      --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath dependencies.runtimeLibraries}" \
      --prefix PATH : "${pkgs.lib.makeBinPath [ pkgs.xdg-utils ]}" \
      --set-default KHAMURA_FFMPEG "${pkgs.lib.getExe pkgs.ffmpeg-full}" \
      --set-default KHAMURA_FFPLAY "${pkgs.lib.getExe' pkgs.ffmpeg-full "ffplay"}" \
      --set-default KHAMURA_FFPROBE "${pkgs.lib.getExe' pkgs.ffmpeg-full "ffprobe"}" \
      --set-default KHAMURA_PACTL "${pkgs.lib.getExe' pkgs.pulseaudio "pactl"}"
    install -Dm644 ${../assets/khamura.svg} "$out/share/icons/hicolor/scalable/apps/khamura.svg"
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
    platforms = [ "x86_64-linux" ];
    mainProgram = "khamura";
  };
}
