#pragma once
#include "rust/cxx.h"
namespace cadrs_sectors {
struct Piece;
rust::Vec<Piece> convex_sector(rust::Slice<const uint8_t> face, double t);
}
