// Serial stand-in for tbb::parallel_for and tbb::task_arena (see scalable_allocator.h).
#pragma once
namespace tbb {
template <typename I, typename F> void parallel_for(I first, I last, const F& f) {
    for (I i = first; i < last; ++i) f(i);
}
template <typename R, typename F> void parallel_for(const R& range, const F& f) { f(range); }
class task_arena {
public:
    template <typename F> auto execute(const F& f) -> decltype(f()) { return f(); }
};
}
