// Serial stand-in for tbb::blocked_range2d (see scalable_allocator.h).
#pragma once
namespace tbb {
template <typename T> class blocked_range {
public:
    blocked_range(T b, T e) : b_(b), e_(e) {}
    T begin() const { return b_; }
    T end() const { return e_; }
private:
    T b_, e_;
};
template <typename R, typename C = R> class blocked_range2d {
public:
    blocked_range2d(R rb, R re, C cb, C ce) : rows_(rb, re), cols_(cb, ce) {}
    const blocked_range<R>& rows() const { return rows_; }
    const blocked_range<C>& cols() const { return cols_; }
private:
    blocked_range<R> rows_;
    blocked_range<C> cols_;
};
}
