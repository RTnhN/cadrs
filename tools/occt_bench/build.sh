#!/bin/sh
# Builds the OCCT side of tools/kernel_perf.py (occt_bench.cpp) against a static OCCT 7.8
# (include/ and lib/ under $OCCT), to the path given (default target/occt_bench).
#
#   tools/occt_bench/build.sh [out]
set -e
O=${OCCT:-/home/casper/work/cadrs/.claude/worktrees/phase3h/target/OCCT}
here=$(dirname "$0")
out=${1:-$here/../../target/occt_bench}
mkdir -p "$(dirname "$out")"
g++ -O2 -std=c++17 -I"$O/include/opencascade" -I"$O/include" "$here/occt_bench.cpp" -L"$O/lib" -Wl,--start-group \
  -lTKBO -lTKBRep -lTKBool -lTKDE -lTKDESTEP -lTKFeat -lTKFillet -lTKG2d -lTKG3d -lTKGeomAlgo -lTKGeomBase \
  -lTKMath -lTKMesh -lTKOffset -lTKPrim -lTKShHealing -lTKTopAlgo -lTKXSBase -lTKCAF -lTKCDF -lTKLCAF -lTKXCAF \
  -lTKVCAF -lTKService -lTKV3d -lTKHLR -lTKernel -Wl,--end-group -lpthread -ldl -o "$out"
