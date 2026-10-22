## Abstract

Fast C implementations of four geometric predicates, the 2D and 3D orientation and incircle tests, are publicly available. Their inputs are ordinary single or double precision floating-point numbers. They owe their speed to two features. First, they employ new fast algorithms for arbitrary precision arithmetic that have a strong advantage over other software techniques in computations that manipulate values of extended but small precision. Second, they are adaptive; their running time depends on the degree of uncertainty of the result, and is usually small. These algorithms work on computers whose floating-point arithmetic uses radix two and exact rounding, including machines that comply with the IEEE 754 floating-point standard. Timings of the predicates, in isolation and embedded in 2D and 3D Delaunay triangulation programs, verify their effectiveness.

## 1 Introduction

Algorithms that make decisions based on geometric tests, such as determining which side of a line a point falls on, often fail when the tests return false answers because of roundoff error. The easiest solution to many of these robustness problems is to use software implementations of exact arithmetic, albeit often at great expense. The goal of improving the speed of correct geometric calculations has received much recent attention, but the most promising proposals take integer or rational inputs, typically of small precision. These methods do not appear to be usable if it is convenient or necessary to use ordinary floating-point inputs.

This paper describes two techniques for implementing fast exact (or extended precision) geometric calculations, and demonstrates them with implementations of four commonly used geometric predicates, the 2D and 3D orientation and incircle tests, available on the Web at 'http://www.cs.cmu.edu/ quake/robust.html'. The orientation test determines whether a point lies to the left of, to the

Supported in part by the Natural Sciences and Engineering Research Council of Canada under a 1967 Science and Engineering Scholarship and by the National Science Foundation under Grant ASC-9318163.

## Robust Adaptive Floating-Point Geometric Predicates

Jonathan Richard Shewchuk School of Computer Science Carnegie Mellon University Pittsburgh, Pennsylvania 15213 jrs@cs.cmu.edu

right of, or on a line or plane; it is an important predicate used in many (perhaps most) geometric algorithms. The incircle test determines whether a point lies inside, outside, or on a circle or sphere, and is used for Delaunay triangulation [8]. Inexact versions of these tests are vulnerable to roundoff error, and the wrong answers they produce can cause geometric algorithms to hang, crash, or produce incorrect output. Although exact arithmetic banishes these difficulties, it is common to hear reports of implementations being slowed by factors of ten or more as a consequence. For these reasons, computational geometry is an important arena for evaluating extended precision arithmetic schemes.

The first technique is a set of algorithms, several of them new, for performing arbitrary precision arithmetic. They differ from traditional methods in two ways. First, they gain speed by relaxing the usual requirement that extended precision numbers be normalized to fixed digit positions. Second, there is no cost to convert floating-point numbers to a specialized extended precision format. The method has its greatest advantage in computations that process values of extended but small precision (several hundred or thousand bits), and seems ideal for computational geometry. The method was largely developed by Priest [12, 13], who designed similar algorithms that run on a wide variety of floating-point architectures, with different radices and rounding behavior. I have made significant speed improvements by relaxing Priest's normalization requirement and optimizing for radix two with exact rounding. This specialization is justified by the wide acceptance of the IEEE 754 floating-point standard.

The second technique is to exploit features of the predicates that frequently make it possible for them to return correct answers without completing an exact computation. The orientation and incircle tests evaluate the sign of a matrix determinant. It is significant that only the sign, and not the magnitude, of the determinant is needed. Fortune and Van Wyk [5] take advantage of this fact by using a floating-point filter: the determinant is first evaluated approximately, and only if forward error analysis indicates that the sign of the approximate result cannot be trusted does one use an exact test. I carry their suggestion to its logical extreme by computing a sequence of successively more accurate approximations to the determinant, stopping only when the accuracy of the sign is assured. To reduce computation time, each approximation reuses a previous, less accurate computation if it is economical to do so. This adaptive approach is described in Section 4, and its application to the orientation and incircle tests is described in Section 5.

## 2 Related Work

There are several exact arithmetic schemes designed specifically for computational geometry; most are methods of exactly evaluating the sign of a determinant. Clarkson [3] proposes an algorithm for using floating-point arithmetic to evaluate the sign of the determinant of a small matrix of integers. A variant of the modified Gram-Schmidt procedure is used to improve the conditioning of the matrix, so that the determinant can subsequently be evaluated safely by Gaussian elimination. The 53 bits of significand available in IEEE double precision numbers are sufficient to operate on 10 10 matrices of 32-bit integers. Clarkson's algorithm is naturally adaptive; its running time is small for matrices whose determinants are not near zero 1 .

Recently, Avnaim, Boissonnat, Devillers, Preparata, and Yvinec [1] proposed an algorithm to evaluate signs of determinants of 2 2 and 3 3 matrices of -bit integers using only and 1 -bit arithmetic, respectively. Surprisingly, this is sufficient even to implement the insphere test (which is normally written as a 4 4 or 5 5 determinant), but with a handicap in bit complexity; 53-bit double precision arithmetic is sufficient to correctly perform the insphere test on points having 24-bit integer coordinates.

Fortune and Van Wyk [5, 6] propose a more general approach (not specific to determinants, or even to predicates) that represents integers using a standard exact arithmetic technique with digits of radix 2 23 stored as double precision floating-point values. (53-bit double precision significands make it possible to add several products of 23-bit integers before it becomes necessary to normalize.) Rather than use a general-purpose arbitrary precision library, they have developed LN, an expression compiler that writes code to evaluate a specific expression exactly. The size of the operands is arbitrary, but is fixed when LN generates a C++ implementation of the expression. An expression can thus be used to generate several functions, each for arguments of different bit lengths. Because the expression and the bit lengths of all operands are known in advance, LN can tune the exact arithmetic aggressively, eliminating loops, function calls, and memory management. The running time of the function thus produced depends on the bit complexity of the inputs. Fortune and Van Wyk report an order-of-magnitude speed improvementover the use of libraries (for equal bit complexity). Furthermore, the expression compiler garners another speed improvement by installing floating-point filters wherever appropriate, calculating static error bounds automatically.

Karasick, Lieber, and Nackman [9] report their experiences optimizing a method for determinant evaluation using rational inputs. Their approach reduces the bit complexity of the inputs by performing arithmetic on intervals (with low precision bounds) rather than exact values. The determinant thus evaluated is also an interval; if it contains zero, the precision is increased and the determinant reevaluated. The procedure is repeated until the interval does not contain zero (or contains only zero), and the sign of the result is certain. Their approach is thus adaptive, although it does not appear to use the results of one iteration to speed the next.

1 The method presented in Clarkson's paper does not work correctly if the determinant is exactly zero, but Clarkson (personal communication) notes that it is easily fixed. 'By keeping track of the scaling done by the algorithm, an upper bound can be maintained for the magnitude of the determinant of the matrix. When that upper bound drops below one, the determinant must be zero, since the matrix entries are integers, and the algorithm can stop.'

Because the Clarkson and Avnaim et al. algorithms are effectively restricted to low precision integer coordinates, I do not compare their performancewith that of my algorithms, though theirs may be faster. Floating-point inputs are more difficult to work with than integer inputs, partly because of the potential for the bit complexity of intermediate values to grow more quickly. (The Karasick et al. algorithm also suffers this difficulty, and is probably not competitive against the other techniques discussed here, although it may be the best existing alternative for algorithms that require rational numbers, such as those computing exact line intersections.) When it is necessary for an algorithm to use floating-point coordinates, the aforementionedmethods are not currently an option (although it might be possible to adapt them using the techniques of Section 3). I am not aware of any prior literature on exact determinant evaluation that considers floating-point operands.

## 3 Arbitrary Precision Floating-Point

## 3.1 Background

Most modern processors support floating-point numbers of the form significand 2 exponent . The significand is represented by a -bit binary number of the form (where each denotes a single bit), plus one additional bit for the sign. This paper does not address issues of overflow and underflow, so I allow the exponent to be an integer in the range . (Fortunately, many applications have inputs that fall within a circumscribed exponent range and will not overflow or underflow.) See the survey by Goldberg [7] for a detailed explanation of floating-point storage formats, particularly the IEEE 754 standard.

Most arbitrary precision libraries store numbers in a multiple-digit format, consisting of a sequence of digits (usually of large radix, like 2 32 ) coupled with a single exponent. A freely available example of the multiple-digit approach is Bailey's MPFUN package [2], a sophisticated portable multiprecision library that uses digits of machine-dependent radix (usually 2 24 ) stored as single precision floating-point values. An alternative is the multiple-term format, wherein a number is expressed as a sum of ordinary floating-point words, each with its own significand and exponent [12]. This approach has the advantage that the result of an addition like 2 300 2 300 (which may well arise in a determinant computation with machine precision floating-point inputs) can be stored in two words of memory, whereas the multiple-digit approach will use at least 601 bits to store the sum, and incur a corresponding speed penalty when performing arithmetic with it.

For the algorithms herein, each arbitrary precision value is expressed as an expansion 2 2 1 , where each is called a component of and is represented by a floating-point value with a -bit significand. To impose some structure on expansions, they are required to be nonoverlapping and ordered by magnitude ( largest, 1 smallest). Two floating-point values and are nonoverlapping if the least significant nonzero bit of is more significant than the most significant nonzero bit of , or vice-versa; for instance, the binary values 1100 and 10 1 are nonoverlapping, whereas 101 and 10 overlap 3 . The number zero does not overlap any number. An expansion is nonoverlapping if all its components are mutually nonoverlapping. Note that a number may be represented by many possible nonoverlappingexpansions; consider 1100 10 1 1001 0 1 1000 1 0 1. A nonoverlapping expansion is desirable because it is easy to determine its sign (take the sign of the largest component) or to produce a crude approximation of its value (take the largest component). Ci 二 十

Multiple-term algorithms can be faster than multiple-digit algorithms because the latter require expensive normalization of results to fixed digit positions, whereas multiple-term algorithms can allow the boundaries between terms to wander freely. Boundaries are still enforced, but can fall at any bit position. In addition, it usually takes time to convert an ordinary floating-point number to the internal format of a multiple-digit library, whereas that number is an expansion of length one. Conversion overhead can be significant for small extended precision calculations.

The central conceptual difference between standard multiple-digit algorithms and the algorithms described herein is that the former perform exact arithmetic by keeping the bit complexity of operands small enough to avoid roundofferror, whereas the latter allow roundoff to occur, then account for it after the fact. To measure roundoff quickly and correctly, a certain standard of accuracy is required from the processor's floating-point units. The algorithms presented herein rely on the assumption that addition, subtraction, and multiplication are performed with exact rounding . This means that if the exact result can be stored in a -bit significand, then the exact result is produced; if it cannot, then it is rounded to the nearest -bit floating-point value (with ties broken arbitrarily). For instance, in four-bit arithmetic the product 111 101 100011 is rounded to 1 001 2 5 . Throughout this paper, the symbols , , and represent -bit floating-point addition, subtraction, and multiplication with exact rounding. A number is said to be expressible in bits if it can be expressed with a -bit significand, not counting the sign bit or the exponent. 二 × p

Algorithms for addition and multiplication of expansions follow. The (rather lengthy) proofs of all theorems are omitted, but are available in a full-length version of this paper.

2 Note that this definition of expansion is slightly different from that used by Priest [12]; whereas Priest requires that the exponents of any two components of an expansion differ by at least , no such requirement is made here.

3 Formally, and are nonoverlapping if there exist integers and such that 2 and 2 , or 2 and 2 . S

Theorems 3 and 6 are the key new results.

## 3.2 Addition

An important basic operation in all the algorithms for performing arithmetic with expansions is the addition of two -bit values to form a nonoverlapping expansion (of length two). Two such algorithms follow.

Theorem 1 (Dekker [4]) Let and be -bit floating-point numbers such that . Then the following algorithm will produce a nonoverlapping expansion such that , where is an approximation to and represents the roundoff error in the calculation of .

F AST -T WO -S UM

1 c←ae ④b

2 virtual ea

3 virtual

4 return

Note that the outputs and do not necessarily have the same sign. Two-term subtraction ('F AST -T WO -D IFF ') is implemented by the sequence ; virtual ; virtual . 个 a

The difficulty with using F AST -T WO -S UM is the requirement that . If the relative sizes of and are unknown, a comparison is required to order the addends before invoking F AST -T WO -S UM . In practice, it is faster on most processors to use the following algorithm. b a

Theorem 2 (Knuth [10]) Let and be -bit floating-point numbers, where 3 . Then the following algorithm will produce a nonoverlapping expansion such that . P &gt; ■

|   T WO -S UM | T WO -S UM        |
|--------------|-------------------|
|            1 |                   |
|            2 | virtual           |
|            3 | virtual virtual   |
|            4 | roundoff virtual  |
|            5 | roundoff virtual  |
|            6 | roundoff roundoff |
|            7 | return            |

Two-term subtraction ('T WO -D IFF ') is implemented by the sequence ; virtual ; virtual virtual ; roundoff virtual ; roundoff virtual ; roundoff roundoff . 介

Havingestablished how to add two -bit values,I turn to the topic of how to add two arbitrary precision values expressed as expansions. The algorithm L INEAR -E XPANSION -S UM below sums two expansions in linear time.

Acomplicating characteristic of the algorithm is that there may be spurious zero components scattered throughout the output expansion, even if no zeros are present in the input expansions. For instance, given the input expansions 1111 0 1001 and 1100 0 1, in four-bit arithmetic the output expansion is 11100 0 0 0 0001. Interspersed zeros in input expansions do no harm except to slow down arithmetic, but this slowdown escalates quickly as expansions grow. It is important for L INEAR -E XPANSION -S UM and S CALE -E XPANSION to perform zero elimination , outputting a +

Figure 1: Operation of L INEAR -E XPANSION -S UM . The expansions and are illustrated with their most significant components on the left. maintains an approximate running total. The F AST -T WO -S UM operations in the bottom row exist to clip a high-order bit off each term, if necessary, before outputting it. 9 qi

<!-- image -->

component (and incrementing an array index) only if it is not zero. For simplicity, versions without zero elimination are presented here, but my implementations eliminate zeros.

Priest [12] presents a similar algorithm (for processors with arbitrary floating-point radix) that guarantees that the components of the output expansion overlap by at most one digit (i.e. one bit in binary arithmetic). An expensive renormalization step is required afterward to remove the overlap. Bycontrast, my algorithm always produces a nonoverlapping expansion, and no renormalization is needed. The algorithm takes advantage of binary arithmetic and exact rounding, but does not follow directly from Priest's results.

Theorem 3 Let 1 and 1 be nonoverlapping expansions of and -bit components, respectively, where 3 and the components of both and are ordered by increasing magnitude. The following algorithm will produce a nonoverlapping expansion such that 1 , where the components of are also in order of increasing magnitude, except that any of the may be zero. h= m+n hi hi ■

L INEAR -E XPANSION -S UM

- 1 Merge and into a single sequence ,

```
in order of nondecreasing magnitude 2 2 2 F AST -T WO -S UM 2 1 3 for 3 to 4 2 F AST -T WO -S UM 1 5 T WO -S UM 1 6 1 7 8 return hm+n ← Qm+n
```

is an approximate sum of the first terms of ; see Figure 1. It is possible to remove the F AST -T WO -S UM operation from the loop, yielding an algorithm that requires only six floating-point operations per iteration, but the preconditions for correct behavior are too complex to explain here. 2

## 3.3 Multiplication

The basic multiplication algorithm computes a nonoverlapping expansion equal to the product of two -bit values. The multiplication is performed by splitting each value into two halves with half the precision, then performing four exact multiplications on these fragments. The trick is to find a way to split a floating-point value in two.

Figure 2: Operation of S CALE -E XPANSION . The expansions and are illustrated with their most significant components on the left.

<!-- image -->

Theorem 4 (Dekker [4]) Let be a -bit floating-point number, where 3 . The following algorithm will produce a 2 -bit value hi and a nonoverlapping 2 1 -bit value such that and . a

- 5 return hi lo
- lo hi lo hi lo S PLIT 1 2 2 1 2 big 3 hi big 4 lo hi c个

The claim may seem absurd. After all, hi and lo have only 1 bits of significand between them; how can they carry all the information of a -bit significand? The secret is hidden in the sign bit of lo . For instance, the seven-bit number1001001can be split into the three-bit terms 1010000 and 111. This property is fortunate, because even if is odd, as it is in IEEE 754 double precision arithmetic, can be split into two 2 -bit values. p

Multiplication is performed by splitting and . The products hi hi , lo hi , hi lo , and lo lo can each be computed exactly by the floating-point unit, producing four values. By subtracting them from in a proper order, one is assured the subtractions are exact and the result is the roundoff error of computing . Dekker [4] attributes the following method to G. W. Veltkamp.

Theorem 5 (Veltkamp) Let and be -bit floating-point numbers, where 4 . The following algorithm will produce a nonoverlapping expansion such that .

|   T WO -P RODUCT | T WO -P RODUCT   |
|------------------|------------------|
|                1 |                  |
|                2 | hi lo S PLIT     |
|                3 | hi lo S PLIT     |
|                4 | 1 hi hi          |
|                5 | 2 1 lo hi        |
|                6 | 3 2 hi lo        |
|                7 | lo lo 3          |
|                8 | return           |

The following algorithm, which multiplies an expansion by a floating-point value, is new.

Theorem 6 Let 1 be an ordered nonoverlapping expansion of -bit components, and let be a -bit value where 4 . Then the following algorithm will produce a nonoverlapping expansion such that 2 1 , where is also ordered, except that any of the may be zero. (See Figure 2.) m hi = h

S CALE -E XPANSION

|   1 | 2 1 T WO -P RODUCT 1        |
|-----|-----------------------------|
|   2 | for 2 to                    |
|   3 | T WO -P RODUCT              |
|   4 | 2 1 2 2 T WO -S UM 2 2      |
|   5 | 2 2 1 F AST -T WO -S UM 2 1 |
|   6 | 2 2                         |
|   7 | return                      |

## 3.4 Approximation

The sign of an expansion can be identified by examining its largest component, but that component may be a poor approximation to the value of the whole expansion; it may carry as little as one bit of significance. Such a component may result, for instance, from cancellation during the subtraction of two nearly-equal expansions.

AnA PPROXIMATE procedureis defined that sums an expansion's components in order from smallest to largest. Because of the nonoverlapping property, A PPROXIMATE produces an approximation having error less than the magnitude of the least significant bit of the approximation's significand.

## 4 Adaptive Precision Arithmetic

Exact arithmetic is expensive, and should be avoided when possible. The floating-point filter suggested by Fortune and Van Wyk [5], which tries to verify the correctness of the approximate result (using error analysis) before resorting to exact arithmetic, is quite effective. If the exact test is only needed occasionally, an application can be made robust at only a small cost in speed. One might hope to improve this idea by computing a sequence of several increasingly accurate results, testing each one in turn for accuracy. Alas, whenever an exact result is required, one suffers both the cost of the exact computation and the additional burden of computing several approximate results in advance. Fortunately, it is sometimes possible to use intermediate results as stepping stones to more accurate results; work already done is not discarded but is refined.

## 4.1 Making Arithmetic Adaptive

F AST -T WO -S UM , T WO -S UM , and T WO -P RODUCT each have the feature that they can be broken into two parts: Line 1, which computes an approximate result, and the remaining lines, which calculate the roundoff error. The latter, more expensive calculation can be delayed until it is needed, if it is ever needed at all. In this sense, these routines can be made adaptive , so that they only produce as much of the result as is needed. I describe here how to achieve the same effect with more general expressions.

Any expression composed of addition, subtraction, and multiplication operations can be calculated adaptively in a mannerthat defines a natural sequence of intermediate results whose accuracy it is appropriate to test. Such a sequence is most easily described by considering the tree associated with the expression, as in Figure 3(a). The leaves of this tree represent floating-point operands, and its internal nodes represent operations. Replace each node whose children are both leaves with the sum , where represents the approximate value of the subexpression, and represents the roundofferror incurred while calculating (Figure 3(b)), then expand the expression to form a polynomial. Ci 十

Figure 3: (a) Formula for the square of the distance between two points and . (b) The lowest subexpressions in the tree are expressed as the sum of an approximate value and a roundoff error. (c) An incremental adaptive method for evaluating the expression. The approximations 1 through 4 are generated and tested in turn. 4 is exact. a

<!-- image -->

In the expanded expression, the terms containing many occurrences of variables are dominated by terms containing fewer occurrences. As an example, consider the expression 2 2 (Figure 3), which calculates the square of the distance between two points in the plane. Set 1 1 and 2 2 . The resulting expression, expanded in full, is

<!-- formula-not-decoded -->

It is significant that each is small relative to its corresponding . Exact rounding guarantees that , where 2 is called the machine epsilon , and bounds the relative error error of any basic floatingpoint operation. In IEEE 754 double precision arithmetic, 2 53 ; in single precision, 2 24 . lyil V b)/(ab) E

The expanded expression above can be divided into three parts, having magnitudes of 1 , , and 2 , respectively. Denote these parts Φ 0 , Φ 1 , and Φ 2 . More generally, for any expression expanded in this manner, let Φ be the sum of all products containing of the variables, so that Φ has magnitude .

One can obtain an approximation with error no larger than by exactly summing the first terms, Φ 0 through Φ 1 . The sequence 1 2 of increasingly accurate approximations can be computed incrementally; is the exact sum of 1 and Φ 1 . Members of this sequence are generated and tested until one is sufficiently accurate. O(e))

An improvement is based on the observation that one can obtain an approximation with error no larger than by adding (exactly) to 1 a correctional term that approximates Φ 1 with ordinary floating-point arithmetic, to form a new approximation , as illustrated in Figure 3(c). The correctional term reduces the error from 1 to , so is nearly as accurate as but takes much less work to compute. This scheme reuses the work done in computing members of , but does not reuse the (much cheaper) correctional terms. Note that 1 , the first value computed by this method, is an approximation to Φ 0 ; if 1 is sufficiently accurate, it is unnecessary to use any exact arithmetic techniques at all. This first test is identical to Fortune and Van Wyk's floating-point filter. O(e))

This method does more work during each stage of the computation than the first method, but typically terminates one stage earlier. Although the use of correctional terms is slower when the exact result must be computed, it can cause a surprising improvement in other cases; for instance, the robust Delaunay tetrahedralization of points arrayed in a tilted grid (see Section 5.4) takes twice as long if the estimate 2 is skipped in the orientation and incircle tests, because 2 is much more expensive to produce.

The decomposition of an expression into an adaptive sequence as described above could be automated by an LN-like expression compiler, but for the predicates described in the next section, I have done the decomposition and written the code manually. Note that these ideas are not exclusively applicable to the multiple-term approach to arbitrary precision arithmetic. They can work with multiple-digit formats as well, though the details differ.

## 5 Predicate Implementations

## 5.1 Orientation and Incircle

Let , , , and be four points in the plane whose coordinates are machine-precision floating-point numbers. Define a procedure O RIENT 2D that returns a positive value if the points , , and are arranged in counterclockwise order, a negative value if they are in clockwise order, and zero if they are collinear. A more common (but less symmetric) interpretation is that O RIENT 2D returns a positive value if lies to the left of the directed line ; for this purpose the orientation test is used by many geometric algorithms. C

Define also a procedure I N C IRCLE that returns a positive value if lies inside the oriented circle . By oriented circle , I mean the unique (and possibly degenerate) circle through , , and , with these points occurring in counterclockwise order about the circle. (If the points occur in clockwise order, I N C IRCLE will reverse the sign of its output, as if the circle's exterior were its interior.) I N C IRCLE returns zero if and only if all four points lie on a common circle.

These definitions extend to arbitrary dimensions. For instance, O RIENT 3D returns a positive value if lies below the oriented plane passing through , , and . By oriented plane , I mean that , , and appear in counterclockwise order when viewed from above the plane.

In any dimension, the orientation and incircle tests may be implemented as matrix determinants. For example:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

These formulae generalize to other dimensions in the obvious way. Expressions 1 and 2 can be shown to be equivalent by simple algebraic transformations, as can Expressions 3 and 4. These equivalences are unsurprising because one expects the results of any orientation or incircle test not to change if all the points undergo an identical translation in the plane. Expression 2, for instance, follows from Expression 1 by translating each point by .

For exact computation, the choice between Expressions 1 and 2, or between 3 and 4, is not straightforward. Expression 2 takes roughly 25% more time to compute in exact arithmetic, and Expression 4 takes about 30% more time than Expression 3. The disparity likely increases in higher dimensions. Nevertheless, the mechanics of error estimation turn the tide in the other direction. Important as a fast exact test is, it is equally important to avoid exact tests whenever possible. Expressions 2 and 4 tend to have smaller errors (and correspondingly smaller error estimates) because their errors are a function of the relative coordinates of the points, whereas the errors of Expressions 1 and 3 are a function of the absolute coordinates of the points.

In most geometric applications, the points that serve as parameters to geometric tests tend to be close to each other. Commonly, their absolute coordinates are much larger than the distances between them. By translating the points so they lie near the origin, working precision is freed for the subsequent calculations. Hence, the errors and error bounds for Expressions 2 and 4 are generally much smaller than for Expressions 1 and 3. Furthermore, the translation can often be done without roundoff error. Figure 4 demonstrates a toy problem: suppose O RIENT 2D is used to find the orientation of each triangle in a triangulation. A well-known property of floating-point arithmetic is that if two -bit floating-point values have the same sign and differ by at most a factor of two, their difference is expressible in bits. Hence, any shaded triangle can be translated so that one of its vertices lies at the origin without roundoff error; the white triangles may or may not suffer from roundoff during such translation. If the complete triangulation is much larger than the portion illustrated, only a small proportion of the triangles (those near a coordinate axis) can suffer roundoff. Because exact translation is the common case, my adaptive geometric predicates test for and exploit this case.

Figure 4: Shaded triangles can be translated to the origin without incurring roundoff error. In most triangulations, such triangles are the common case.

<!-- image -->

Once a determinant has been chosen for evaluation, there are several methods to evaluate it. A few are surveyed by Fortune and Van Wyk [5]; only their conclusion is repeated here. The cheapest method of evaluating the determinant of a 5 5 or smaller matrix seems to be dynamic programming applied to cofactor expansion. Evaluate the 2 determinants of all 2 2 minors of the first two columns, then the 3 determinants of all 3 3 minors of the first two columns, and so on. All four of my predicates use this method.

## 5.2 O RIENT 2D

My implementation of O RIENT 2D computes a sequence of up to four results (labeled A through D) as illustrated in Figure 5. The exact result D may be as long as sixteen components, but zero elimination is used, so a length of two to six components is more common in practice.

Figure 5: Adaptive calculations used by the 2D orientation test. Dashed boxes represent nodes in the original expression tree.

<!-- image -->

A, B, and C are logical places to test the accuracy of the result before continuing. In most applications, the majority of calls to O RIENT 2D will end with the approximation A. Although the four-component expansion B, like A, has an error of , it is a likely stopping point because B is exact if the four subtractions at the bottom of the expression tree are performed without roundoff error (corresponding to the shaded triangles in Figure 4). Because this is the common case, O RIENT 2D explicitly tests whether all the roundoff terms are zero. The corrected estimate C has an error bound of 2 . If C is not sufficiently accurate, the exact determinant D is computed.

There are two interesting features of this test, both of which arise because only the sign of the determinant is needed. First, the correctional term added to B to form C is not added exactly; instead, the A PPROXIMATE procedure of Section 3.4 finds an approximation B of B, and the correctional term is added to B with the possibility of roundoff error. The consequent errors may be of magnitude B , and would normally preclude obtaining an error bound of 2 . However, the sign of the determinant is only questionable if B is of magnitude , so an 2 error bound for C can be established.

Thesecond interesting feature is that,if C is not sufficiently accurate, no more approximations are computed before computing the exact determinant. To understand why, consider three collinear points; the determinant defined by these points is zero. If a coordinate of one of these points is perturbed by the least significant bit of its significand, the determinant typically increases to . Hence, one might guess that when a determinant is no larger than 2 , it is probably zero. This intuition seems to hold in practice for all four predicates considered herein, on both random and 'practical' point sets. Determinants that don't stop with approximation C are nearly always zero.

Table 1: Timings for O RIENT 2D on a DEC 3000/700 with a 225 MHz Alpha processor. All determinants use the 2D version of either Expression 1 or the more stable Expression 2 as indicated. Timings for the adaptive tests are categorized according to which result was the last generated. Timings of Bailey's MPFUN package and Fortune and Van Wyk's LN package are included for comparison.

| Double precision O RIENT 2D timings in microseconds   | Double precision O RIENT 2D timings in microseconds   | Double precision O RIENT 2D timings in microseconds   | Double precision O RIENT 2D timings in microseconds   |
|-------------------------------------------------------|-------------------------------------------------------|-------------------------------------------------------|-------------------------------------------------------|
| Points Method                                         | Uniform Random                                        | Geometric Random                                      | Nearly Collinear                                      |
| Approximate (2)                                       | 0.15                                                  | 0.15                                                  | 0.16                                                  |
| Exact (1)                                             | 6.56                                                  | 6.89                                                  | 6.31                                                  |
| Exact (2)                                             | 8.35                                                  | 8.48                                                  | 8.13                                                  |
| Exact (1), MPFUN                                      | 92.85                                                 | 94.03                                                 | 84.97                                                 |
| Adaptive A (2), approx.                               | 0.28                                                  | 0.27                                                  | 0.22                                                  |
| Adaptive B (2)                                        |                                                       |                                                       | 1.89                                                  |
| Adaptive C (2)                                        |                                                       |                                                       | 2.14                                                  |
| Adaptive D (2), exact                                 |                                                       |                                                       | 8.35                                                  |
| LN adaptive (2), approx. LN adaptive (2), exact       | 0.32                                                  | n/a n/a                                               | 4.43                                                  |

The error bound for A is 3 16 2 1 2 , where 1 and 2 are as indicated in Figure 5. This error bound has the pleasing property that it is zero in the common case that all three input points lie on a horizontal or vertical line. Hence, although O RIENT 2D usually resorts to exact arithmetic when given collinear input points, it only performs the approximate test in the two cases that occur most commonly in practice.

Table 1 lists timings for O RIENT 2D, given random inputs. Observe that the adaptive test, when it stops at the approximate result A, takes nearly twice as long as the approximate test because of the need to compute an error bound. The table includes a comparison with Bailey's MPFUN [2], chosen because it is the fastest portable and freely available arbitrary precision package I know of. O RIENT 2D coded with my (nonadaptive) algorithms is roughly thirteen times faster than O RIENT 2D coded with MPFUN.

Also included is a comparison with an orientation predicate for 53-bit integer inputs, created by Fortune and Van Wyk's LN. The LN-generated orientation predicate is quite fast because it takes advantage of the fact that it is restricted to bounded integer inputs. My exact tests cost less than twice as much as LN's; this seems like a reasonable price to pay for the ability to handle arbitrary exponents in the input.

These timings are not the whole story; LN's static error estimate is typically much larger than the runtime error estimate used for adaptive stage A, and LN uses only two stages of adaptivity, so the LN-generated predicates are slower in some applications, as Section 5.4 will demonstrate. (It is significant that for 53-bit integer inputs, my multiple-stage predicates rarely pass stage B because the initial translation is usually done without roundoff error;hence, the LN-generated O RIENT 2Doften takes more than twice as long to produce an exact result.) It must be emphasized, however, that these are not inherent differences between LN's multiple-digit integer approach and my multiple-term floating-point approach; LN could, in principle, employ the same runtime error estimate and a similar multiple-stage adaptivity scheme.

Figure 6: Adaptive calculations used by the 3D orientation test. Bold numbers indicate the length of an expansion. Only part of the expression tree is shown; two of the three cofactors are omitted, but their results appear as dashed components and expansions.

<!-- image -->

## 5.3 O RIENT 3D , I N C IRCLE , and I N S PHERE

Figure 6 illustrates the implementation of O RIENT 3D. As with O RIENT 2D, A is the standard floating-point result, B is exact if the subtractions at the bottom of the tree incur no roundoff, C represents a drop in the error bound from to 2 , and D is the exact determinant. The error bounds are zero if all four input points share the same , , or -coordinate, so only the approximate test is needed in the most common cases of coplanarity. ()0

Table 2 lists timings for O RIENT 3D, I N C IRCLE , and I N -S PHERE , given random inputs. In each case, the error bound for A increases the amount of time required to perform the approximate test in the adaptive case by a factor of 2 to 2 5. Thegapbetweenmyexactalgorithmsand MPFUN is smaller than in the case of O RIENT 2D, but is still a factor of 3 to 7 5.

I N C IRCLE is implemented similarly to O RIENT 3D, as the determinants are similar. The implementation of I N S PHERE differs from the other three tests in that, due to programmer laziness, D is not computed incrementally from B; rather, if C is not accurate enough, D is computed from scratch.

Table 2: Timings for O RIENT 3D , I N C IRCLE , and I N S PHERE on a DEC 3000/700. All determinants are Expression 1 or 3, or the more stable Expression 2 or 4, as indicated.

| Double precision O RIENT 3D timings in microseconds   | Double precision O RIENT 3D timings in microseconds   | Double precision O RIENT 3D timings in microseconds   | Double precision O RIENT 3D timings in microseconds   |
|-------------------------------------------------------|-------------------------------------------------------|-------------------------------------------------------|-------------------------------------------------------|
| Points Method                                         | Uniform Random                                        | Geometric Random                                      | Nearly Coplanar                                       |
| Approximate (2)                                       | 0.25                                                  | 0.25                                                  | 0.25                                                  |
| Exact (1)                                             | 33.30                                                 | 38.54                                                 | 32.90                                                 |
| Exact (2)                                             | 42.69                                                 | 48.21                                                 | 42.41                                                 |
| Exact (1), MPFUN                                      | 260.51                                                | 262.08                                                | 246.64                                                |
| Adaptive A (2), approx.                               | 0.61                                                  | 0.60                                                  | 0.62                                                  |
| Adaptive B (2)                                        |                                                       |                                                       | 12.98                                                 |
| Adaptive C (2)                                        |                                                       |                                                       | 15.59                                                 |
| Adaptive D (2), exact                                 |                                                       |                                                       | 27.29                                                 |
| LN adaptive (2), approx.                              | 0.85                                                  | n/a                                                   |                                                       |
| LN adaptive (2), exact                                |                                                       | n/a                                                   | 18.11                                                 |
| Double precision I N C IRCLE timings in microseconds  | Double precision I N C IRCLE timings in microseconds  | Double precision I N C IRCLE timings in microseconds  | Double precision I N C IRCLE timings in microseconds  |
| Points Method                                         | Uniform Random                                        | Geometric Random                                      | Nearly Cocirc.                                        |
| Approximate (4)                                       | 0.31                                                  | 0.28                                                  | 0.30                                                  |
| Exact (3)                                             | 71.66                                                 | 83.01                                                 | 75.34                                                 |
| Exact (4)                                             | 91.71                                                 | 118.30                                                | 104.44                                                |
| Exact (3), MPFUN                                      | 350.77                                                | 343.61                                                | 348.55                                                |
| Adaptive A (4), approx.                               | 0.64                                                  | 0.59                                                  | 0.64                                                  |
| Adaptive B (4)                                        |                                                       |                                                       | 44.56                                                 |
| Adaptive C (4)                                        |                                                       |                                                       | 48.80                                                 |
| Adaptive D (4), exact                                 |                                                       |                                                       | 78.06                                                 |
| LN adaptive (4), approx.                              | 1.33                                                  | n/a                                                   |                                                       |
| LN adaptive (4), exact                                |                                                       | n/a                                                   | 32.44                                                 |
| Double precision I N S PHERE timings in microseconds  | Double precision I N S PHERE timings in microseconds  | Double precision I N S PHERE timings in microseconds  | Double precision I N S PHERE timings in microseconds  |
| Points Method                                         | Uniform Random                                        | Geometric Random                                      | Nearly Cospher.                                       |
| Approximate (4)                                       | 0.93                                                  | 0.95                                                  | 0.93                                                  |
| Exact (3)                                             | 324.22                                                | 378.94                                                | 347.16                                                |
| Exact (4)                                             | 374.59                                                | 480.28                                                | 414.13                                                |
| Exact (3), MPFUN                                      | 1,017.56                                              | 1,019.89                                              | 1,059.87                                              |
| Adaptive A (4), approx.                               | 2.13                                                  | 2.14                                                  | 2.14                                                  |
| Adaptive B (4)                                        |                                                       |                                                       | 166.21                                                |
| Adaptive C (4)                                        |                                                       |                                                       | 171.74                                                |
| Adaptive D (4), exact                                 |                                                       |                                                       | 463.96                                                |
| LN adaptive (4), approx.                              | 2.35                                                  | n/a                                                   | 116.74                                                |
| LN adaptive (4), exact                                |                                                       | n/a                                                   |                                                       |

Fortunately, C is usually accurate enough.

## 5.4 Triangulation

To evaluate the effectiveness of the adaptive tests in applications, I tested them in two of my Delaunay triangulation codes. Triangle [14] is a 2D Delaunay triangulator and mesh generator, publicly available from Netlib, that uses a divideand-conquer algorithm [11, 8]. Pyramid is a 3D Delaunay tetrahedralizer that uses an incremental algorithm [15]. For both 2D and 3D, three types of inputs were tested: uniform random points, points lying (approximately) on the boundary of a circle or sphere, and a square or cubic grid of lattice points, tilted so as not to be aligned with the coordinate axes. The latter two were chosen for their nastiness. The lattices have been tilted using approximate arithmetic, so they are not perfectly cubical, and the exponents of their coordinates vary enough that LN cannot be used. (I could have used perfect lattices with 53-bit integer coordinates, but O RIENT 3D and I N S PHERE would never pass stage B; the perturbed lattices occasionally force the predicates into stage C or D.)

Table 3: Statistics for 2D divide-and-conquer Delaunay triangulation of several point sets.

| 2D divide-and-conquer Delaunay triangulation                | 2D divide-and-conquer Delaunay triangulation   | 2D divide-and-conquer Delaunay triangulation   | 2D divide-and-conquer Delaunay triangulation   |
|-------------------------------------------------------------|------------------------------------------------|------------------------------------------------|------------------------------------------------|
|                                                             | Uniform Random                                 | Perimeter of Circle                            | Tilted Grid                                    |
| Input points                                                | 1,000,000                                      | 1,000,000                                      | 1,000,000                                      |
| O RIENT 2D calls                                            |                                                |                                                |                                                |
| Adaptive A, approx. Adaptive B Adaptive C Adaptive D, exact | 9,497,314                                      | 6,291,742                                      | 9,318,610 121,081 118 3                        |
| Average time, s                                             | 0.32                                           | 0.38                                           | 0.33                                           |
| LN approximate LN exact                                     | 9,497,314                                      | 2,112,284 4,179,458                            | n/a n/a                                        |
| LN average time, s                                          | 0.35                                           | 3.16                                           | n/a                                            |
| I N C IRCLE calls                                           |                                                |                                                |                                                |
| Adaptive A, approx. Adaptive B Adaptive C Adaptive D, exact | 7,596,885                                      | 3,970,796 50,551 120                           | 7,201,317 176,470 47 4                         |
| Average time, s                                             | 0.65                                           | 1.11                                           | 1.67                                           |
| LN approximate LN exact LN average time, s                  | 6,077,062 1,519,823 7.36                       | 0 4,021,467 32.78                              | n/a n/a n/a                                    |
| Program running time,                                       | seconds                                        |                                                |                                                |
| Approximate version Robust version LN robust version        | 57.3 61.7 116.0                                | 59.9 64.7 214.6                                | 48.3 62.2 n/a                                  |

Table 4: Statistics for 3D incremental Delaunay tetrahedralization of several point sets. The approximate code failed to terminate on the tilted grid input.

| 3D incremental Delaunay tetrahedralization                  | 3D incremental Delaunay tetrahedralization   | 3D incremental Delaunay tetrahedralization   | 3D incremental Delaunay tetrahedralization   |
|-------------------------------------------------------------|----------------------------------------------|----------------------------------------------|----------------------------------------------|
|                                                             | Uniform Random                               | Surface of Sphere                            | Tilted Grid                                  |
| Input points                                                | 10,000                                       | 10,000                                       | 10,000                                       |
| O RIENT 3D counts                                           |                                              |                                              |                                              |
| Adaptive A, approx. Adaptive B Adaptive C                   | 2,735,668                                    | 1,935,978                                    | 5,542,567 602,344 1,267,423                  |
| Average time, s                                             | 0.72                                         | 0.72                                         | 4.12                                         |
| LN approximate LN exact                                     | 2,735,668                                    | 1,935,920 58                                 | n/a n/a                                      |
| LN average time, s                                          | 0.99                                         | 1.00                                         | n/a                                          |
| I N S PHERE counts                                          |                                              |                                              |                                              |
| Adaptive A, approx. Adaptive B Adaptive C Adaptive D, exact | 439,090                                      | 122,273 180,383 1,667                        | 3,080,312 267,162 548,063                    |
| Average time, s                                             | 2.23                                         | 96.45                                        | 48.12                                        |
| LN approximate LN exact                                     | 438,194 896                                  | 104,616 199,707                              | n/a n/a                                      |
| LN average time, s                                          | 2.50                                         | 70.82                                        | n/a                                          |
| Program running time, seconds                               | Program running time, seconds                | Program running time, seconds                | Program running time, seconds                |
| Approximate version                                         | 4.3                                          | 3.0                                          |                                              |
| Robust version                                              | 5.8                                          | 34.1                                         | 108.5                                        |
| LN robust version                                           | 6.5                                          | 30.5                                         | n/a                                          |

The results for 2D, outlined in Table 3, indicate that the four-stage predicates add about 8% to the total running time for randomly distributed input points, mainly because of the error bound tests. For the more difficult point sets, the penalty may be as great as 30%. Of course, this penalty applies to precisely the point sets that are most likely to cause difficulties when exact arithmetic is not available.

The results for 3D, outlined in Table 4, are somewhat less pleasing. The four-stage predicates add about 35% to the total running time for randomly distributed input points; for points distributed approximately on the surface of a sphere, the penalty is a factor of eleven. Ominously, however, the penalty for the tilted grid is uncertain, because the tetrahedralization program using approximate arithmetic failed to terminate. A debugger revealed that the point location routine was stuck in an infinite loop because a geometric inconsistency had been introduced into the mesh due to roundoff error. Robust arithmetic is not always slower after all.

## 6 Conclusions

As Priest points out, multiple-term algorithms can be used to implement extended (but finite) precision arithmetic as well as exact arithmetic; simply compress and then truncate each result to a fixed number of components. Perhaps the greatest potential of these algorithms is in providing a fast and simple way to extend slightly the precision of critical variables in numerical algorithms. Hence, it would not be difficult to provide a routine that quickly computes the intersection point of two segments with double precision endpoints, correctly rounded to a double precision result. Speed considerations may make it untenable to accomplish this by calling a standard extended precision library. The techniques Priest and I have developed are simple enough to be coded directly in numerical algorithms, avoiding function call overhead and conversion costs.

A useful tool in coding such algorithms would be an expression compiler similar to Fortune and Van Wyk's [5], which converts an expression into exact arithmetic code, complete with error bound derivation and floating-point filters. Such a tool might even be able to automate the process of breaking an expression into adaptive stages as described in Section 4.

It might be fruitful to explore whether the methods described by Clarkson [3] and Avnaim et al. [1] can be extended by fast multiprecision methods to handle arbitrary double precision floating-point inputs. One could certainly relax their constraints on the bit complexity of the inputs; for instance, the method of Avnaim et al. could be made to perform the I N S PHERE test on 64-bit inputs using expansions of length three. Unfortunately, it is not obvious how to adapt these integer-based techniques to inputs with wildly differing exponents. It is also not clear whether such hybrid algorithms would be faster than straightforward adaptiv- ity. Nevertheless, Clarkson's approach looks promising for larger determinants. Although my methods work well for small determinants, they are unlikely to work well for sizes much larger than 5 5. Even if one uses Gaussian elimination rather than cofactor expansion (an important adjustment for larger matrices [5, 9]), the adaptivity technique does not scale well with determinants, because of the large number of terms in the expanded polynomial. Clarkson's technique maybetheonlyeconomicalapproachfor matriceslarger than 10 10.

Whether or not these issues are resolved in the near future, researchers can make use today of tests for orientation and incircle in two and three dimensions that are correct, fast, and immediately applicable to double precision floating-point inputs. I invite working computational geometers to try my code in their implementations, and I hope that it will save them from worrying about robustness so they may concentrate on geometry.

## References

- [1] Francis Avnaim, Jean-Daniel Boissonnat, Olivier Devillers, Franco P. Preparata, and Mariette Yvinec. Evaluating Signs of Determinants Using Single-Precision Arithmetic . 1995.
- [2] David H. Bailey. A Portable High Performance Multiprecision Package . Technical Report RNR-90-022, NASA Ames Research Center, May 1993.
- [3] Kenneth L. Clarkson. Safe and Effective Determinant Evaluation . 33rd Annual Symposium on Foundations of Computer Science, pages 387-395, 1992.
- [4] T. J. Dekker. A Floating-Point Technique for Extending the Available Precision . Numerische Mathematik 18 :224-242, 1971.
- [5] Steven Fortune and Christopher J. Van Wyk. Efficient Exact Arithmetic for Computational Geometry . Ninth Annual Symposium on Computational Geometry, pages 163-172, May 1993.
- [6] . Static Analysis Yields Efficient Exact Integer Arithmetic for Computational Geometry . To appear in Transactions on Mathematical Software, 1996.
- [7] David Goldberg. What Every Computer Scientist Should Know About Floating-Point Arithmetic . ACM Computing Surveys 23 (1):5-48, March 1991.
- [8] Leonidas J. Guibas and Jorge Stolfi. Primitives for the Manipulation of General Subdivisions and the Computation of Voronoi Diagrams . ACM Transactions on Graphics 4 (2):74-123, April 1985.
- [9] Michael Karasick, Derek Lieber, and Lee R. Nackman. Efficient Delaunay Triangulation Using Rational Arithmetic . ACM Transactions on Graphics 10 (1):71-91, January 1991.
- [10] Donald Ervin Knuth. The Art of Computer Programming: Seminumerical Algorithms , second edition, volume 2. Addison Wesley, 1981.
- [11] D. T. Lee and B. J. Schachter. Two Algorithms for Constructing a Delaunay Triangulation . Int. J. Comput. Inf. Sci. 9 :219-242, 1980.
- [12] Douglas M. Priest. Algorithms for Arbitrary Precision Floating Point Arithmetic . Tenth Symposium on Computer Arithmetic, pages 132143, 1991.
- [13] . On Properties of Floating Point Arithmetics: Numerical Stability and the Cost of Accurate Computations . Ph.D. thesis, Department of Mathematics, University of California at Berkeley, November 1992.
- [14] Jonathan Richard Shewchuk. Triangle: Engineering a 2D Quality Mesh Generator and Delaunay Triangulator . First Workshop on Applied Computational Geometry. Association for Computing Machinery, May 1996.
- [15] David F. Watson. Computingthe -dimensional Delaunay Tessellation with Application to Voronoi Polytopes . Computer Journal 24 :167-172, 1981.