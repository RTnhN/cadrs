# SMLib oracle

A second, independent B-rep kernel for the native kernel's tests: SMLib from NVIDIA's
[usd-brep](https://github.com/NVIDIA-Omniverse/usd-brep) (Apache-2.0), built here with a
serial stand-in for the few oneTBB pieces it uses (`tbb_shim/`), so no TBB headers are needed.

    cmake -S tools/smlib_oracle -B target/smlib_oracle -DUSD_BREP_DIR=<usd-brep checkout> -DCMAKE_BUILD_TYPE=Release
    cmake --build target/smlib_oracle -j
    CADRS_SMLIB_ORACLE=$PWD/target/smlib_oracle/smlib_oracle cargo test -r -p cadrs_kernel --test smlib_oracle -- --nocapture

`-DSMLIB_MESSAGES=ON` builds SMLib with its error messages (each failure's file and line on
stderr with `ORACLE_VERBOSE=1`), in a separate build directory.

The native kernel writes its bodies as `.smb` (`native::oracle::smb_text`); `smlib_oracle`
reads them and prints one line of JSON (volume, area, counts, whether the result is a manifold
solid):

    smlib_oracle props  in.smb
    smlib_oracle fillet in.smb out.smb radius x y z [x y z ...]   # the edges nearest the points
