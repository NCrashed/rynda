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
    xlibs.libXcursor
    xlibs.libXrandr
    freetype
    expat
    libxkbcommon
    cmake
    openssl
    python3
    libGL
    glfw
    # vulkan-validation-layers
  ];

  APPEND_LIBRARY_PATH = lib.makeLibraryPath [
    # vulkan-loader
    libGL
    glfw
    xlibs.libXcursor
    xlibs.libXi
    xlibs.libXrandr
  ];

  shellHook = ''
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$APPEND_LIBRARY_PATH"
  '';
}
