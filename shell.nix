with import <nixpkgs> {};

stdenv.mkDerivation rec {
  name = "finalfusion-utils-env";
  env = buildEnv { name = name; paths = buildInputs; };

  nativeBuildInputs = [ pkgconfig ];
  buildInputs = [
    rustup
    alsaLib
    xorg.libX11
    xorg.libXi
    xorg.libXinerama
    xorg.libXext
    xorg.libXcursor
    xorg.libXrandr
    freetype
    expat
    libxkbcommon
    cmake
    openssl
    python3
    libGL
    glfw
    renderdoc # for debugging
  ];

  APPEND_LIBRARY_PATH = lib.makeLibraryPath [
    libGL
  ];

  shellHook = ''
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$APPEND_LIBRARY_PATH"
  '';
}
