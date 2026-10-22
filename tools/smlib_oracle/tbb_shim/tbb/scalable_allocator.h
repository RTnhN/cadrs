// Serial stand-in for the parts of oneTBB that SMLib uses, so the oracle builds without TBB's
// headers (and runs deterministically).
#pragma once
#include <cstdlib>
inline void* scalable_malloc(size_t n) { return std::malloc(n); }
inline void* scalable_calloc(size_t n, size_t s) { return std::calloc(n, s); }
inline void* scalable_realloc(void* p, size_t n) { return std::realloc(p, n); }
inline void scalable_free(void* p) { std::free(p); }
