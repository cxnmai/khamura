{ pkgs }:
{
  runtimeLibraries = with pkgs; [
    fontconfig freetype libxkbcommon vulkan-loader wayland libxcb
  ];
  nativeBuildInputs = with pkgs; [
    pkg-config llvmPackages.libclang linuxHeaders cmake nasm
  ];
  environment = {
    KHAMURA_PACTL = pkgs.lib.getExe' pkgs.pulseaudio "pactl";
    KHAMURA_FFMPEG = pkgs.lib.getExe pkgs.ffmpeg-full;
    KHAMURA_FFPLAY = pkgs.lib.getExe' pkgs.ffmpeg-full "ffplay";
    KHAMURA_FFPROBE = pkgs.lib.getExe' pkgs.ffmpeg-full "ffprobe";
    LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
    BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.linuxHeaders}/include -I${pkgs.stdenv.cc.libc_dev}/include";
    ZED_PATH_SAMPLE_COUNT = "0";
  };
}
