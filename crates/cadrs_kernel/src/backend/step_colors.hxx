#pragma once
#include "rust/cxx.h"
namespace cadrs_colors {
struct Model;
Model read_step_colors(rust::Str path);
rust::Vec<uint8_t> step_color_fixture();
}
