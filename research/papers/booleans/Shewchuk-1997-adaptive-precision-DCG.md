## Adaptive Precision Floating-Point Arithmetic and Fast Robust Geometric Predicates

Jonathan Richard Shewchuk October 1, 1997 CMU-CS-96-140R

From Discrete &amp; Computational Geometry 18 (3):305-363, October 1997.

School of Computer Science Carnegie Mellon University Pittsburgh, PA 15213

## Abstract

Exact computer arithmetic has a variety of uses, including the robust implementation of geometric algorithms. This article has three purposes. The first is to offer fast software-level algorithms for exact addition and multiplication of arbitrary precision floating-point values. The second is to propose a technique for adaptive precision arithmetic that can often speed these algorithms when one wishes to perform multiprecision calculations that do not always require exact arithmetic, but must satisfy some error bound. The third is to use these techniques to develop implementations of several common geometric calculations whose required degree of accuracy depends on their inputs. These robust geometric predicates are adaptive; their running time depends on the degree of uncertainty of the result, and is usually small.

These algorithms work on computers whose floating-point arithmetic uses radix two and exact rounding, including machines complying with the IEEE 754 standard. The inputs to the predicates may be arbitrary single or double precision floating-point numbers. C code is publicly available for the 2D and 3D orientation and incircle tests, and robust Delaunay triangulation using these tests. Timings of the implementations demonstrate their effectiveness.

Supported in part by the Natural Sciences and Engineering Research Council of Canada under a 1967 Science and Engineering Scholarship and by the National Science Foundation under Grant CMS-9318163. The views and conclusions contained in this document are those of the author and should not be interpreted as representing the official policies, either express or implied, of NSERC, NSF, or the U.S. Government.

Keywords: arbitrary precision floating-point arithmetic, computational geometry, geometric robustness, orientation test, incircle test, Delaunay triangulation

## Contents

| 1 Introduction                                                     | 1 Introduction                                                     | 1 Introduction                                                     |   1 |
|--------------------------------------------------------------------|--------------------------------------------------------------------|--------------------------------------------------------------------|-----|
| 2 Arbitrary Precision Floating-Point Arithmetic                    | 2 Arbitrary Precision Floating-Point Arithmetic                    | 2 Arbitrary Precision Floating-Point Arithmetic                    |   3 |
|                                                                    | 2.1                                                                | Background . . . . . . . . . . . . . . . . . . . .                 |   3 |
|                                                                    | 2.2                                                                | Properties of Binary Arithmetic . . . . . . . . . .                |   5 |
|                                                                    | 2.3                                                                | Simple Addition . . . . . . . . . . . . . . . . . .                |   6 |
|                                                                    | 2.4                                                                | Expansion Addition . . . . . . . . . . . . . . . .                 |   9 |
|                                                                    | 2.5                                                                | Simple Multiplication . . . . . . . . . . . . . . .                |  18 |
|                                                                    | 2.6                                                                | Expansion Scaling . . . . . . . . . . . . . . . .                  |  21 |
|                                                                    | 2.7                                                                | Compression and Approximation . . . . . . . . .                    |  24 |
|                                                                    | 2.8                                                                | Other Operations . . . . . . . . . . . . . . . . .                 |  26 |
| 3 Adaptive Precision Arithmetic                                    | 3 Adaptive Precision Arithmetic                                    | 3 Adaptive Precision Arithmetic                                    |  27 |
|                                                                    | 3.1                                                                | Why Adaptivity? . . . . . . . . . . . . . . . . .                  |  27 |
|                                                                    | 3.2                                                                | Making Arithmetic Adaptive . . . . . . . . . . .                   |  28 |
| 4 Implementation of Geometric Predicates                           | 4 Implementation of Geometric Predicates                           | 4 Implementation of Geometric Predicates                           |  31 |
|                                                                    | 4.1                                                                | Related Work in Robust Computational Geometry                      |  31 |
|                                                                    | 4.2                                                                | The Orientation and Incircle Tests . . . . . . . .                 |  36 |
|                                                                    | 4.3                                                                | O RIENT 2D . . . . . . . . . . . . . . . . . . . .                 |  38 |
|                                                                    | 4.4                                                                | O RIENT 3D, I N C IRCLE , and I N S PHERE . . . . .                |  42 |
|                                                                    | 4.5                                                                | Performance in Two Triangulation Programs . . .                    |  45 |
| 5 Caveats                                                          | 5 Caveats                                                          | 5 Caveats                                                          |  46 |
| 6 Conclusions                                                      | 6 Conclusions                                                      | 6 Conclusions                                                      |  49 |
| A Linear-Time Expansion Addition without Round-to-Even Tiebreaking | A Linear-Time Expansion Addition without Round-to-Even Tiebreaking | A Linear-Time Expansion Addition without Round-to-Even Tiebreaking |  51 |
| B Why the Tiebreaking Rule is Important                            | B Why the Tiebreaking Rule is Important                            | B Why the Tiebreaking Rule is Important                            |  52 |

## About this Report

a0

An electronic copy of this report, and the software described herein, can be obtained through the Web page http://www.cs.cmu.edu/ quake/robust.html.

Copyright 1998 by Jonathan Richard Shewchuk. This report may be freely duplicated and distributed so long as this copyright notice remains intact. Please mail comments and corrections to me at jrs@cs.cmu.edu.

Many thanks to Steven Fortune, Douglas Priest, and Christopher Van Wyk, who each provided comments on a draft of this paper, and whose papers provided the foundations for this research. Steven Fortune also provided LN-generated predicates for timing comparisons, and unwittingly sparked this research in mid1994 with a few brief email responses. Thanks also to David O'Hallaron, James Stichnoth, and Daniel Tunkelang for their comments.

## 1 Introduction

Software libraries for arbitrary precision floating-point arithmetic can be used to accurately perform many error-prone or ill-conditioned computations that would be infeasible using only hardware-supported approximate arithmetic. Some of these computations have accuracy requirements that vary with their input. For instance, consider the problem of finding the center of a circle, given three points that lie on the circle. Normally, hardware precision arithmetic will suffice, but if the input points are nearly collinear, the problem is ill-conditioned and the approximate calculation may yield a wildly inaccurate result or a division by zero. Alternatively, an exact arithmetic library can be used and will yield a correct result, but exact arithmetic is slow; one would rather use it only when one really needs to.

This article presents two techniques for writing fast implementations of extended precision calculations like these, and demonstrates them with implementations of four commonly used geometric predicates. The first technique is a suite of algorithms, several of them new, for performing arbitrary precision arithmetic. The method has its greatest advantage in computations that process values of extended but small precision (several hundred or thousand bits), and seems ideal for computational geometry and some numerical methods, where much benefit can be realized from a modest increase in precision. The second technique is a way to modify these algorithms so that they compute their result adaptively; they are quick in most circumstances, but are still slow when their results are prone to have high relative error. A third subject of this article is a demonstration of these techniques with implementations and performance measurements of four commonly used geometric predicates. An elaboration of each of these three topics follows.

Methods of simulating exact arithmetic in software can be classified by several characteristics. Some exact arithmetic libraries operate on integers or fixed-point numbers, while others operate on floating-point numbers. To represent a number, the former libraries store a significand of arbitrary length; the latter store an exponent as well. Some libraries use the hardware's integer arithmetic units, whereas others use the floating-point units. Oddly, the decision to use integers or floating-point numbers internally is orthogonal to the type of number being represented. It was once the norm to use integer arithmetic to build extended precision floating-point libraries, especially when floating-point hardware was uncommon and differed between computer models. Times have changed, and modern architectures are highly optimized for floating-point performance; on many processors, floating-point arithmetic is faster than integer arithmetic. The trend is reversing for software libraries as well, and there are several proposals to use floating-point arithmetic to perform extended-precision integer calculations. Fortune and Van Wyk [12, 11], Clarkson [6], and Avnaim, Boissonnat, Devillers, Preparata, and Yvinec [1] have described algorithms of this kind, designed to attack the same computational geometry robustness problems considered later in this article. These algorithms are surveyed in Section 4.1.

Another differentiating feature of multiprecision libraries is whether they use multiple exponents. Most arbitrary precision libraries store numbers in a multiple-digit format, consisting of a sequence of digits (usually of large radix, like a0✂✁☎✄ ) coupled with a single exponent. A freely available example of the multiple-digit approach is Bailey's MPFUN package [2], a sophisticated portable multiprecision library that uses digits of machine-dependent radix (usually a0✂✄✝✆ ) stored as single precision floating-point values. An alternative is the multiple-component format, wherein a number is expressed as a sum of ordinary floating-point words, each with its own significand and exponent [23, 7, 19]. This approach has the advantage that the result of an addition like a0✂✁☎✞☎✞✠✟✡a0☞☛✌✁☎✞☎✞ (which may well arise in calculations like the geometric predicates discussed in Section 4.2) can be stored in two words of memory, whereas the multiple-digit approach will use at least 601 bits to store the sum, and incur a corresponding speed penalty when performing arithmetic with it. On the other hand, the multiple-digit approach can more compactly represent most numbers, because only one exponent is stored. (MPFUN sacrifices this compactness to take advantage of floating-point hardware; the exponent of each digit is unused.) More pertinent is the difference in speed, discussed briefly in Section 2.1.

The algorithms described herein use floating-point hardware to perform extended precision floatingpoint arithmetic, using the multiple-component approach. These algorithms, described in Section 2, work under the assumption that hardware arithmetic is performed in radix two with exact rounding. This assumption holds on processors compliant with the IEEE 754 floating-point standard. Proofs of the correctness of all algorithms are given.

The methods herein are closely related to, and occasionally taken directly from, methods developed by Priest [23, 24], but are faster. The improvement in speed arises partly because Priest's algorithms run on a wide variety of floating-point architectures, with different radices and rounding behavior, whereas mine are limited to and optimized for radix two with exact rounding. This specialization is justified by the wide acceptance of the IEEE 754 standard. My algorithms also benefit from a relaxation of Priest's normalization requirement, which is less strict than the normalization required by multiple-digit algorithms, but is nonetheless time-consuming to enforce.

I demonstrate these methods with publicly available code that performs the two-dimensional and threedimensional orientation and incircle tests, calculations that commonly arise in computational geometry. The orientation test determines whether a point lies to the left of, to the right of, or on a line or plane; it is an important predicate used in many (perhaps most) geometric algorithms. The incircle test determines whether a point lies inside, outside, or on a circle or sphere, and is used for Delaunay triangulation [14]. Inexact versions of these tests are vulnerable to roundoff error, and the wrong answers they produce can cause geometric algorithms to hang, crash, or produce incorrect output. Although exact arithmetic banishes these difficulties, it is common to hear reports of implementations being slowed by factors of ten or more as a consequence [16, 11]. For these reasons, computational geometry is an important arena for evaluating extended precision arithmetic schemes.

The orientation and incircle tests evaluate the sign of a matrix determinant. It is significant that only the sign, and not the magnitude, of the determinant is needed. Fortune and Van Wyk [11] take advantage of this fact by using a floating-point filter: the determinant is first evaluated approximately, and only if forward error analysis indicates that the sign of the approximate result cannot be trusted does one use an exact test. I carry their suggestion to its logical extreme by computing a sequence of successively more accurate approximations to the determinant, stopping only when the accuracy of the sign is assured. To reduce computation time, approximations reuse a previous, less accurate computation when it is economical to do so. Procedures thus designed are adaptive; they refine their results until they are certain of the correctness of their answer. The technique is not limited to computational geometry, nor is it limited to finding signs of expressions; it can be employed in any calculation where the required degree of accuracy varies. This adaptive approach is described in Section 3, and its application to the orientation and incircle tests is described in Section 4.

Readers who wish to use these predicates in their own applications are encouraged to download them and try them out. However, be certain to read Section 5, which covers two important issues that must be considered to ensure the correctness of the implementation: your processor's floating-point behavior and your compiler's optimization behavior. Furthermore, be aware that exact arithmetic is not a panacea for all robustness woes; its uses and limitations are discussed in Section 4.1. Exact arithmetic can make robust many algorithms that take geometric input and return purely combinatorial output; for instance, a fully robust convex hull implementation can be produced with recourse only to an exact orientation test. However, in algorithms that construct new geometric objects, exact arithmetic is sometimes constrained by its cost and its inability to represent arbitrary irrational numbers.

A few words are appropriate to describe some of the motivation for pursuing robust predicates for floating-point, rather than integer, operands. One might argue that real-valued input to a geometric pro- gram can be scaled and approximated in integer form. Indeed, there are few geometric problems that truly require the range of magnitude that floating-point storage provides, and integer formats had a clear speed advantage over floating-point formats for small-scale exact computation prior to the present research. The best argument for exact floating-point libraries in computational geometry, besides convenience, is the fact that many existing geometric programs already use floating-point numbers internally, and it is easier to replace their geometric predicates with robust floating-point versions than to retrofit the programs to use integers throughout. Online algorithms present another argument, because they are not always compatible with the scaled-input approach. One cannot always know in advance what resolution will be required, and repeated rescalings may be necessary to support an internal integer format when the inputs are real and unpredictable. In any case, I hope that this research will make it easier for programmers to choose between integer and floating-point arithmetic as they prefer.

## 2 Arbitrary Precision Floating-Point Arithmetic

## 2.1 Background

✂

✗

Most modern processors support floating-point numbers of the form significand exponent . The significand is a -bit binary number of the form ✄✆☎✝✄✞✄✞✄✟☎✠☎✠☎ , where each denotes a single bit; one additional bit represents the sign. This article does not address issues of overflow and underflow, so I allow the exponent to be an integer in the range ✡☞☛✍✌✏✎✑✌✓✒ . (Fortunately, many applications have inputs whose exponents fall within a circumscribed range. The four predicates implemented for this article will not overflow nor underflow if their inputs have exponents in the range ✡☞☛✕✔✞✖ ✘✗ ✔✙✒ and IEEE 754 double precision arithmetic is used.) Floating-point values are generally normalized , which means that if a value is not zero, then its most significant bit is set to one, and the exponent adjusted accordingly. For example, in four-bit arithmetic, binary ✔✚✔ (decimal ✔✜✛ ) is represented as ✔✘☎✢✔ ✔✣✁ a0✂✁ . See the survey by Goldberg [13] for a detailed explanation of floating-point storage formats, particularly the IEEE 754 standard.

✂

✟

✟

✥

✄

✟

✥

✔

✗

Exact arithmetic often produces values that require more than bits to store. For the algorithms herein, each arbitrary precision value is expressed as an expansion ✤✦✥★✧✏✥✪✩ ✬✫✠✫✠✫ , where each ✥✪✭ is called a component of and is represented by a floating-point value with a -bit significand. To impose some structure on expansions, they are required to be nonoverlapping and ordered by magnitude ( ✥✮✩ largest, smallest). Two floating-point values and are nonoverlapping if the least significant nonzero bit of is more significant than the most significant nonzero bit of , or vice versa; for instance, the binary values ✔✚✔ ✗✚✗ and ☛✕✔ ☎✢✔ are nonoverlapping, whereas and overlap. The number zero does not overlap any number. An expansion is nonoverlapping if all its components are mutually nonoverlapping. Note that a number may be represented by many possible nonoverlapping expansions; consider ✔✚✔ ✗✚✗ ☛✕✔ ☎✢✔✓✧ ✗✚✗ ✰✗ ☎✢✔✣✧✱✔ ✗✚✗✚✗ ✲✗ ☎✢✔ . A nonoverlapping expansion is desirable because it is easy to determine its sign (take the sign of the largest component) or to produce a crude approximation of its value (take the component with largest magnitude).

✥

✯

✥

✯

✥

✯

✔

✔✚✔✚✔✚✔✚✔ Note that this definition of expansion is slightly different from that used by Priest [23]; whereas Priest requires that the exponents of any two components of the expansion differ by at least , no such requirement is made here.

✗

✔

✔

✟

✟

✔

✟

Two floating-point values and are adjacent if they overlap, if overlaps , or if overlaps . For instance, ✔✚✔ ✗✚✗ is adjacent to ✔✚✔ , but ✗✚✗✚✗ is not. An expansion is nonadjacent if no two of its components are adjacent. Surprisingly, any floating-point value has a corresponding nonadjacent expansion; for instance, may appear at first not to be representable as a nonoverlapping expansion of one-bit components, but

✵

✳

Formally, and are nonoverlapping if there exist integers and such that ✶✻✺★✸✙✼✞✽ and ✾ ✷✿✾❁❀❂✼✑✽ , or ✷❃✺❄✸✙✼✑✽ and ✾ ✶❅✾❁❀❂✼✑✽ .

✴

✶

✷

✸

✹

a0

a0

✗

✥

✥

✯

✔

✗

✔

✔

a0

✗

✎

✯

a0

✄

✄

a0

✂

✁

✤

a0

✟

✥

✥

✤

✂

consider the expansion ✗✚✗✚✗✚✗✚✗ ☛✕✔ . The trick is to use the sign bit of each component to separate it from its larger neighbor. We will later see algorithms in which nonadjacent expansions arise naturally.

✔

✟

Multiple-component algorithms (based on the expansions defined above) can be faster than multipledigit algorithms because the latter require expensive normalization of results to fixed digit positions, whereas multiple-component algorithms can allow the boundaries between components to wander freely. Boundaries are still enforced, but can fall at any bit position. In addition, it usually takes time to convert an ordinary floating-point number to the internal format of a multiple-digit library, whereas any ordinary floating-point number is an expansion of length one. Conversion overhead can account for a significant part of the cost of small extended precision computations.

✂

a0

✁

✁

✔

✗

✔

✧

✔

The central conceptual difference between standard multiple-digit algorithms and the multiple-component algorithms described herein is that the former perform exact arithmetic by keeping the bit complexity of operands small enough to avoid roundoff error, whereas the latter allow roundoff to occur, then account for it after the fact. To measure roundoff quickly and correctly, a certain standard of accuracy is required from the processor's floating-point units. The algorithms presented herein rely on the assumption that addition, subtraction, and multiplication are performed with exact rounding . This means that if the exact result can be stored in a -bit significand, then the exact result is produced; if it cannot, then it is rounded to the nearest -bit floating-point value. For instance, in four-bit arithmetic the product ✔✚✔✚✔ ✗✚✗✚✗ ✔✚✔ is rounded to ✔✘☎ ✗✚✗ . If a value falls precisely halfway between two consecutive -bit values, a tiebreaking rule determines the result. Two possibilities are the round-to-even rule, which specifies that the value should be rounded to the nearest -bit value with an even significand, and the round-toward-zero rule. In four-bit arithmetic, ✗✚✗ ✔✚✔ is rounded to ✔✘☎ under the round-to-even rule, and to ✔✘☎ ✗✚✗ under the roundtoward-zero rule. The IEEE 754 standard specifies round-to-even tiebreaking as a default. Throughout this article, the symbols , , and represent -bit floating-point addition, subtraction, and multiplication with exact rounding. Due to roundoff, these operators lack several desirable arithmetic properties. Associativity is an example; in four-bit arithmetic, ✗✚✗✚✗ ✔✚✔ ✞✝✟✂ ✔✚✔✍✧ ✗✚✗✚✗ , but ✗✚✗✚✗ ✂✠✆ ✔✚✔ ✡✂ ✔✚✔ ✞✝ ❃✧ ✗✚✗ . A list of reliable identities for floating-point arithmetic is given by Knuth [17].

✗

✂

✂

Another useful notation is err ✆☞☛✎✍ ✏✝ , which denotes the roundoff error incurred by using a -bit floatingpoint operation to approximate a real operation (addition, subtraction, multiplication, or division) on the operands and . Note that whereas ulp is an unsigned quantity, err is signed. For any basic operation, ☛✒✍ ✓☛✔✑ err ✆☞☛✕✍ ✏✝ , and exact rounding guarantees that err ✆☞☛✕✍ ✏✝✏✖✘✗ ulp ✆☞☛✒✍ ✏✝ .

✂

✂

✆

☛

✝

✧

✔

✆

✔

✧

✆

✔

✂

✗

☎

✗

✗

☎

✗

✔

✔

✗

☎

✗

✗

☎

✗

✔

✔

Roundoff is often analyzed in terms of ulps , or 'units in the last place.' An ulp is the effective magnitude of the low-order ( th) bit of a -bit significand. An ulp is defined relative to a specific floating point value; I shall use ulp ✆☞☛✌✝ to denote this quantity. For instance, in four-bit arithmetic, ulp ✔✚✔ ✗✚✗ , and ulp ✞✝ ✗✚✗ .

☛

✄

☎

✔

✂

✍

✑

❃✄ In the pages that follow, various properties of floating-point arithmetic are proven, and algorithms for manipulating expansions are developed based on these properties. Throughout, binary and decimal numbers are intermixed; the base should be apparent from context. A number is said to be expressible in bits if it can be expressed with a -bit significand, not counting the sign bit or the exponent. I will occasionally refer to the magnitude of a bit , defined relative to a specific number; for instance, the magnitude of the second nonzero bit of binary ☛✕✔✚✔✚✔ is four. The remainder of this section is quite technical; the reader may wish to skip the proofs on a first reading. The key new results are Theorems 13, 19, and 24, which provide algorithms for summing and scaling expansions.

✖

✄

✧

✔

✁

✔

✟

✄

✂

✄

✄

✂

☎

✗

✗

✔

✗

✁

a0

✆

✂

✄

✂

✄

✤

✔

✁

a0

✄

✆

✖

✔

✗

✍

✄

✗

☎

a0

✗

✭

✭

✄

✗

✄

✂

✗

✄

<!-- image -->

✠

☎

✠

✠

✄

✟

✄

☛

☛

Figure 1: Demonstration of the first two lemmata. Vertical lines represent four-bit floating-point values. The roundoff error is the distance between ✁✄✂✆☎ and ✁✞✝✟☎ . Lemma 1 states that the error cannot be larger than . Lemma 3(b) states that if ✁✄✂✟☎ ✠☛✡✌☞✎✍✑✏✒☞✔✓✖✕✘✗ ✂✚✙✜✛ (for ✢✤✣✦✥ and ✧★✣✪✩ , this means that ✁✄✂✆☎ falls into the darkened region), then the error is no greater than ☞☛✍ . This lemma is useful when a computed value falls close to a power of two.

## 2.2 Properties of Binary Arithmetic

☛

✄

✄

✤

☞

Exact rounding guarantees that err ✆☞☛✕✍ ✏✝✏✖✌✗ ulp ✆☞☛✕✍ ✏✝ , but one can sometimes find a smaller bound for the roundoff error, as evidenced by the two lemmata below. The first lemma is useful when one operand is much smaller than the other, and the second is useful when the sum is close to a power of two. For Lemmata 1 through 5, let and be -bit floating-point numbers.

✖

Lemma 1 Let ☛✕✂ ✄✻✧ err ✆☞☛✔✂ . The roundoff error err ✆☞☛✕✂ ✝✏✖ is no larger than or . (An analogous result holds for subtraction.)

☛

✭

✄

✂

✄

✄

✟

✄

✂

✖

✖

✖

Corollary 2 The roundoff error err ✆☞☛✕✂ ✏✝ can be expressed with a -bit significand.

✆

✄

✄

✄

✖

✂

✖

✖

✄

✄

✄

✄

✟

✟

☛

✝

✖

✖

☛

✖

✖

✖

Proof: Assume without loss of generality that ☛✟✖ ✬✫ . The sum ☛✔✂ ✏✄ is the -bit floating-point number closest to . But is a -bit floating-point number, so err ✆☞☛✒✂ ✏✝✏✖✘✗ ✖✘✗ . (See Figure 1.)

✄

✖

✖

✖

Lemma 3 For any basic floating-point operation , let ☛✒✍ ☛✔✑ err ✆☞☛✕✍ ✏✝ . Then:

✖

✍

✄

a0

✯

✖

☛

✑

✄

✖

a0

✆

a0

✟

✔

☛

✖

✖

✖

☛

✖

Proof: Assume without loss of generality that ☛✟✖ ✮✫ . Clearly, the least significant nonzero bit of err ✆☞☛✟✂ ✏✝ is no smaller in magnitude than ulp ✏✝ . By Lemma 1, err ✆☞☛ ✬✄ ✏✝✏✖✘✗ ; hence, the significand of err ✆☞☛ ✓✄ ✏✝ is no longer than that of . It follows that err ✆☞☛✕✂ ✏✝ is expressible in bits.

✄

✧

✄

✟

- (a) If err ✆☞☛ ✏✝✏✖ for some integer , then ✔✱ ✞✝ .

✯

✑

- ✮✫ ✰✫ (b) If err ✆☞☛ ✏✝✏✖ ✮✲ for some integer , then ✰✲ ✱✴✳ ✞✝ .

✄

✂

✄

✖

☛

✑

a0

✄

✖

✔

a0

✭

✭

✆

a0

✤

✟

✂

✔

✔

✔

✔

✗

✔

✗

✂

✄

## Proof:

a0

✔

☛

✧

✔

✗

✗

☛

✧

✔

✔

<!-- formula-not-decoded -->

✔

✔

☛

✧

✔

✔

✄

✧

✔

✗

Figure 2: Two demonstrations of Lemma 5.

a0

✭

✆

☛

✝

✎

✂

✖

☛

✑

✖

a0

✗

✱

a0

✟

✭

✆

✱

✝

✭

✆

✱

- (b) The numbers ✔✱✎✳ ✑✎ ✔✱✎✳ ✑✎ ✔✱✴✳ ☛✲✖ ✑✎✠☎✠☎✠☎✆✎ are all expressible in bits. Any value ❃✄ ✖✘✗ ✱✴✳ ✞✝ is within a distance of from one of these numbers. (See Figure 1.)

✭

- (a) The numbers ✑✎ ☛✲✔ ✞✝ ✑✎ ✑✎✠☎✠☎✠☎ are all expressible in bits. Any value ✻✄ ✁a0 ✔✱ ✞✝ is within a distance less than from one of these numbers.

a0

✤

✔

a0

✭

✤

✤

☛

✂

✭

✆

✆

✝

✝

✖

☛

✑

✭

✆

✤

✝

✭

✆

✭

a0

✟

The next two lemmata identify special cases for which computer arithmetic is exact. The first shows that addition and subtraction are exact if the result has smaller magnitude than the operands.

Lemma 4 Suppose that and ✖✘✗ . Then ✬✄ . (An analogous result holds for subtraction.)

✆

✟

✟

✫

✄

✄

✖

✖

✖

☛

Many of the algorithms will rely on the following lemma, which shows that subtraction is exact for two operands within a factor of two of each other:

✄

✄

✄

✖

☛

✖

✖

☛

✟

✟

✟

Proof: Without loss of generality, assume ☛✟✖ . Clearly, the least significant nonzero bit of is no smaller in magnitude than ulp ✏✝ . However, ✖✘✗ . It follows that can be expressed in bits.

Lemma 5 (Sterbenz [26]) Suppose that ✄✂ ✆☎ ✿✒ . Then ✓☛ ✕☛✓✄ .

✄

☛

✄

✄

✄

✡

✎

✄

✧

a0

Proof: Without loss of generality, assume ☛✟✖ ✮✫ . (The other case is symmetric, because .) Then ✝✂ ✞☎ . The difference satisfies ✖✘✗ ✖✘✗ ; the result follows by Lemma 4.

✂

☛

✄

✂

✄

✡

✄

✎

a0

☛

✒

☛

☛

✄

✓✡ ✿✒ ☛✓✄ Two examples demonstrating Lemma 5 appear in Figure 2. If and have the same exponent, then floating-point subtraction is analogous to finding the difference between two -bit integers of the same sign, and the result is expressible in bits. Otherwise, the exponents of and differ by one, because ✄✂ ✟☎ . In this case, the difference has the smaller of the two exponents, and so can be expressed in bits.

✎

## 2.3 Simple Addition

An important basic operation in all the algorithms for performing arithmetic with expansions is the addition of two -bit values to form a nonoverlapping expansion (of length two). Two such algorithms, due to Dekker and Knuth respectively, are presented.

☛

☛

✄

✂

✥

✟

✯

☛

✟

✄

✖

☛

✧

✖

✫

✥

✖

✄

✟

✖

✂

Theorem 6 (Dekker [7]) Let and be -bit floating-point numbers such that . Then the following algorithm will produce a nonoverlapping expansion such that , where is an approximation to and represents the roundoff error in the calculation of .

✯

✥

✄

✄

☛

✟

✖

☛

✄

✖

✗

✖

☛

✖

✖

☛

✄

✖

✄

✖

☛

✂

✧

☛

✄

a0

a0

✭

✆

✔

☛

✄

✟

a0

a0

✄

✧

✧

a0

a0

✔

a0

✯

a0

✗

a0

a0

✖

☛

a0

✖

a0

✭

✖

a0

✖

✖

✖

☛

✖

✗

✄

✄

✗

✗

✗

✥

✗

✗

✔

✔

✔

✁

☛

a0

✄

✤

✂

✄

✧

☛

✂

✄

✄

☛

☛

✭

✭

✥

✄

✄

✎

✎

☞

✄

✥

✯

✧

✄

✄

✧

✄

<!-- formula-not-decoded -->

✗

Figure 4: Demonstration of F AST -T WO -S UM where and have opposite sign and ✝✆✟✞ ✠✡✞ .

✥

✄

✄

☛

Proof: Line 1 computes , but may be subject to rounding, so we have err ✆☞☛ ✲✄ ✏✝ . By assumption , so and must have the same sign (or ).

✖

☎

✄

☛

✎

<!-- formula-not-decoded -->

✂

☎

✄

☛

☛

✄

✥

✖

✄

✖

✡

✲

✧✏✥ ✧✰✄ Line 3 is also computed exactly. By Corollary 2, virtual err ✆☞☛✕✂ ✏✝ is expressible in bits.

✯

✄

✂

✥

✎

✯

✆

✝

✟

✟

✟

✮✫ ✥★✧ Line 2 computes the quantity virtual , which is the value that was really added to in Line 1. This subtraction is computed exactly; this fact can be proven by considering two cases. If and have the same sign, or if ✖✘✗ , then ✞☎ ✿✒ and one can apply Lemma 5 (see Figure 3). On the other hand, if and are opposite in sign and , then ✡☞☛ ✎✠☛ ✿✒ and one can apply Lemma 5 to Line 1, showing that was computed exactly and therefore virtual (see Figure 4). In either case the subtraction is exact, so virtual err ✆☞☛✕✂ ✏✝ .

✖

✖

✯

✥

✧

☛

✄

✂

☛

✄

✧

✥

✯

✧

☛

Note that the outputs and do not necessarily have the same sign, as Figure 3 demonstrates. Twoterm subtraction ('F AST -T WO -D IFF ') is implemented by the sequence ✏☞ ☎✑ virtual ✒✑ ✓☞ virtual . The proof of the correctness of this sequence is analogous to Theorem 6.

✗

✆

✟

✟

✟

✟

✤

✥

✥

✯

✧

✄

✄

✟

✄❃☛✓✄ It follows that err ✆☞☛ ✏✄ ✏✝ and err ✆☞☛ ✲✄ ✏✝ , hence . Exact rounding guarantees that ulp ✟✝ , so and are nonoverlapping.

✯

✄

✄

✄

✥

✄

✄

☞

✥

✎

☎

✎

✂

✄

☛

✄

✎

✄

✄

✙

✙

✖

☛

☛

✖

☛

✄

✖

✄

✖

✙

✄

☛

☛

✄

✥

✥

✯

✧

✄

✄

✧

☛

✔

✔

<!-- formula-not-decoded -->

Figure 3: Demonstration of F AST -T WO -S UM where and have the same sign. The sum of ✙✎✙✴✙✴✙ ✁a0✂a0 and ✄a0☎a0 is the expansion ✄a0☎a0 ✄a0☎a0✂a0 ✂✪✥ ✙✎✙ .

☛

✧

✔

✗

✗

✔

✁

a0

✤

✙

☛

☛

✄

✥

☛

✧

✧

✧

✧

✧

✄

✧

✧

✧

✧

✥

✄

✥

☛

✄

✄

☛

✄

✧

✧

✄

✄

✥

✄

✯

a0

✯

✁

✄

✧

✧

✔

✄

✗

☛

☛

✔

✔

✁

✔

☎

✗

✔

✔

✗

✔

✔

☎

✧

✔

✔

✔

✔

✔

✗

✗

✗

✗

✔

☛

✗

✔

✔

✔

✔

✔

✔

✔

✗

✗

✔

✔

✔

✗

✔

✁

✥

✄

a0

✤

✧

☛

✁

✁

✁

a0

a0

a0

✄

☛

✄

✁

✄

✄

☛

✠

☎

✠

☛

✂

✂

☛

✄

✭

✯

The difficulty with using F AST -T WO -S UM is the requirement that ☛✟✖ . If the relative sizes of and are unknown, a comparison is required to order the addends before invoking F AST -T WO -S UM . With most C compilers , perhaps the fastest portable way to implement this test is with the statement ' if ((a &gt; b) == (a &gt; -b)) '. This test takes time to execute, and the slowdown may be surprisingly large because on modern pipelined and superscalar architectures, an if statement coupled with imperfect microprocessor branch prediction may cause a processor's instruction pipeline to drain. This explanation is speculative and machine-dependent, but the T WO -S UM algorithm below, which avoids a comparison at the cost of three additional floating-point operations, is usually empirically faster . Of course, F AST -T WO -S UM remains faster if the relative sizes of the operands are known a priori , and the comparison can be avoided.

☛

☛

✄

✥

✯

✄

✟

✟

✂

✆

Theorem 7 (Knuth [17]) Let and be -bit floating-point numbers, where . Then the following algorithm will produce a nonoverlapping expansion such that ✧✲✥ , where is an approximation to and is the roundoff error in the calculation of .

✥

☛

☞

✎

✄

✄

✄

✄

☛

☛

|   T WO -S | UM                      |
|-----------|-------------------------|
|         1 |                         |
|         2 | ☛✒✂ virtual             |
|         3 | virtual virtual         |
|         4 | roundoff ✡✄ virtual ☛✒✄ |
|         5 | roundoff virtual        |
|         6 | roundoff roundoff       |
|         7 | ✓☞ return               |

☛

☛

☛

☞

☞

✄

✄

✄

☛

☛

✥✟✎ Proof: If ✬✫ , then Lines 1, 2, and 4 correspond precisely to the F AST -T WO -S UM algorithm. Recall from the proof of Theorem 6 that Line 2 is calculated exactly; it follows that Line 3 of T WO -S UM is calculated exactly as well, because virtual can be expressed exactly. Hence, roundoff is zero, roundoff is computed exactly, and the procedure is correct.

✖

✯

✆

✝

✖

☛

✖

✖

✄

✖

✧✰✄ Now, suppose that , and consider two cases. If ✎✖ ☛✟✖ , then is computed exactly by Lemma 4. It immediately follows that virtual , virtual , and roundoff , roundoff , and are zero.

✥

✥

✧

☛

✄

✂

✄

✟

✆

✥

✄

☛

✝

✄

✧

✄

✟

✫

✖

☛

✂

✖

✄

✧✏✥ ☛✓✄ Because , we have ✎✖ ❅✧ ✖ ☛✔✂ , so the roundoff errors err ✆☞☛✔✂ ✏✝ and err each cannot be more than ulp , so virtual ✓✡ (for ✏✛ ) and Lemma 5 can be applied to show that Line 4 is exact. Hence, roundoff err ✆☞☛ ✦☛ err . Finally, Line 6 is exact because by Corollary 2, roundoff roundoff err ✆☞☛✕✂ ✏✝ is expressible in bits.

☛

✧

✧✰✄ Conversely, if ✎✖ , Lines 1 and 2 may be subject to rounding, so err ✆☞☛ ✏✝ , and virtual err ✆☞☛ ✏✝ err . (See Figure 5.) Lines 2, 3, and 5 are analogous to the three lines of F AST -T WO -D IFF (with Line 5 negated), so Lines 3 and 5 are computed exactly. Hence, virtual virtual err ☛✌✝ , and roundoff err ☛✌✝ .

✄

✒

✖

✄

✖

✂

✫

✄

✥

✂

✄

✄

✎

✖

a0

✗

✄

✖

✄

✖

✲

✖

☛

✖

✆

✖

✝

✄

It follows that err ✆☞☛✕✂ ✏✝ and err ✆☞☛✕✂ ✏✝ , hence .

☞

✥

✑

☞

☞

✑

The exceptions are those few that can identify and optimize the fabs() math library call.

✆

✥

✄

☛

✝

✟

✄

✧

☛

✂

✄

✝

✥✣✧ ✧✏✥ Two-term subtraction ('T WO -D IFF ') is implemented by the sequence ✲✄ ☎✑ virtual ✒✑ virtual ✕✂ virtual roundoff virtual ☎✑ roundoff virtual ✓☞ roundoff ✬✄ roundoff .

✥

✄

✄

✄

✄

✄

☛

☛

✄

☛

✯

☛

✂

✂

On a DEC Alpha-based workstation, using the bundled C compiler with optimization level 3, T WO -S UM uses roughly 65% as much time as F AST -T WO -S UM conditioned with the test ' if ((a &gt; b) == (a &gt; -b)) '. On a SPARCstation IPX, using the GNUcompiler with optimization level 2, T WO -S UM uses roughly 85% as much time. On the other hand, using the SPARCstation's bundled compiler with optimization (which produces slower code than gcc), conditional F AST -T WO -S UM uses only 82% as much time as T WO -S UM . The lesson is that for optimal speed, one must time each method with one's own machine and compiler.

✄

☞

✄

☞

✥

✟

✟

✄

✄

☛

✧

✥

☛

✄

✄

☛

✥

✯

✖

a0

✖

a0

✟

✯

✂

✫

✛

✥

☛

☛

✟

✄

✄

✯

✖

☛

✁

✧

✖

a0

☛

☛

✖

✖

✆

✥

✯

✄

☛

☞

☞

✥

✥

✥

✂

✄

✄

✖

✖

✖

✫

✖

✄

✖

☛

✄

✯

✧

☛

✧

☛

✄

✄

✄

✁

☛

✟

☛

a0

✄

✟

✂

✧

✄

✆

✥

✄

☛

✟

✄

☛

✟

✄

✯

✆

✥

✄

☛

☛

✝

✄

✭

✯

✯

✟

a0

a0

✟

<!-- image -->

✠

✁

☎

✄

✧

✧

✂

☛

☛

✗

✗

☎

☎

✗

✔

✔

✔

☛

✯

✧

✧

☛

Figure 5: Demonstration of T WO -S UM where ✁a0 and ✘✠ . The sum of ✙✴✙ ☎✄ ✙✎✙ and ✙✴✙ ✄a0 is the expansion ✄a0☎a0✂a0☎a0 ✆✄ ✙✴✙ .

✁

✙

✂

a0

Corollary 8 Let and be the values returned by F AST -T WO -S UM or T WO -S UM .

- (a) If for some integer , then ✔✱ ✞✝ .

✯

✥

✯

✔

✖

✖

✆

✖

✯

✖

a0

✯

Proof: is the roundoff error err ✆☞☛ ✏✄ for some and . By Theorems 6 and 7, . The results follow directly from Lemma 3.

✭

✥

✯

- ✰✫ ✰✫ (b) If ✰✲ for some integer , then ✰✲ ✱✴✳ ✞✝ .

✯

✄

✄

✧

✥

✯

✂

✝

☛

☛

☛

Corollary 9 Let and be the values returned by F AST -T WO -S UM or T WO -S UM . On a machine whose arithmetic uses round-to-even tiebreaking, and are nonadjacent.

✆

✥

✯

Proof: Exact rounding guarantees that ulp ✟✝ . If the inequality is strict, and are nonadjacent. If ulp ✟✝ , the round-to-even rule ensures that the least significant bit of the significand of is zero, so and are nonadjacent.

✥

✧

✤

## 2.4 Expansion Addition

✞

✞

✟

✟

✆

✝

Having established how to add two -bit values, I turn to the topic of how to add two arbitrary precision values expressed as expansions. Three methods are available. E XPANSION -S UM adds an -component expansion to an -component expansion in ✠✝✡✞ time. L INEAR -E XPANSION -S UM and F AST -E XPANSION -S UM do the same in ✠✝ time.

Despite its asymptotic disadvantage, E XPANSION -S UM can be faster than the linear-time algorithms in cases where the size of each expansion is small and fixed, because program loops can be completely unrolled and indirection overhead can be eliminated (by avoiding the use of arrays). The linear-time algorithms have conditionals that make such optimizations untenable. Hence, E XPANSION -S UM and F AST -E XPANSION -S UM are both used in the implementations of geometric predicates described in Section 4.

✆

✯

✗

✄

✤

✆

✥

✥

✯

✥

✯

✖

✖

a0

✭

✖

✟

✖

a0

✭

✆

a0

✟

✠

✠

✠

✠

✠

✡

✠

✂

✙

✄

✯

✄

✭

☛

✄

✥

☛

☛

✄

✧

✧

✧

✧

✧

✧

✧

✝

✂

✥

✄

✄

✥

✟

✥

✄

✄

✄

☛

✄

✯

✭

✧

✧

✧

✧

✤

✔

✔

✗

✔

✔

✗

✔

✔

✔

✗

✗

✗

✗

✔

✔

✗

✗

✔

✔

✔

✔

✔

✔

✁

a0

✤

✟

✝

✥

✟

✭

✥

E XPANSION -S UM and L INEAR -E XPANSION -S UM both have the property that their outputs are nonoverlapping if their inputs are nonoverlapping, and nonadjacent if their inputs are nonadjacent. F AST -E XPANSION -S UM is faster than L INEAR -E XPANSION -S UM , performing six floating-point operations per component rather than nine, but has three disadvantages. First, F AST -E XPANSION -S UM does not always preserve either the nonoverlapping nor the nonadjacent property; instead, it preserves an intermediate property, described later. Second, whereas L INEAR -E XPANSION -S UM makes no assumption about the tiebreaking rule, F AST -E XPANSION -S UM is designed for machines that use round-to-even tiebreaking, and can fail on machines with other tiebreaking rules. Third, the correctness proof for F AST -E XPANSION -S UM is much more tedious. Nevertheless, I use F AST -E XPANSION -S UM in my geometric predicates, and relegate the slower L INEAR -E XPANSION -S UM to Appendix A. Users of machines that have exact rounding but not round-to-even tiebreaking should replace calls to F AST -E XPANSION -S UM with calls to L INEAR -E XPANSION -S UM .

✟

✗

☎

✔

✟

✟

✟

✗

✗

☎

✔

✔

✗

✗

✗

A complicating characteristic of all the algorithms for manipulating expansions is that there may be spurious zero components scattered throughout the output expansions, even if no zeros were present in the input expansions. For instance, if the expansions ✔✚✔✚✔✚✔ and ✔✚✔ ✗✚✗ ☎✢✔✚✔ are passed as inputs to any of the three expansion addition algorithms, the output expansion in four-bit arithmetic is ✔✚✔✚✔ ✗✚✗ ✕✗ ✗✚✗✚✗ . One may want to add expansions thus produced to other expansions; fortunately, all the algorithms in this article cope well with spurious zero components in their input expansions. Unfortunately, accounting for these zero components could complicate the correctness proofs significantly. To avoid confusion, most of the proofs for the addition and scaling algorithms are written as if all input components are nonzero. Spurious zeros can be integrated into the proofs (after the fact) by noting that the effect of a zero input component is always to produce a zero output component without changing the value of the accumulator (denoted by the variable ). The effect can be likened to a pipeline delay; it will become clear in the first few proofs.

Each algorithm has an accompanying dataflow diagram, like Figure 6. Readers will find the proofs easier to understand if they follow the diagrams while reading the proofs, and keep several facts in mind. First, Lemma 1 indicates that the down arrow from any T WO -S UM box represents a number no larger than either input to the box. (This is why a zero input component yields a zero output component.) Second, Theorems 6 and 7 indicate that the down arrow from any T WO -S UM box represents a number too small to overlap the number represented by the left arrow from the box.

a0

I begin with an algorithm for adding a single -bit value to an expansion.

✠

✂

✛

✁

✧

✁

✠

✭

✤

✁

✁

✎

✠

☎

✠

✠

✠

✤

✄

✳

✎

✄

```
G ROW -E XPANSION ☛✁ ✚✎ ✏✝ 1 2 for to 3 ☞a0 T WO -S UM ☞a0 ✟✁ ✠✭ 4 ✌a0 5 return
```

☞

✳

✤

☞

✭

✎

✭

✭

✎

☎

☎

✠

✠

Theorem 10 Let ✄✂✆☎ ✞✝ ✠✭ be a nonoverlapping expansion of -bit components, and let be a -bit value where . Suppose that the components ✟✁ ✎✠☎✠☎✠☎✜✎ ✟✁ are sorted in order of increasing magnitude, except that any of the may be zero. Then the following algorithm will produce a nonoverlapping expansion such that ✞✝ ✭✟✧ ✡✁ , where the components ✎✠☎✠☎✠☎✆✎ are also in order of increasing magnitude, except that any of the may be zero. Furthermore, if is nonadjacent and round-to-even tiebreaking is used, then is nonadjacent.

a0

✞

☞

✆

✝

✝

✆

is an approximate sum of and the first components of ; see Figure 6. In an implementation, the array can be collapsed into a single scalar.

✄

✠

✄

✄

✯

☎

✝

✂

✂

✟

✠

a0

a0

✭

✧

✫

✂

☎

✭

✳

✤

✤

✭

✠

✟

✭

✯

✤

☞

✔

✤

✝

✆

✄

✁

☛

✁

✤

✄

✂

✠

✍

<!-- image -->

✂

☎

✆

✤

✆

✁

✄

✁

✠

✠

✠

✠

✠

Figure 6: Operation of G ROW -E XPANSION . The expansions and are illustrated with their most significant components on the left. All T WO -S UM boxes in this article observe the convention that the larger output ( ) emerges from the left side of each box, and the smaller output ( ) from the bottom or right. Each term is an approximate running total.

✠

Proof: At the end of each iteration of the for loop, the invariant ✕✭ holds. Certainly this invariant holds for ❃✧ after Line 1 is executed. From Line 3 and Theorem 7, we have that ✭✦✧ ✜✭ ; from this one can deduce inductively that the invariant holds for all (relevant values of) . Thus, after Line 4 is executed, .

✖

✯

✭

✠

a0

✭

✠

✭

✁

✠

✠

✟

a0

✭

a0

✭

☛

✁

For all , the output of T WO -S UM (in Line 3) has the property that and do not overlap. By Lemma 1, ✖✘✗ ✜✭ , and because is a nonoverlapping expansion whose nonzero components are arranged in increasing order, cannot overlap any of ✟✁ ✎✠☎✠☎✠☎ . It follows that cannot overlap any of the later components of , because these are constructed by summing with later components. Hence, is nonoverlapping and increasing (excepting zero components of ). If round-to-even tiebreaking is used, then and are nonadjacent for all (by Corollary 9), so if is nonadjacent, then is nonadjacent.

a0

✭

✁

✭

✳

✤

✎

✭

✳

✠

✠

✭

If any of the ✜✭ is zero, the corresponding output component is also zero, and the accumulator value is unchanged ( ). (For instance, consider Figure 6, and suppose that is zero. The accumulator value shifts through the pipeline to become , and a zero is harmlessly output as . The same effect occurs in several algorithms in this article.)

✖

✖

✁

✄

✖

a0

Corollary 11 The first components of are each no larger than the corresponding component of . (That is, ✖✘✗ ✖✘✗ ✎✠☎✠☎✠☎❁✎ ✖✘✗ .) Furthermore, ✖✘✗ .

✠

✖

✁

✖

✎

✖

✠

✄

✖

✁

✄

✖

✖

✠

☎

✖

✁

☎

✖

✖

✠

Proof: Follows immediately by application of Lemma 1 to Line 3. (Both of these facts are apparent in Figure 6. Recall that the down arrow from any T WO -S UM box represents a number no larger than either input to the box.)

✁

✤

✤

✤

If is a long expansion, two optimizations might be advantageous. The first is to use a binary search to find the smallest component of greater than or equal to ulp ✏✝ , and start there. A variant of this idea, without the search, is used in the next theorem. The second optimization is to stop early if the output of a T WO -S UM operation is the same as its inputs; the expansion is already nonoverlapping.

✁

✆

✄

A na¨ ıve way to add one expansion to another is to repeatedly use G ROW -E XPANSION to add each component of one expansion to the other. One can improve this idea with a small modification.

✭

✝

a0

✭

✧

a0

✭

☛

✤

✭

✁

✁

✠

✖

✁

✖

✁

✭

✭

a0

✟

✂

✝

✭

✝

✝

✧

✄

✟

✂

✝

✭

✝

✁

✝

✄

✂

✭

✟

✯

a0

✁

a0

✆

✤

✁

✟

✁

✁

✆

a0

a0

✯

✁

✗

✁

✁

✁

✁

a0

a0

✄

✁

✁

✁

✄

a0

a0

✤

✤

✠

✁

✁

✁

✤

a0

✄

✄

✭

✠

✁

✯

✂

✝

☎

✝

✳

✠

✤

✤

✠

✝

✧

✂

✁

✝

☎

✝

✄

✤

✁

✝

✁

✄

✠

✠

a0

✁

✤

a0

a0

✭

✠

✠

✁

✂✁

✠

✁

✁

✂✄

✁

✆

✁

✄

✠

✠

✠

Figure 7: Operation of E XPANSION -S UM .

<!-- image -->

✛

a0

✧

✠

✭

☎

✳

✤

✭

✩

✠

✭

✧

a0

✧

✁

✠

✂

✠

```
E XPANSION -S UM ☛✁ ✚✎ 1 2 for to 3 ✎✠☎✠☎✠☎❁✎ G ROW -E XPANSION ✝☎ ✎✠☎✠☎✠☎✆✎ 4 return
```

✭

✳

✠

✩

✭

✁

✧

✭

✟

✩

✭

a0

✭

✆

Theorem 12 Let ✬✧ ✞✝ and ✞✝ be nonoverlapping expansions of and ❄✂ -bit components, respectively, where . Suppose that the components of both and are sorted in order of increasing magnitude, except that any of the ✆✭ or may be zero. Then the following algorithm will produce a nonoverlapping expansion such that ✞✝ ✡✁ , where the components of are in order of increasing magnitude, except that any of the may be zero. Furthermore, if and are nonadjacent and round-to-even tiebreaking is used, then is nonadjacent.

✠

✁

✠

✠

✭

✎

✠

✭

✳

✤

✠

✭

✳

☎

☛

✯

☞

✔

Proof: That ✞✝ ✞✝ ✞✝ upon completion can be proven by induction on Line 3.

☎

☎

☛

✤

✆

✭

✤

✆

✎

a0

✭

✝

✭

✳

☎

✆

☞

☎

✭

✠

✎

✠

✭

✳

✤

✠

✠✭ After setting , E XPANSION -S UM traverses the expansion from smallest to largest component, individually adding these components to using G ROW -E XPANSION (see Figure 7). The theorem would follow directly from Theorem 10 if each component were added to the whole expansion , but to save time, only the subexpansion ✎✠☎✠☎✠☎✆✎ is considered. (In Figure 7, this optimization saves three T WO -S UM operations that would otherwise appear in the lower right corner of the figure.)

a0

a0

✭

✖

✠

✝

✖

✗

✖

a0

✖

a0

✤

✎

a0

✄

✖

✠

a0

✝

✭

✖

☛

a0

✤

✖

a0

✖

✠

✭

✎

✠

✭

✳

✤

✠

✭

✳

When is considered, the components ✎✠☎✠☎✠☎✜✎ have already been summed into . According to Corollary 11, after iteration of Line 3. Because is an increasing nonoverlapping expansion, for any , cannot overlap , and furthermore (unless ). Therefore, when one sums into , one can skip the first ☛✓✔ components of without sacrificing the nonoverlapping and increasing

a0

a0

✂

✩

✭

✤

a0

✭

a0

✁

✁

a0

✁

✁

✁

✁

✆

a0

a0

✁

✁

✁

✁

✁

a0

a0

a0

a0

✁

✁

✁

✁

✄

a0

a0

a0

✄

✠

✁

✁

✁

✤

✤

a0

a0

✤

✞

✯

✂

✠

☎

✝

✤

☞

☞

✠

✝

✂

✂

☎

✭

☎

✤

✂

✆

✁

✤

✞

✠

✭

✯

✁

✫

✁

✁

a0

✭

✝

a0

✂

✠

✞

✤

✠

✁

✭

a0

a0

✠

✁

✟

a0

✭

✁

a0

a0

✭

✧

✁

✗

✁

a0

a0

✝

✠

✠

✠

✞

✠

✤

✠

✭

✗

✗

✗

✂

✔

✔

✔

✔

☎✢✔ Under the assumption that all expansions are strongly nonoverlapping, it is possible to prove the first key result of this article: the F AST -E XPANSION -S UM algorithm defined below behaves correctly under roundto-even tiebreaking. The algorithm can also be used with round-toward-zero arithmetic, but the proof is different. I have emphasized round-to-even arithmetic here due to the IEEE 754 standard.

✔

☛

✠

properties of . Similarly, if and are each nonadjacent, one can skip the first components of without sacrificing the nonadjacent property of .

a0

✭

✠

☛

✔

✁

✯

a0

✠

✠

No difficulty ensues if is a spurious zero component, because zero does not overlap any number. G ROW -E XPANSION will deposit a zero at and continue normally.

✟

✔

✟

✔

✟

✔

✗

✟

✔

✟

Unlike E XPANSION -S UM , F AST -E XPANSION -S UM does not preserve the nonoverlapping or nonadjacent properties, but it is guaranteed to produce a strongly nonoverlapping output if its inputs are strongly nonoverlapping. An expansion is strongly nonoverlapping if no two of its components are overlapping, no component is adjacent to two other components, and any pair of adjacent components have the property that both components can be expressed with a one-bit significand (that is, both are powers of two). For instance, ✔✚✔ ✗✚✗✚✗ ✔✚✔ and ✗✚✗✚✗✚✗ ✗✚✗✚✗ are both strongly nonoverlapping, but ✔✚✔✚✔ ✗✚✗ ✔✚✔ is not, nor is ✗✚✗ . A characteristic of this property is that a zero bit must occur in the expansion at least once every bits. For instance, in four-bit arithmetic, a strongly nonoverlapping expansion whose largest component is ✔✚✔✚✔✚✔ can be no greater than ✔✚✔✚✔✚✔✘☎ ✔✚✔✚✔✚✔ ✔✚✔✚✔✚✔ ☎✠☎✜☎ . Any nonadjacent expansion is strongly nonoverlapping, and any strongly nonoverlapping expansion is nonoverlapping, but the converse implications do not apply. Recall that any floating-point value has a nonadjacent expansion; hence, any floating-point value has a strongly nonoverlapping expansion. For example, ✔✚✔✚✔✚✔✘☎✢✔ may be expressed as ✗✚✗✚✗✚✗ .

✗

✟

A variant of this algorithm was presented by Priest [23], but it is used differently here. Priest uses the algorithm to sum two nonoverlapping expansions, and proves under general conditions that the components of the resulting expansion overlap by at most one digit (i.e. one bit in binary arithmetic). An expensive renormalization step is required afterward to remove the overlap. Here, by contrast, the algorithm is used to sum two strongly nonoverlapping expansions, and the result is also a strongly nonoverlapping expansion. Not surprisingly, the proof demands more stringent conditions than Priest requires: binary arithmetic with exact rounding and round-to-even tiebreaking, consonant with the IEEE 754 standard. No renormalization is needed.

✁

✧

✂

☎

✭

✝

✤

✁

✂

a0

✖

✧

✂

✩

✭

✁

✤

a0

✭

✁

a0

✝

✠

✞

✂

Theorem 13 Let ✜✭ and ✞✝ be strongly nonoverlapping expansions of and -bit components, respectively, where . Suppose that the components of both and are sorted in order of increasing magnitude, except that any of the ❁✭ or may be zero. On a machine whose arithmetic uses the round-to-even rule, the following algorithm will produce a strongly nonoverlapping expansion such that ✞✝ , where the components of are also in order of increasing magnitude, except that any of the may be zero.

✠

✁

a0

a0

a0

✧

✭

✧

✂

☎

✁

✩

a0

✠

✠

✳

```
F AST -E XPANSION -S UM ✚✎ 1 Merge and into a single sequence , in order of nondecreasing magnitude (possibly with interspersed zeros) 2 ☞a0 F AST -T WO -S UM ✁a0 ✂a0 3 for to 4 ☞a0 T WO -S UM ☞a0 ✂a0 5 ✓☞ 6 return
```

✭

✭

✠

✆

☎

✄

✳

✯

✎

✩

✆

☞

✤

✠

✝

✭

✛

✎

☞

a0

✠

✭

☎

✟

✟

✗

✟

✟

✫

☛

✝

✳

✤

✝

✩

✟

☞

✆

✞

✁

✝

a0

✭

✆

✆

✄

✎

✭

☛

✤

✤

✝

✎

✭

✝

✭

✖

✧

☛

a0

a0

✖

✁

☛

✖

✗

☎

✗

✗

a0

a0

✖

a0

✁

✗

✟

a0

✯

a0

✧

✗

If we omit from the sum, there are two cases to consider. If , then is less than , and is less than . (For instance, if , , and , then ✍☛ ✄a0 ☎✢✔ ✔✚✔✚✔✚✔ ✔✚✔✚✔✚✔ ☎✜☎✠☎ , and ✔✘☎✢✔ ✔✚✔✚✔✚✔ ✔✚✔✚✔✚✔ ☎✠☎✜☎ .) Conversely, if ✁✆☎ , then is less than , and is less than . (For instance, if , , and ✔✚✔✚✔✚✔ , then ✔✚✔✚✔✚✔ ✔✚✔✚✔✚✔ ☎✠☎✠☎ , and ✔✚✔✚✔✚✔✘☎ ✔✚✔✚✔✚✔ ✔✚✔✚✔✚✔✦☎✠☎❁☎ .) In either case, .

✁

✟

a0

a0

✖

a0

a0

✭

✆

a0

✱

✝

✭

a0

a0

✖

✗

✗

✗

✖

a0

a0

☛

a0

a0

✁

✖

✧

✖

✁

a0

✆

☛

✤

✝

✂

✧

✖

✯

✧

Lemma 16 The expansion produced by F AST -E XPANSION -S UM is a nonoverlapping expansion whose components are in order of increasing magnitude (excepting zeros).

✭

<!-- image -->

✤

✆

✁

✄

✁

✠

✠

✠

✠

✠

Figure 8: Operation of F AST -E XPANSION -S UM . The terms maintain an approximate running total.

a0

✍

✆

is an approximate sum of the first components of ; see Figure 8.

a0

Several lemmata will aid the proof of Theorem 13. I begin with a proof that the sum itself is correct.

✂

✂

✝

☎

✯

✭

Lemma 14 (Q Invariant) At the end of each iteration of the for loop, the invariant holds. This assures us that after Line 5 is executed, , so the algorithm produces a correct sum.

a0

✝

✳

✤

✩

✠

✝

✧

✂

✝

☎

✝

✳

✤

✩

a0

✝

✭

✝

✤

✧

✝

✝

✤

a0

✝

Proof: The invariant clearly holds for ❃✧ after Line 2 is executed. For larger values of , Line 4 ensures that ; the invariant follows by induction.

a0

a0

✧

✂

✝

✖

✖

✂

✝

✁

✝

✤

✤

a0

a0

✝

✖

✯

✯

✭

✭

✟

✠

✭

☛

✧

a0

✭

☛

✟

a0

✭

Lemma 15 Let be a series formed by merging two strongly nonoverlapping expansions, or a subseries thereof. Suppose that is the largest component and has a nonzero bit of magnitude or smaller for some integer . Then ✔✱✴✳ ✞✝ , and ✔✱ .

a0

a0

a0

a0

✁

a0

✁

a0

a0

✖

✁

a0

✁

✆

✝

✤

☛

✔

a0

✭

✆

✖

a0

✭

✆

✝

a0

a0

✝

✯

✂

✝

✁

✝

✤

Proof: Let and be the expansions (or subsequences thereof) from which was formed, and assume that the component comes from the expansion . Because is the largest component of and has a nonzero bit of magnitude or smaller, and because is strongly nonoverlapping, ✂a0 ✌✖ is less than ✔✱ . (For instance, if ✂✣✧✏✖ and , then ✘✖ ✔✚✔✚✔✚✔✘☎ ✔✚✔✚✔✚✔ ✔✚✔✚✔✚✔✦☎✠☎✜☎ .) The same bound applies to the expansion , so ✟✖ ✔✱✴✳ ✞✝ .

✖

a0

✖

✗

a0

✭

a0

✂

✧

✖

a0

a0

✯

✧

✗

✁

✖

a0

✁

✧

✔

a0

a0

✖

✁

✗

✆

✝

✗

✗

✖

✖

✤

a0

a0

a0

a0

✁

a0

✭

a0

a0

✟

✂

✝

✭

☛

✤

✝

✖

a0

a0

✖

a0

a0

a0

a0

✧

✭

✖

a0

✁

a0

✁

✤

a0

✁

✁

✁

a0

a0

✆

a0

✁

✁

✆

a0

a0

✁

a0

✁

✁

✁

a0

a0

✄

a0

✁

✁

✄

a0

a0

✤

✖

✱

a0

✁

✁

a0

a0

✄

a0

✗

a0

a0

✖

✁

✁

a0

a0

a0

✭

✭

✗

✆

✆

✤

✝

✁

✝

✤

☛

✠

✝

✔

a0

a0

✗

✧

a0

a0

a0

✭

✁

☛

a0

a0

✁

☛

a0

a0

✁

✁

a0

✧

a0

a0

a0

✖

✭

a0

a0

a0

✁

a0

a0

a0

✁

✖

✖

a0

✁

✗

✗

a0

☛

✗

✭

a0

a0

✄

✤

✁

a0

✖

✭

✗

a0

☛

a0

a0

✄

✤

✠

a0

✭

a0

a0

a0

✔

☎

✔

✗

✗

✗

✗

☎

✔

✗

✗

<!-- image -->

✂

✂

✄

✂

☎

✧

✣

✩

✄

✆

✍

✕

✗

✄

✆

✄

✔

✔

✔

✔

✔

✔

✗

✔

✔

✔

✔

✔

✔

✔

✔

✔

✗

✔

✔

✔

✗

✔

✔

✔

✖

a0

✖

✂

✭

✝

✠

✤

✭

☛

✤

✖

✖

Figure 9: Demonstration (for ) of how the Q Invariant is used in the proof that is nonoverlapping. The top two values, and , are being summed to form . Because has a nonzero bit of magnitude no greater than , and because is formed by merging two strongly nonoverlapping expansions, the sum ✝✟✞ ✡✠ ✝✟✞ can be no larger than illustrated in this worst-case example. As a result, ☛✠ cannot be large enough to have a roundoff error of , so ☛✠ is smaller than and cannot overlap . (Note that ✕✘✗ is not part of the sum; it appears above in a box drawn as a placeholder that bounds the value of each expansion.)

✁

✠

☛

✎✠☎✠☎✠☎❁✎ Assume without loss of generality that the exponent of is zero, so that is of the form ✔✘☎ , where an asterisk represents a sequence of arbitrary bits.

✭

✭

☛

✤

✖

✠

✖

a0

a0

✑

<!-- formula-not-decoded -->

✠

✭

✑

✖

a0

✱

✟

✖

a0

✠

✆

✍

✙

✆

✄

✍

✗

✙

✆

✍

✕

✗

Proof: Suppose for the sake of contradiction that two successive nonzero components of overlap or occur in order of decreasing magnitude. Denote the first such pair produced and ; then the components are nonoverlapping and increasing (excepting zeros).

✠

✭

☛

✤

✠

✭

☛

✤

a0

✑

✤

✤

and are produced by a T WO -S UM or F AST -T WO -S UM operation, and are therefore nonadjacent by Corollary 9 (because the round-to-even rule is used). ✕✭ is therefore of the form ✗✚✗ (having no bits of magnitude smaller than four). Because ✰✫ , Corollary 8(a) guarantees that

✠

✭

✟

✤

✠

✭

☛

Because the offending components and are nonzero and either overlapping or of decreasing magnitude, there must be at least one nonzero bit in the significand of whose magnitude is no greater than one. One may ask, where does this offending bit come from? is computed by Line 4 from and ✘✭ , and the offending bit cannot come from (which is of the form ✗✚✗ ), so it must have come from ✘✭ . Hence, has a nonzero bit of magnitude one or smaller. Applying Lemma 15, one finds that .

✭

a0

✠

✭

☛

✤

a0

✑

✠

✤

✠

✭

☛

✤

✭

✳

✤

✤

a0

a0

✖

✱

a0

✭

✳

✤

✖

A bound for ☛✌✄ can be derived by recalling that is of the form ✔✘☎ , and ✎✠☎✠☎✠☎❁✎ are nonoverlapping and increasing. Hence, ☛✌✄ .

✤

✠

✝

✠

✝

✤

✂

✝

✝

Rewrite the Q Invariant in the form ☛✌✄ . Using the bounds derived above, we obtain

✭

✖

a0

✭

✤

✖

✂

✝

✭

✝

✠

✝

<!-- formula-not-decoded -->

See Figure 9 for a concrete example.

Inequalities 1 and 2 cannot hold simultaneously. The result follows by contradiction.

It is implicitly assumed here that the first offending pair is not separated by intervening zeros. The proof could be written to consider the case where intervening zeros appear, but this would make it even more convoluted. Trust me.

☞

✭

✟

☛

a0

✱

✟

✭

✟

✭

☛

✠

✧

✭

☛

✖

✂

✝

✭

✝

✠

✖

✠

✤

✭

a0

✭

✠

✠

✠

✭

✤

✭

✠

✖

✂

✳

✝

✍

✭

a0

✝

✖

✂

✝

✭

✝

✤

a0

✖

a0

✙

✖

✂

✖

✂

✟

✝

✭

✝

✝

a0

✁

✠

✝

✝

✝

✁

✖

✖

✗

✗

✆

✔

a0

✔

✭

✳

✤

✔

✗

☎

✗

✗

✔

✔

✔

✔

✔

✔

✔

✔

✗

✗

✔

✔

✔

✔

✔

✔

✔

✔

✗

✗

✔

✔

✔

✔

✗

✍

✝

✠

✭

✖

✂

☛

a0

✠

✍

☛

✗

✝

✠

a0

✭

☛

✭

✤

☛

✤

a0

✤

✔

✭

✂

✔

a0

✝

✝

✤

✠

a0

✝

✠

✤

✠

✠

✠

✍

✂

✍

✗

✠

✄

☎

✙

✄

✆

✄

✄

✄

✁

<!-- image -->

✂

✄

✆

✆

✄

☛

✄

✄

✄

☛

✧

✁

✆

Figure 10: A four-bit example where F AST -E XPANSION -S UM generates two adjacent components and ✁a0 . The figure permits me a stab at explaining the (admittedly thin) intuition behind Theorem 13: suppose is of the form ☎✄ . Because is the roundoff term associated with ✆a0 , ✝a0 must be of the form a0✂a0 if round-to-even arithmetic is used. Hence, the bit of magnitude in ✞a0 must have come from . This implies that is no larger than ✙✴✙✎✙✴✙ ✄a0 , which imposes bounds on how large ✟a0 and ✡✠ can be (Lemma 15); these bounds in turn imply that can be no larger than , and ☛a0 can be no larger than ✄a0 . Furthermore, ☞✠ cannot be adjacent to ☛a0 because neither ✌✠ nor ✍a0 can have a bit of magnitude .

✠

✧

✟

a0

✠

✩

✙

Proof of Theorem 13: Lemma 14 ensures that . Lemma 16 eliminates the possibility that the components of overlap or fail to occur in order of increasing magnitude; it remains only to prove that is strongly nonoverlapping. Suppose that two successive nonzero components and are adjacent.

✠

✠

a0

✑

✭

✠

☛

✠

Assume without loss of generality that the exponent of is zero, so that is of the form ✔✘☎ . As in the proof of Lemma 16, must have the form ✗✚✗ .

✭

☛

✤

✗

✭

a0

✭

✤

✠

☛

✤

✠

✭

☛

✤

a0

✑

Because and are adjacent, the least significant nonzero bit of has magnitude two; that is, is of the form . Again we ask, where does this bit come from? As before, this bit cannot come from , so it must have come from ✘✭ . Hence, ✘✭ has a nonzero bit of magnitude two. Applying Lemma 15, we find that ✔✱✴✳ and ✔✱✴✳ .

✭

✠

✝

a0

✠

a0

✭

✤

✝

✭

a0

✖

a0

✳

✤

✖

a0

a0

a0

✑

✔

✳

a0

a0

✳

✄

✤

Bounds for and ☛✌✄ can also be derived by recalling that is of the form ✔✘☎ and is the largest component of a nonoverlapping expansion. Hence, , and ☛✌✄ .

✝

✭

☛

✝

✭

✝

✤

✠

✝

✂

✝

✝

✤

✤

✠

✝

✂

Rewriting the Q Invariant in the form , we obtain

a0

✭

✳

✠

✟

✝

✤

✤

✤

✤

✝

✭

☛

✖

✠

✝

✖

✂

✝

✤

✖

✔

✂

✝

a0

✖

✝

<!-- formula-not-decoded -->

☛

✠

a0

a0

✝

✤

✤

✄

✭

✤

✭

✧

The Q Invariant also gives us the identity ✕✭ ☛✌✄ . Hence,

✖

✠

✖

✭

✤

✝

✭

✠

✝

☎

✝

✭

a0

✝

<!-- formula-not-decoded -->

✭

a0

✖

a0

✟

✠

✭

☛

✖

a0

a0

✤

✟

✤

✤

Recall that the value is at least . Consider the possibility that might be greater than ; by Corollary 8(b), this can occur only if ✱✴✳ , contradicting Inequality 3. Hence, must be exactly , and is expressible in one bit. (Figure 10 gives an example where this occurs.)

✤

✖

✠

✖

☛

✭

✳

✟

✭

✳

☛

☛

a0

a0

✭

✭

✠

✠

✠

✁

✁

a0

a0

✖

✁

a0

✂

✁

a0

✠

✝

✭

✧

✁

✝

✤

✤

✁

✁

✔

✧

✝

✁

✖

a0

a0

a0

✁

✔

✭

a0

✁

✆

☛

a0

✄

✁

✄

✖

a0

✧

✭

✖

✳

a0

✂

✁

✁

a0

✤

✖

✝

✭

a0

✟

✝

✟

✤

✠

a0

✗

✭

✤

✝

✖

✭

☛

✖

✔

✠

✲

a0

a0

a0

✧

✁

✖

✂

✂

✄

✤

✝

✟

✝

a0

✄

a0

✧

✁

✁

✂

✂

a0

✗

✝

a0

✔

✄

✁

✤

✧

✁

✁

✗

✠

✝

a0

a0

✤

✧

✗

✠

✂

☛

✠

✙

✄

✗

✠

✭

☛

☛

✠

✠

✧

☛✕✔

✗

✠

a0

✁

✠

☞

✧

✠

✄

☛✕✔

✠

✠

✆

✠

✭

✠

✆

✠

✠

✤

✭

✧

✗

✂

☛

✖

✠

✑

✭

✖

a0

✂

a0

✫

✖

✖

✫

✫

✔

a0

✑

✖

a0

✭

✳

✄

✎

✭

✳

✁

Because and ✘✭ ✂a0 ✘✭ ✎✠☎✠☎✠☎ are of the form ✗✚✗✚✗ , ✎✠☎✠☎✠☎ must be as well, and are therefore not adjacent to . It follows that cannot contain three consecutive adjacent components.

✠

✔

✖

✭

✤

✖

✖

✠

✔

✭

☛

✖

Similarly, the value is at least . Consider the possibility that might be greater than ; by Corollary 8(b), this can occur only if ✱✴✳ , contradicting Inequality 4. Hence, must be exactly , and is expressible in one bit.

✖

✂

✝

✝

✝

✖

a0

✭

✟

✭

☛

✫

a0

✱

✟

✔

✖

✭

☛

✝

✝

a0

✝

✖

a0

✖

a0

✭

✔

✟

✠

✭

☛

✖

✲

a0

✤

✟

✖

✭

✤

✖

By Corollary 8(a), (because ✧✱✔ ). Using this inequality, the inequality ☛✌✄ , and the Q Invariant, one can deduce that ✟✂ ✤✲ ✔✱ . Because is formed from two nonoverlapping increasing expansions, this inequality implies that ✿✭ ☛✌✄ ✗✚✗ binary (recalling that ), and hence ✂a0 ✎✠☎✠☎✠☎ must all be of the form ✗✚✗✚✗ (having no bits of magnitude smaller than ). is also of the form ✗✚✗✚✗ , because and are produced by a T WO -S UM or F AST -T WO -S UM operation, and are therefore nonadjacent by Corollary 9 (assuming the round-to-even rule is used).

✭

✭

✳

✎

✠

✭

✳

✄

a0

✳

a0

✠

✳

✄

✎

✳

✁

✠

a0

✑

These arguments prove that if two components of are adjacent, both are expressible in one bit, and no other components are adjacent to them. Hence, is strongly nonoverlapping.

✟

✤

✤

✭

✠

✠

The proof of Theorem 13 is more complex than one would like. It is unfortunate that the proof requires strongly nonoverlapping expansions; it would be more parsimonious if F AST -E XPANSION -S UM produced nonoverlapping output from nonoverlapping input, or nonadjacent output from nonadjacent input. Unfortunately, it does neither. For a counterexample to the former possibility, consider adding the nonoverlapping expansion ✔✚✔✚✔✚✔ ✗✚✗✚✗✚✗ ✔✚✔✚✔✚✔ ❂✗ ☎✢✔✚✔✚✔✚✔ to itself in four-bit arithmetic. (This example produces an overlapping expansion if one uses the round-to-even rule, but not if one uses the round-toward-zero rule.) For a counterexample to the latter possibility, see Figure 10. On a personal note, it took me quite a bit of effort to find a property between nonoverlapping and nonadjacent that is preserved by F AST -E XPANSION -S UM . Several conjectures were laboriously examined and discarded before I converged on the strongly nonoverlapping property. I persisted only because the algorithm consistently works in practice.

✟

It is also unfortunate that the proof requires explicit consideration of the tiebreaking rule. F AST -E XPANSION -S UM works just as well on a machine that uses the round-toward-zero rule. The conditions under which it works are also simpler-the output expansion is guaranteed to be nonoverlapping if the input expansions are. One might hope to prove that F AST -E XPANSION -S UM works regardless of rounding mode, but this is not possible. Appendix B demonstrates the difficulty with an example of how mixing round-toward-zero and round-to-even arithmetic can lead to the creation of overlapping expansions.

The algorithms E XPANSION -S UM and F AST -E XPANSION -S UM can be mixed only to a limited degree. E XPANSION -S UM preserves the nonoverlapping and nonadjacent properties, but not the strongly nonoverlapping property; F AST -E XPANSION -S UM preserves only the strongly nonoverlapping property. Because nonadjacent expansions are strongly nonoverlapping, and strongly nonoverlapping expansions are nonoverlapping, expansions produced exclusively by one of the two algorithms can be fed as input to the other, but it may be dangerous to repeatedly switch back and forth between the two algorithms. In practice, E XPANSION -S UM is only preferred for producing small expansions, which are nonadjacent and hence suitable as input to F AST -E XPANSION -S UM .

✁

✁

✁

☛

✝

✟

✞

☛

✔

✖

✁

✭

✖

✖

a0

✝

✖

✣☛ These estimates correlate well with the measured performance of the algorithms. I implemented each procedure as a function call whose parameters are variable-length expansions stored as arrays, and measured

✝

✟

✞

☛

a0

It is useful to consider the operation counts of the algorithms. E XPANSION -S UM uses ✝✡✞ T WO -S UM operations, for a total of ✝✡✞ flops (floating-point operations). F AST -E XPANSION -S UM uses T WO -S UM operations and one F AST -T WO -S UM operation, for a total of ✄✂ flops. However, the merge step of F AST -E XPANSION -S UM requires comparison operations of the form ' if ✰✲ '. Empirically, each such comparison seems to take roughly as long as three flops; hence, a rough measure is to estimate that F AST -E XPANSION -S UM takes as long to execute as flops.

✭

✭

a0

✭

✤

✳

✠

✤

✔

✔

✖

a0

✠

☛

a0

✑

✠

✤

✖

a0

✭

✳

✤

✤

✠

✭

✖

✠

✤

✭

✖

✤

a0

✂

✝

✟

a0

a0

✠

✝

✱

✂

☛

✞

✟

✔

✞

a0

✤

✗

them on a DEC Alpha-based workstation using the bundled compiler with optimization level 3. By plotting their performance over a variety of expansion sizes and fitting curves, I found that E XPANSION -S UM runs in ✠✝ ✁a0 microseconds, and F AST -E XPANSION -S UM runs in ✄✂ ✝✡✞ ✰✗ microseconds. F AST -E XPANSION -S UM is always faster except when one of the expansions has only one component, in which case G ROW -E XPANSION should be used.

☎

a0

✛

✆

✟

✞

✝

☛

✗

☎

✗

☎

✖

✟

☎

✁

As I have mentioned, however, the balance shifts when expansion lengths are small and fixed. By storing small, fixed-length expansions as scalar variables rather than arrays, one can unroll the loops in E XPANSION -S UM , remove array indexing overhead, and allow components to be allocated to registers by the compiler. Thus, E XPANSION -S UM is attractive in this special case, and is used to advantage in my implementation of the geometric predicates of Section 4. Note that F AST -E XPANSION -S UM is difficult to unroll because of the conditionals in its initial merging step.

✠

On the other hand, the use of arrays to store expansions (and non-unrolled loops to manage them) confers the advantage that spurious zero components can easily be eliminated from output expansions. In the procedures G ROW -E XPANSION , E XPANSION -S UM , and F AST -E XPANSION -S UM , as well as the procedures S CALE -E XPANSION and C OMPRESS in the sections to come, zero elimination can be achieved by maintaining a separate index for the output array and advancing this index only when the procedure produces a nonzero component of . In practice, versions of these algorithms that eliminate zeros are almost always preferable to versions that don't (except when loop unrolling confers a greater advantage). Zero elimination adds a small amount of overhead for testing and indexing, but the lost time is virtually always regained when further operations are performed on the resulting shortened expansions.

✠

Experience suggests that it is economical to use unrolled versions of E XPANSION -S UM to form expansions of up to about four components, tolerating interspersed zeros, and to use F AST -E XPANSION -S UM with zero elimination when forming (potentially) larger expansions.

## 2.5 Simple Multiplication

The basic multiplication algorithm computes a nonoverlapping expansion equal to the product of two -bit values. The multiplication is performed by splitting each value into two halves with half the precision, then performing four exact multiplications on these fragments. The trick is to find a way to split a floating-point value in two. The following theorem was first proven by Dekker [7]:

✱

✆

✄

✗

✗

☛

☛

✂

✖

☛

✖

✖

☛

✖

☛

☛

☛

✂

✫

✟

✠

✂

Theorem 17 Let be a -bit floating-point number, where . Choose a splitting point such that ✆☎ ✂★☛✰✔ . Then the following algorithm will produce a ✂✣☛ ✝☎ -bit value hi and a nonoverlapping ✞☎ ✻☛ ✞✝ -bit value lo such that hi ✰✫ lo and hi lo .

☛

☞

a0

✟

✄

✔

☛

✝

✆

☎

<!-- formula-not-decoded -->

☛

☞

☛

✎

☛

☛

☛

✔

✂

☛

☞

✄

☛

The claim may seem absurd. After all, hi and lo have only ✂✣☛✰✔ bits of significand between them; how can they carry all the information of a -bit significand? The secret is hidden in the sign bit of lo . For instance, the seven-bit number ✗✚✗ ✗✚✗ can be split into the three-bit terms ✗✚✗✚✗✚✗ and ☛✕✔✚✔✚✔ . This

✆

✛

✝

☛

☎

✔

✔

✔

✧

☞

✝

☛

✔

✗

✔

☛

✯

☛

✯

✄

☛

☛

✔

✔

☛

☛

✧

☛

<!-- formula-not-decoded -->

✧

Figure 11: Demonstration of S PLIT splitting a five-bit number into two two-bit numbers.

a0

☛

✂

☛

☛

✡

property is fortunate, because even if is odd, as it is in IEEE 754 double precision arithmetic, can be split into two ✂✁ -bit values.

✠

✧

a0

☛

✟

☛

☛

✟

✆

✡

a0

☛

☛

✆

✝

✡

☛

✆

✠

✄

☛

✝

a0

a0

✡

Line 2 is also subject to rounding, so big err ✂✠☛✌✝ err . It will become apparent shortly that the proof relies on showing that the exponent of big is no greater than the exponent of . Both err ☛✌✝✏✖ and err ☛✌✝✏✖ are bounded by ulp , so the exponent of big can only be larger than that of ☛✡ if every bit of the significand of is nonzero except possibly the last (in four-bit arithmetic, must have significand ✔✚✔✚✔ or ✔✚✔✚✔✚✔ ). By manually checking the behavior of S PLIT in these two cases, one can verify that the exponent of big is never larger than that of .

✆

✠

✄

☛

✝

☎

☎

✄

a0

✡

✻☛ By Lemma 5, Lines 3 and 4 are calculated exactly. It follows that hi err , and lo err ; the latter is expressible in bits. To show that hi is expressible in ✂❂☛ bits, consider that its least significant bit cannot be smaller than ulp ✆☞☛ big ✦✧ ulp ✆☞☛✌✝ . If hi has the same exponent as , then hi must be expressible in bits; alternatively, if hi has an exponent one greater than that of (because ❃☛ err ☛✌✝ has a larger exponent than ), then hi is expressible in one bit (as demonstrated in Figure 11).

a0

☛

✱

✄

✂

Proof: Line 1 is equivalent to computing . (Clearly, can be expressed exactly, because multiplying a value by a power of two only changes its exponent, and does not change its significand.) Line 1 is subject to rounding, so we have err ☛✒✂ .

✖

✆

✝

✤

✠

✄

☛

☛

✆

✡

☛

✂

✖

✆

✠

✄

a0

The reason this fact is useful is because, with Line 2, it implies that err ☛✌✝✏✖ ulp , and so the error term err is expressible in bits (for ).

☛

✂

☛

☛

☛

☎

✝

☛

☛

Finally, the exactness of Line 4 implies that hi lo as required. Multiplication is performed by setting ✣✧ ☎✄ ✂✆ , so that the -bit operands and are each split into two -bit pieces, hi , lo , hi , and lo . The products hi hi , lo hi , hi lo , and lo lo can each be computed exactly by the floating-point unit, producing four values. These could then be summed using the F AST -E XPANSION -S UM procedure in Section 2.4. However, Dekker [7] provides several faster ways to accomplish the computation. Dekker attributes the following method to G. W. Veltkamp.

☛

✄

✂

✥

✟

✯

☛

✄

✧

✥

✟

✯

✂

✫

✁

✥

☛

✄

✁

☛

☛

✄

✄

☛

✱

✂

☛

✄

☎

✄

☛

✄

☛

✄

☛

✄

Theorem 18 Let and be -bit floating-point numbers, where . Then the following algorithm will produce a nonoverlapping expansion such that , where is an approximation to and represents the roundoff error in the calculation of . Furthermore, if round-to-even tiebreaking is used, and are nonadjacent. (See Figure 12.)

✆

✠

✄

☛

✝

☎

☛

✔

a0

✡

☛

✧

☛

☎

✧

a0

✱

✄

✤

✆

✠

✄

a0

☛

a0

☛

✁

☛

☛

☛

✠

✧

✧

✧

✧

✧

✧

✆

a0

✠

✁

✠

✄

✟

✄

☛

✔

☛

☎

✗

☛

✡

☛

✔

☛

✧

✧

✧

✧

☛

☛

a0

✧

✔

✥

✡

☛

✟

✗

✔

✔

✫

✟

a0

✗

✔

✔

☛

a0

✗

✔

✔

✔

☛

a0

✡

✗

✗

✗

✗

✔

✔

☛

✗

✗

✔

✔

✔

✟

✖

✗

✔

✔

☛

✆

✠

✗

✗

✗

✄

✔

✔

☛

✗

✁

✁

✁

✁

a0

a0

a0

a0

✆

✁

✁

✤

✆

✠

✄

✆

☛

☛

✝

✝

☛

✡

☛

✭

✥

☛

✡

☞

✯

✁

✁

✄

✧

✧

☎

✝

<!-- image -->

✁

✣

☞

☎

✣

✁

✣

☎

✁

✣

☎

✣

☛

✔

✔

✔

✗

✗

✗

✗

✔

✔

☛

✯

✧

✁

✁

☛

☎

✄

Figure 12: Demonstration of T WO -P RODUCT in six-bit arithmetic where ✣✦✙✎✙✴✙ ✁a0 ✙✴✙ , hi hi ✙✴✙✴✙ ✁a0✂a0☎a0 , and lo lo ✙✎✙ . Note that each intermediate result is expressible in six bits. The resulting expansion is ✙✴✙ ✁a0 ✙✎✙ ✄a0 ✝✆ ✟✞ ✂✪✙✎✙ ✄a0✂a0 .

✥

✆

☛

✝

✝

✄

✄

✆

✄

✄

✱

✆

✆

✝

✝

|   T WO -P RODUCT |                               |
|------------------|-------------------------------|
|                1 | ✪✎                            |
|                2 | ✌☞ ☛✒☎ ✆☞☛ hi lo S PLIT ✆☞☛   |
|                3 | ✪✎ hi lo S PLIT ✄a0☎a0 ✄✓✆☞☛  |
|                4 | ✄❁✎ hi hi ✄a0☎a0 ✁a0✂a0 ✄✓✆☞☛ |
|                5 | lo hi                         |
|                6 | ✄a0☎a0 ✁a0✂a0 hi lo           |
|                7 | ✄✓✆☞☛ ✆☞☛ lo lo ✄a0☎a0        |
|                8 | ✍☞ return                     |

a0

✁

☛

☎

✤

✄

✝

✄

✁

☎

✁

✄

✝

✟

✱

✄

✄

Without loss of generality, assume that the exponents of and are , so that and are integers in the range ✤✠✎ ✔✙✒ . In the proof of Theorem 17 it emerged that hi and hi are integers in the range ✔✱ ✔✱ , and lo and lo are integers in the range ✡✠ ✟☛ ✌☞ . From these ranges and the assumption that , one can derive the inequalities lo hi , lo hi , and err ✆☞☛ ✏✝ ✁☎✄ hi hi .

☛

✄

✥

✧

☛

✄

✟

☛

☎

☎

✄

✧

☛

✄

✟

☛

✄

✟

☛

✄

✟

☛

✄

✟

☛

We are assured that Line 5 is executed without roundoff error if the value ✁a0☎a0 ✠☛ lo hi hi lo lo lo err ✆☞☛ ✲✄ ✏✝ is expressible in bits. I prove that this property holds by showing that the left-hand expression is a multiple of ✑☛ ✌☞ , and the right-hand expression is strictly smaller than ✟☛ ✌☞ .

☛

✁

☛

✄

☎

✄

☛

a0

✱

✄

✖

☛

✖

☛

Proof: Line 1 is subject to rounding, so we have ✥✣✧ err ✆☞☛✎☎ ✣✄ ✏✝ . The multiplications in Lines 4 through 7 are all exact, because each factor has no more than bits; it will be proven that each of the subtractions is also exact, and thus err ✆☞☛✕☎ ✏✝ .

✡

a0

✱

☛

✖

✖

✄

✖

✖

☛

✖

✗

✡

✍

✤

✖

✎

☛

✖

✖

✄

✤

✒

✖

✗

✍

✤

✖

✄

✖

☎

✄

✗

a0

✱

☛

✤

✗

✂

✤

✎

✫

✒

✁

☛

✖

Intuitively, hi hi ought to be within a factor of two of ✲✄ , so that Line 4 is computed exactly (by Lemma 5). To confirm this hunch, note that err ✆☞☛ ✏✝ hi hi lo hi hi lo lo lo err ✆☞☛✕☎ ✏✝ hi hi ✏✎ hi hi (using the inequalities stated above), which justifies the use of Lemma 5. Because Line 4 is computed without roundoff, ✁a0✂a0 lo hi hi lo lo lo err ✆☞☛✒☎ ✏✝ .

✄

✟

✂

✤

a0

✄

a0

✁

✄

The upper bound on the absolute value of the right-hand expression follows immediately from the upper bounds for hi , lo , lo , and err ✆☞☛ ✏✝ . To show that the left-hand expression is a multiple of ✒✠ ✟☛ ✌☞ , consider that ✁a0☎a0 must be a multiple of because ☛✔☎ and hi hi have exponents of at least ✂★☛ .

a0

✱

☛

✄

a0

✤

✖

☛

☛

✄

a0

✄

✖

✧

✁

✁

✁

✥

☛

✄

✁

✄

✤

✧

✧

✧

✧

✧

✧

✙

✁

✁

✥

✄

✤

☛

☛

☛

☛

☎

☎

☎

☎

☎

✄

✄

✄

✄

☎

☎

✄

✄

✄

✝

✝

✝

✁

✧

✧

✧

✧

✧

✧

✧

✄

✤

✎

✎

✄

✔

☞

✔

✥

✔

✔

✧

✧

✗

✗

✗

✔

✔

☎

✗

✗

✔

✔

✔

✔

✄

✗

✗

✗

✗

✔

✔

✱

✄

✄

☛

✗

✗

✔

✔

✔

✔

✗

✗

✗

✔

✔

✔

✔

✗

✗

✔

✔

✔

✔

✔

✗

✗

✗

✗

✗

✗

✗

✔

✔

✗

✔

✔

✁

✁

✁

✁

✁

✁

a0

a0

a0

a0

a0

a0

✄

✄

✁

✄

✄

✄

☎

✄

✤

a0

✯

✄

✤

✆

✧

☛

✠

✱

✄

✄

a0

✱

✁

☛

✤

✄

✁

✁

☞

✤

✆

✥

✁

✧

✎

✯

☛

✄

✝

✗

a0

✄

✄

✱

✟

✄

☛

☎

☛

✄

✄

✝

✄

✄

✖

✝

☛

✂

✟

☛

✖

☛

✔

✄

✖

✁

✟

✖

☛

✖

☛

✖

✠

✱

✄

✄

✖

✄

✖

✧

☛

a0

✄

✟

a0

a0

✤

✠

✠

✠

✆

✁

✄

<!-- image -->

✁

✁

✁

✠

✁

✁

Figure 13: Operation of S CALE -E XPANSION .

✁

a0

✱

✟

✟

✱

✠

To show that Line 6 is computed without roundoff error, note that lo lo is an integer no greater than ✔✱ (because lo and lo are integers no greater than ✡✠ ✟☛ ✌☞ ), and err ✆☞☛ ✬✄ ✏✝ is an integer no greater than . Thus, ✁a0☎a0 lo lo err ✆☞☛ is an integer no greater than , and is expressible in bits.

✥

✯

✭

☛

✄

☛

☛

☎

✠

✠

✍

✠

✁

✁

✂✁

✠

✁

✁

✆

✁

✁

✂✄

Hence, ✁a0☎a0 lo hi must be a multiple of ✟☛ ✌☞ because lo is an integer, and hi is a multiple of ✡✠ ✟☛ ✌☞ . Hence, Line 5 is computed exactly, and ✄a0☎a0 hi lo lo lo err ✆☞☛ ✏✝ .

a0

☛

✁

✄

☛

a0

✄

☛

Finally, Line 7 is exact simply because err ✆☞☛✒☎ ✏✝ can be expressed in bits. Hence, ✧✲✥ . If round-to-even tiebreaking is used, and are nonadjacent by analogy to Corollary 9.

✱

☛

✤

✤

✁

☛

✄

✄

✟

✧

✁

☛

☛

✄

☛

## 2.6 Expansion Scaling

The following algorithm, which multiplies an expansion by a floating-point value, is the second key new result of this article.

✠

✂

✁

✖

✁

✧

✠

☎

✭

✤

✁

✭

✁

✠

✄

✁

```
S CALE -E XPANSION ☛✁ ✿✎ ✏✝ 1 ☞a0 T WO -P RODUCT ☛✁ ✏✝ 2 for to 3 T WO -P RODUCT ☛✁ ✠✭ 4 ☞a0 ☛✌✄ T WO -S UM ☞a0 ☛✌✄ 5 ☞a0 F AST -T WO -S UM ✭✑✎ 6 7 return
```

✝

✂

✄

✂

✠

Theorem 19 Let ✄✂ ✞✝ be a nonoverlapping expansion of -bit components, and let be a -bit value where . Suppose that the components of are sorted in order of increasing magnitude, except that any of the may be zero. Then the following algorithm will produce a nonoverlapping expansion such that , where the components of are also in order of increasing magnitude, except that any of the may be zero. Furthermore, if is nonadjacent and round-to-even tiebreaking is used, then is nonadjacent.

✆

✄

✎

✠

✆

a0

✤

✝

a0

☞

✁

✝

✄

✆

✤

✎

✆

✄

✝

☎

✝

✱

✤

✠

a0

✁

✍

a0

✤

✆

a0

a0

✁

✁

✆

a0

a0

a0

✄

✄

✁

a0

✁

a0

a0

✁

✄

✧

✁

✁

✁

✁

✁

✁

a0

a0

a0

✄

✆

✄

✁

a0

✄

✄

a0

a0

✁

✁

✁

✁

✄

✁

✄

a0

a0

a0

✄

✄

✁

✁

✤

a0

✄

✧

✂

✠

✫

✭

✭

✭

✄

✝

☎

✤

✭

✧

✠

✄

☎

✯

☞

✆

✆

☞

✄

✠

✭

✄

✄

a0

✎

✭

✭

☛

✎

✄

✭

✠

☎

✤

✯

✄

✎

☞

✝

✭

✠

✧

☛

✆

✄

✁

✤

✭

☛

✝

☞

✝

☞

✄

✆

✎

✄

a0

✄

✱

✆

✭

a0

✎

☎

a0

✁

✄

✭

✝

✄

✭

☛

✤

✂

✄

✝

✂

☛

✁

✄

a0

✟

✄

✯

a0

✭

✄

a0

✁

✁

✝

✄

✝

a0

As illustrated in Figure 13, S CALE -E XPANSION multiplies each component of by and sums the results. It should be apparent why the final expansion is the desired product, but it is not so obvious why the components of are guaranteed to be nonoverlapping and in increasing order. Two lemmata will aid the proof.

a0

✭

✁

✁

✁

✝

✁

✄

✝

✄

✁

✠

✁

✄

✠

Lemma 20 Let ✜✭ and be two nonoverlapping nonzero components of , with and ❁✭ . Let be a correctly rounded approximation to ❁✭ , and let be a two-component expansion exactly equal to . (Such an expansion is produced by Line 3, but here is defined also for ✻✧ .) Then is too small in magnitude to overlap the double-width product . Furthermore, if ✜✭ and are nonadjacent, then is not adjacent to .

✁

a0

✄

✜✭ ✠✭ It follows that is an integer, and ✜✭ . The latter fact and exact rounding imply that . Hence, and do not overlap.

✄

✆

✝

✝

✭

Lemma 21 For some , let be the smallest integer such that ✆✭ ✁a0 (hence ✠✭ does not overlap ✂a0 ). Then , and thus ✖✘✗ ulp ✏✝ .

✄

✭

✖

✁

✖

a0

a0

✄

✖

✄

✖

✖

✯

✖

✧

✔

✄

✤

✤

✁

✁

✝

✁

✭

✁

✝

✄

Proof: By scaling and by appropriate powers of (thereby shifting their exponents without changing their significands), one may assume without loss of generality that and are integers with magnitude less than ✔✱ , and that (and hence a radix point falls between and ).

✄

✄

a0

✱

✖

✁

a0

✁

✔

✖

✁

✖

✁

✭

✖

✗

✖

✁

✁

✝

✖

If ✜✭ and are nonadjacent, scale so that is an integer and ✜✭ . Then , so and are not adjacent.

✖

a0

✖

✗

a0

✖

✖

✯

✖

✠

a0

a0

✆

✖

✁

✖

a0

a0

a0

Proof: The inequality ✖✡✗ ✁a0 holds for after Line 1 is executed even if is rounded to a larger magnitude, because , and is expressible in bits. For larger values of , the bound is proven by induction. Assume that is the smallest integer such that ✆✭ ✄a0 ✆☎ ; by the inductive hypothesis, ☛✌✄ . ✝☎

✤

a0

a0

a0

☛

a0

a0

✁

a0

☛

✖

a0

✭

✖

✗

✖

✄

✖

✜✭ ✠✭ Lines 3, 4, and 5 compute , an approximation of ☛✌✄ , and are subject to roundoff error in Lines 4 and 5. Suppose that ☛✌✄ and ✠✭ have the same sign, that ☛✌✄ has its largest possible value , and that ✠✭ has its largest possible value . For these assignments, roundoff does not occur in Lines 4 and 5, and ✿✧ ☛✌✄ ✜✭ ✁a0 . Otherwise, roundoff may occur, but the monotonicity of floating-point multiplication and addition ensures that cannot be larger than . ✁a0

☛

a0

✖

✄

✖

✖

✁

✖

✄

✄

✟

The inequality ✖✌✗ ulp is guaranteed by exact rounding because is the roundoff term associated with the computation of in Line 5.

a0

✄

✭

✟

✂

✝

✄

✭

✝

☛

✤

✧

✂

✭

✝

✁

✄

✯

✧

✔

✭

✤

✖

✭

✤

✤

✆

✄

✄

✝

Proof of Theorem 19: One can prove inductively that at the end of each iteration of the for loop, the invariant holds. Certainly this invariant holds for after Line 1 is executed. By induction on Lines 3, 4, and 5, one can deduce that the invariant holds for all (relevant values of) . (The use of F AST -T WO -S UM in Line 5 will be justified shortly.) Thus, after Line 6 is executed, ✧✰✄ .

✁

✄

✠

✝

✝

✭

✝

✭

✁

✄

✤

✭

✄

✭

✝

✁

✝

a0

✭

✟

✁

✭

✁

✯

a0

✞

✖

✁

✁

✖

a0

✖

✁

✝

✖

✁

✁

a0

✁

✁

✖

✁

✄

✯

✔

✠

✂

✝

✄

✯

✠

I shall prove that the components of are nonoverlapping by showing that each time a component of is written, that component is smaller than and does not overlap either the accumulator nor any of the remaining products ( ); hence, the component cannot overlap any portion of their sum. The first claim,

✄

✝

☎

✭

✤

✝

a0

✄

✁

✂

✄

✖

✁

✤

✖

✤

a0

☎

Because ✠✭ and ✠✭ are nonoverlapping, ✜✭ must be a multiple of . Suppose that is the smallest integer such that ; then .

☎

a0

✄

✭

✁

a0

☎

✄

✭

✟

✭

✄

✖

a0

✖

✝

☎

✝

✖

✁

✤

✖

✠

a0

✁

✖

✁

✝

✄

a0

✤

✝

☛

✭

✖

✖

a0

a0

✄

✭

☛

✖

a0

✤

a0

✄

✭

✄

☛

✭

✖

✁

a0

✖

✖

a0

a0

✤

☛

a0

✗

✖

✁

✤

✄

✄

a0

✖

✠

✧

✄

✄

a0

☎

✁

a0

✖

✄

a0

✄

✖

a0

✖

a0

✄

✭

✖

✂

✖

✄

✭

☛

✠

a0

✄

a0

✖

☛

✄

✖

✁

✖

a0

✖

a0

✗

a0

a0

✭

✯

✁

✭

✁

✖

a0

✄

✄

☛

a0

a0

✧

a0

✤

✤

✳

that each component does not overlap the accumulator , is true because is the roundoff error incurred while computing .

✠

✤

✄

✭

✭

✁

✖

✖

✖

✖

✠

a0

✝

✝

✝

✠

✠

To show that each component of is smaller than and does not overlap the remaining products, I shall consider , the remaining odd components of , and the even components of separately. The component , computed by Line 1, does not overlap the remaining products ( ✄✆✎ ✟✁ ✄✆✎✠☎✠☎✠☎ ) by virtue of Lemma 20. The even components, which are computed by Line 4, do not overlap the remaining products because, by application of Lemma 1 to Line 4, a component ☛✌✄ is no larger than , which is bounded in turn by Lemma 20.

Odd components of , computed by Line 5, do not overlap the remaining products by virtue of Lemma 21, which guarantees that ✖✒✗ ulp ✏✝ . The remaining products are all multiples of ulp ✏✝ (because the remaining components of are multiples of ).

✄

☛

✭

✄

✄

✤

a0

✖

✆

✆

✠

If round-to-even tiebreaking is used, the output of each T WO -S UM , F AST -T WO -S UM , and T WO -P RO -DUCT statement is nonadjacent. If is nonadjacent as well, the arguments above are easily modified to show that is nonadjacent.

✖

✖

✁

✭

✁

a0

✗

✰✲ ✰✫ Note that if an input component ✜✭ is zero, then two zero output components are produced, and the accumulator value is unchanged ( ☛✌✄ ).

✁

The use of F AST -T WO -S UM in Line 5 is justified because (except if , in which case F AST -T WO -S UM still works correctly). To see this, recall that ✆✭ is a multiple of ✝☎ (with defined as in Lemma 21), and consider two cases: if ❁✭ , then is computed exactly and ✭✣✧ , so ✪✧ ✆☎ ✬✫ ☛✌✄ . If ✜✭ is larger than ✝☎ , it is at least twice as large, and hence is at least ☛✌✄ , so even if roundoff occurs and is not zero, ☛✌✄ .

a0

✄

✄

✁

The following corollary demonstrates that S CALE -E XPANSION is compatible with F AST -E XPANSION -S UM .

✭

✧

✭

a0

✭

Corollary 22 If is strongly nonoverlapping and round-to-even tiebreaking is used, then is strongly nonoverlapping.

✁

✁

✁

✠

✁

Proof: Because is nonoverlapping, is nonoverlapping by Theorem 19. We have also seen that if is nonadjacent, then is nonadjacent and hence strongly nonoverlapping; but is only guaranteed to be strongly nonoverlapping, and may deviate from nonadjacency.

a0

✁

✁

✡

✁

✳

✁

✤

✡

✳

✤

✁

☛

✤

✠

✁

✳

✄

✠

☎

a0

✡

✠

✠

Suppose two successive components ✜✭ and ✜✭ are adjacent. By the definition of strongly nonoverlapping, ✜✭ and ✠✭ are both powers of two and are not adjacent to ❁✭ or ✜✭ . Let be the integer satisfying and . For these components the multiplication of Line 3 is exact, so , ☛✡ , and . Applying Lemma 1 to Line 4, ☛✌✄ ✭✻✧ . However, the components and may cause difficulty (see Figure 14). We know is nonoverlapping, but can these two components be adjacent to their neighbors or each other?

✠

✤

✤

✭

✤

✠

✧

✤

✭

✳

✤

✧

✭

✭

✳

✤

✧

✠

✄

✭

☛

✤

✤

✄

✠

✄

✭

✭

✳

✧

The arguments used in Theorem 19 to prove that is nonadjacent, if is nonadjacent and round-toeven tiebreaking is used, can be applied here as well to show that and are not adjacent to any components of produced before or after them, but they may be adjacent to each other. Assume that and are adjacent (they cannot be overlapping).

a0

✄

✭

✳

✤

✠

✭

✳

✤

✄

✭

✳

✤

✠

✄

✭

✭

✳

✤

✧

✗

✄

is computed in Line 5 from and . The latter addend is equal to , because . is not adjacent to , because they are produced in Line 5 from a F AST -T WO -S UM operation. Hence, the least significant nonzero bit of (that is, the bit that causes it to be adjacent to ) must have come

✠

✄

✭

☛

✄

✭

✳

✠

☎

✭

a0

✫

✁

✄

✭

☛

✤

a0

✭

✧

✗

✠

✁

✖

a0

✖

✖

a0

✖

a0

✄

a0

☛

a0

✤

✁

✄

✁

✭

✖

a0

✖

a0

✠

✄

✭

✖

✄

✖

✖

✖

a0

✭

✖

✠

✧

a0

✖

✝

a0

✳

✤

✭

✤

✖

✠

✖

✁

a0

✖

✁

✭

✖

a0

✭

✖

✖

a0

✄

✭

✖

✟

✖

✁

✭

✖

✖

✭

✄

✭

✧

✠

a0

✭

✳

a0

✤

✳

✭

✳

✤

✧

✁

✠

✄

a0

✭

☛

✳

✁

✤

✤

✗

✄

✭

✳

a0

✤

✁

✠

a0

✁

✠

a0

✠

✄

✁

✠

✠

✄

a0

✗

a0

✄

✭

✄

☛

✭

☛

✤

✤

✖

✁

✭

a0

✧

✭

✄

a0

☛

✄

✠

✖

✭

✠

<!-- image -->

✠

✄

✭

✧

☛✡

✭

✳

✗

✄

✭

☛

✠

✄

✭

✧

☛✌✄

✧

✗

✠

✄

✭

✳

Figure 14: An adjacent pair of one-bit components in a strongly nonoverlapping input expansion may cause S CALE -E XPANSION to produce an adjacent pair of one-bit components in the output expansion.

✭

✳

✖

✠

a0

✖

✗

✤

✄

✡

✆

✄

✭

✳

✖

✠

✠

✖

✡

a0

✆

✳

✤

✆

✄

✖

✁

✳

✠

✖

a0

a0

✳

✄

✤

✤

from , which is equal to ✑✳ . It follows that is a multiple of ☛✡ ulp ✏✝ . Because ✜✭ ☛✡ , Lemma 21 implies that ulp ✏✝ . Hence, ulp ✏✝ .

✄

✤

✖

✗

✆

✄

✭

✳

✤

Similarly, because ❁✭ ☛✡ ✑✳ , Lemma 21 implies that ☛✡ ulp ✏✝ . The components and can only be adjacent in the case ulp ✏✝ . In this case, both components are expressible in one bit.

✭

☛

✤

✖

✁

✖

✤

✖

✠

✄

✭

☛

✤

✖

✧

✡

✆

✄

Hence, each adjacent pair of one-bit components in the input can give rise to an isolated adjacent pair of one-bit components in the output, but no other adjacent components may appear. If is strongly nonoverlapping, so is .

✠

## 2.7 Compression and Approximation

The algorithms for manipulating expansions do not usually express their results in the most compact form. In addition to the interspersed zero components that have already been mentioned (and are easily eliminated), it is also common to find components that represent only a few bits of an expansion's value. Such fragmentation rarely becomes severe, but it can cause the largest component of an expansion to be a poor approximation of the value of the whole expansion; the largest component may carry as little as one bit of significance. Such a component may result, for instance, from cancellation during the subtraction of two nearly equal expansions.

The C OMPRESS algorithm below finds a compact form for an expansion. More importantly, C OMPRESS guarantees that the largest component is a good approximation to the whole expansion. If round-to-even tiebreaking is used, C OMPRESS also converts nonoverlapping expansions into nonadjacent expansions.

Priest [23] presents a more complicated 'Renormalization' procedure that compresses optimally. Its greater running time is rarely justified by the marginal reduction in expansion length, unless there is a need to put expansions in a canonical form.

✁

✂

☎

✝

✁

✝

✫

Theorem 23 Let ✜✭ be a nonoverlapping expansion of -bit components, where . Suppose that the components of are sorted in order of increasing magnitude, except that any of the may

✧

✭

✤

✁

✝

✂

✁

✭

✛

✁

✭

a0

a0

✡

✠

✤

a0

a0

✄

✭

✳

✄

✁

a0

✭

✳

✤

a0

a0

✧

✄

a0

✭

✳

✡

✤

✁

✳

✤

✧

✁

✁

✁

a0

✤

a0

a0

✤

✧

a0

✄

✗

✄

✭

✁

a0

✭

✧

a0

a0

a0

✡

✄

✄

✭

☛

✤

✁

✁

✁

✁

✁

a0

✡

a0

a0

a0

✗

✄

✄

✭

✄

✁

✭

a0

✳

✤

a0

a0

✳

✤

✄

✁

✤

✄

✭

✳

✤

✄

✧

✁

☛

a0

✳

✤

a0

✄

✤

✄

✠

✤

✆

✁

✄

✁

✠

✠

✠

✠

Figure 15: Operation of C OMPRESS when no zero-elimination occurs.

<!-- image -->

✠

✧

☎

✗

✆

✠

✝

✠

✭

✧

✧

✭

✩

✂

✠

✠

```
C OMPRESS ☛✁ 1 2 ✂✁ ☎✝ 3 for downto 4 ☞a0 ☎✄ F AST -T WO -S UM ☞a0 ✟✁ ✜✭ 5 if then 6 ✝✆✟✞✠✞✡✆ 7 ✂✁ ✂✁ 8 ☛✄ 9 ✝✆✟✞✠✞✡✆ 10 ✑✂ ✌☞ 11 for ☞✁ to 12 ☞a0 ☎✄ F AST -T WO -S UM ✁a0 ✘✭ ✔✝ 13 if then 14 ✞✡✆ 15 ✑✂ ✌☞ ✑✂ 16 ✞✡✆ ✌a0 17 Set (the length of ) to ✑✂ 18 return
```

✱

☞

✞

✠

✁

✁

✠

Figure 15 illustrates the operation of C OMPRESS . For clarity, and are presented as two separate arrays in the C OMPRESS pseudocode, but they can be combined into a single working array without conflict by replacing every occurrence of ' ' with ' '.

a0

✠

✠

Proof Sketch: C OMPRESS works by traversing the expansion from largest to smallest component, then back

a0

✭

✤

✠

✭

✠

✠

be zero. Then the following algorithm will produce a nonoverlapping expansion (nonadjacent if round-toeven tiebreaking is used) such that ✞✝ ✄✁ , where the components are in order of increasing magnitude. If , none of the will be zero. Furthermore, the largest component approximates with an error smaller than ulp .

✄

a0

✁

☞

✁

✁

✁

☎

☞

✝

✩

✠

✁

✁

✁

a0

✁

✁

✁

a0

a0

✁

✁

✆

✩

✁

✁

a0

a0

✁

✠

✆

✁

✁

a0

a0

✯

✯

✆

☎

✆

✆

☞

☞

✝

✄

✄

✔

☞

✎

✎

✝

✄

✁

✁

✧

✧

☎

☎

✁

a0

✁

✄

✠

a0

✝

✝

✁

a0

✁

☛

✁

✁

✗

✗

☞

☞

✁

✁

☞

✱

✁

✝

✔

✁

☞

☎

✝

✟

✁

☞

a0

✁

☞

✔

✁

✁

a0

✟

✄

a0

✁

✝

✁

✔

✔

✁

a0

✁

a0

✝

☛

✁

✁

✔

✄

✆

✆

✎

✎

a0

✝

✁

✁

a0

✄

a0

a0

✁

✁

✤

a0

✤

✄

✄

✄

✤

✗

✆

from smallest to largest, replacing each adjacent pair with its two-component sum. The first traversal, from largest to smallest, does most of the compression. The expansion ✰✫✠✫✠✫ ✂✟ ✝✆✟✞✠✞✡✆ produced by Lines 1 through 9 has the property that ulp ✁a0 for all (and thus successive components overlap by at most one bit). This fact follows because the output of F AST -T WO -S UM in Line 4 has the property that ulp ☞a0 ✔✝ , and the value of thus produced can only be increased slightly by the subsequent addition of smaller nonoverlapping components.

a0

✤

✭

a0

The second traversal, from smallest to largest, clips any overlapping bits. The use of F AST -T WO -S UM in Line 12 is justified because the property that ulp ✁a0 ✘✭ guarantees that (the sum of the components that are smaller than ✚✭ ) is smaller than ✚✭ . The expansion ✞✡✆ ✞✡✆ ✫✠✫✠✫ ✌✟ is nonoverlapping (nonadjacent if round-to-even tiebreaking is used) because F AST -T WO -S UM produces nonoverlapping (nonadjacent) output.

✩

✤

✄

☛

✩

✩

✩

✤

✤

✠

✠

✠

✆

✝

✖

✠

✠

✠

✠

✠

✄

✠

✤

✖

✠

To ensure that is a good approximation to , only the second traversal is necessary; however, the first traversal is more effective in reducing the number of components. The fastest way to approximate is to simply sum its components from smallest to largest; by the reasoning used above, the result errs by less than one ulp. This observation is the basis for an A PPROXIMATE procedure that is used in the predicates of Section 4.

✗

✆

✝

✠

✱

✠

✱

☛

✤

a0

✠

✄

✠

✤

During the second traversal, an approximate total is maintained in the accumulator . The component is produced by the last F AST -T WO -S UM operation that produces a roundoff term; this roundoff term is no greater than ulp . Hence, the sum ☛✌✄ ✫✠✫✠✫ (where the components of are nonoverlapping) is less than ulp ✌✝ , therefore ✟✖ ulp .

✖

☛

✩

✠

✠

✩

✩

☛

✩

✠

✠

✆

✖

✆

✩

✝

✁

✭

Theorem 23 is not the strongest statement that can be made about C OMPRESS . C OMPRESS is effective even if the components of the input expansion have a certain limited amount of overlap. Furthermore, the bound for ✟✖ is not tight. (I conjecture that the largest possible relative error is exhibited by a number that contains a nonzero bit every th bit; observe that ulp ✞✝ ulp ✞✝ ✫✠✫✠✫ cannot be further compressed.) These improvements complicate the proof and are not explored here.

## 2.8 Other Operations

✟

✆

✝

Distillation is the process of summing unordered -bit values. Distillation can be performed by the divideand-conquer algorithm of Priest [23], which uses any expansion addition algorithm to sum the values in a tree-like fashion as illustrated in Figure 16. Each -bit addend is a leaf of the tree, and each interior node represents a call to an expansion addition algorithm. If E XPANSION -S UM is used (and zero elimination is not), then it does not matter whether the tree is balanced; distillation will take precisely ✁a0 ❂☛✰✔ ✞✝ T WO -S UM operations, regardless of the order in which expansions are combined. If F AST -E XPANSION -S UM is used, the speed of distillation depends strongly on the balance of the tree. A well-balanced tree will yield an ✁a0✄✂✆☎✞✝✟a0 distillation algorithm, an asymptotic improvement over distilling with E XPANSION -S UM . As I have mentioned, it is usually fastest to use an unrolled E XPANSION -S UM to create expansions of length four, and F AST -E XPANSION -S UM with zero elimination to sum these expansions.

✁

a0

✎

a0

✄

a0

To find the product of two expansions and , use S CALE -E XPANSION (with zero elimination) to form the expansions ✟✁ , then sum these using a distillation tree.

✎✠☎✠☎✠☎ Division cannot always, of course, be performed exactly, but it can be performed to arbitrary precision by an iterative algorithm that employs multiprecision addition and multiplication. Consult Priest [23] for one such algorithm.

a0

✂

✔

✟

✤

✆

✔

✟

✤

✡

✆

✔

✒

✄

✟

☛

✟

a0

✟

✟

✟

a0

☛

✟

✟

✟

✠

✤

a0

✝

✁

☛

✤

✗

✂

✂

✆

✝

✝

✄

✞

a0

☎

✟

a0

✆

☎

☛

✤

✟

a0

a0

✁

✄

a0

☎

✤

✆

Figure 16: Distillation of sixteen -bit floating-point values.

<!-- image -->

✧

The easiest way to compare two expansions is to subtract one from the other, and test the sign of the result. An expansion's sign can be easily tested because of the nonoverlapping property; simply check the sign of the expansion's most significant nonzero component. (If zero elimination is employed, check the component with the largest index.) A nonoverlapping expansion is equal to zero if and only if all its components are equal to zero.

## 3 Adaptive Precision Arithmetic

## 3.1 Why Adaptivity?

Exact arithmetic is expensive, and when it can be avoided, it should be. Some applications do not need exact results, but require the absolute error of a result to fall below some threshold. If this threshold is known before the computation is performed, it is economical to employ adaptivity by prediction . One writes several procedures, each of which approximates the result with a different degree of precision, and with a correspondingly different speed. Error bounds are derived for each of these procedures; these bounds are typically much cheaper to compute than the approximations themselves, except for the least precise approximation. For any particular input, the application computes the error bounds and uses them to choose the procedure that will attain the necessary accuracy most cheaply.

Sometimes, however, one cannot determine whether a computation will be accurate enough before it is done. An example is when one wishes to bound the relative error, rather than the absolute error, of the result. (A special case is determining the sign of an expression; the result must have relative error less than one.) The result may prove to be much larger than its error bound, and low precision arithmetic will suffice, or it may be so close to zero that it is necessary to evaluate it exactly to satisfy the bound on relative error. One cannot generally know in advance how much precision is needed.

In the context of determinant evaluation for computational geometry, Fortune and Van Wyk [11] suggest using a floating-point filter. An expression is evaluated approximately in hardware precision arithmetic first. Forward error analysis determines whether the approximate result can be trusted; if not, an exact result is computed. If the exact computation is only needed occasionally, the application is slowed only a little.

One might hope to improve this idea further by computing a sequence of increasingly accurate results, testing each one in turn for accuracy. Alas, whenever an exact result is required, one suffers both the cost of the exact computation and the additional burden of computing several approximate results in advance. Fortunately, it is often possible to use intermediate results as stepping stones to more accurate results; work already done is not discarded but is refined.

## 3.2 Making Arithmetic Adaptive

F AST -T WO -S UM , T WO -S UM , and T WO -P RODUCT each have the feature that they can be broken into two parts: Line 1, which computes an approximate result, and the remaining lines, which calculate the roundoff error. The latter, more expensive calculation can be delayed until it is needed, if it is ever needed at all. In this sense, these routines can be made adaptive , so that they only produce as much of the result as is needed. I describe here how to achieve the same effect with more general expressions.

✥

✭

✯

✭

✥

✭

✥

✭

Any expression composed of addition, subtraction, and multiplication operations can be calculated adaptively in a manner that defines a natural sequence of intermediate results whose accuracy it is appropriate to test. Such a sequence is most easily described by considering the tree associated with the expression, as in Figure 17(a). The leaves of this tree represent floating-point operands, and its internal nodes represent operations. Replace each node whose children are both leaves with the sum , where represents the approximate value of the subexpression, and represents the roundoff error incurred while calculating , as illustrated in Figure 17(b). Expand the expression to form a polynomial.

✝

✯

☛

✄

✝

✄

✟

<!-- formula-not-decoded -->

✍

✄

✍

✟

a0

✄

☛

✁

✖

✯

✭

✖

✗

✖

✖

a0

✤

☛

✆

✔

✝

Expression 5 can be divided into three parts, having magnitudes of ✞✝ , ✠✝ , and ✠✝ , respectively. Denote these parts , , and . More generally, for any expression expanded in this manner, let be the sum of all products containing of the variables, so that has magnitude ✠✝ .

✝

✯

✭

In the expanded expression, the terms containing many occurrences of variables (roundoff errors) are dominated by terms containing fewer occurrences. As an example, consider the expression ✆☞☛ ✁a0 ✂a0 ✆☞☛ ☎✄ ✕☛ ✂✄ (Figure 17), which calculates the square of the distance between two points in the plane. Set ☎a0 ✍☛ ✆a0 ✕✧✏✥ and ☎✄ ✻☛✓✄ ✂✄ ✧✲✥ . The resulting expression, expanded in full, is

✥

✤

✥

✄

✄

✟

✄

✟

✄

✄

✤

✤

✄

✟

✄

✟

a0

✟

a0

✄

It is significant that each is small relative to its corresponding . Using standard terminology from forward error analysis [28], the quantity ulp ✞✝ is called the machine epsilon , denoted . Recall that exact rounding guarantees that ✞✝ ✥✪✭ ; the quantity bounds the relative error err ✆☞☛ ✏✝ ✆☞☛ ✏✝ of any basic floating-point operation. Note that ✍✧ . In IEEE 754 double precision arithmetic, ✍✧ ; in single precision, a0☞☛✌✄✝✆ .

a0

✟

✔

✟

✟

✄

✥

✤

✯

✤

✥

✯

✯

✤

✯

a0

✁

✟

☛

✄

✄

✄

✝

✧

✟

✯

a0

✞

a0

☛

✤

✯

a0

✭

✯

✆

✄

✝

✄

✟

✯

✯

✝

✄

✄

✱

✆

✭

✝

✆

✥

✭

✆

✝

✟

✆

✆

✭

✝

✝

✝

✆

✝

✭

<!-- image -->

✁

☎

Figure 17: (a) Formula for the square of the distance between two points and . (b) The lowest subexpressions in the tree are expressed as the sum of an approximate value and a roundoff error. (c) A simple incremental adaptive method for evaluating the expression. The approximations and are generated and tested in turn. The final expansion ✡a0 is exact. Each includes all terms of size ✄✂ ✡✠ or larger, and hence has error no greater than ☎✂ . (d) Incremental adaptivity taken to an extreme. The three subexpression trees ✆✞✝ , , and are themselves calculated adaptively. Each incorporates only the terms needed to reduce its error to ☎✂ ✖✍ .

a0

✝

a0

✝

✞

✠

✎

☛

a0

☛

The approximation is not the way to achieve an error bound of ✠✝ with the least amount of work. For instance, a floating-point calculation of using no exact arithmetic techniques will achieve an ✠✝ error bound, albeit with a larger constant than the error bound for . Experimentation has shown that the fastest adaptive predicates are written by calculating an approximation having bound ✠✝ as quickly as possible, then moving on to the next smaller order of magnitude. Improvements in the constant prefacing each error bound will make a difference in only a small number of cases. Hence, I will consider two modifications to the technique just described. The first modification computes each error bound from the minimum possible number of roundoff terms. This lazy approach is presented here for instructional

✠

✝

✝

✟

✟

✆

✝

✞

✆

✁

✏

✍

✛

✗

✆

☛

✟

✍

One can obtain an approximation with error no larger than ✠✝ by computing exactly the sum of the first terms, through . The sequence ✡✠ ✎✠☎✠☎✠☎ of increasingly accurate approximations can be formed incrementally; is the exact sum of and . Members of this sequence are generated and tested, as illustrated in Figure 17(c), until one is sufficiently accurate.

✠

✆

✝

✟

✠

✝

✁

☛

✏

✤

✛

a0

✝

✠

✤

✤

✥

✄

✄

✤

a0

✟

✍

✥

✤

✄

✄

✆

✝

✝

✆

✝

✝

a0

✠

✗

✤

✁

✏

✍

a0

☛

✗

✛

✟

✆

✝

✝

purposes, but is not generally the fastest. The second modification I will consider, and the one I recommend for use, is faster because it spends less time collating small data.

a0

✞

✤

✄

a0

✝

The first modification is to compute the subexpressions , , and adaptively as well. The method is the same: replace each bottom-level subexpression of (and and ) with the sum of an approximate result and an error term, and expand into a sum of terms of differing order. An approximation having an error bound of magnitude ✠✝ may be found by approximating each term with error ✠✝ . Because the term has magnitude at most ✠✝ , it need not be approximated with any better relative error than ✠✝ .

✟

✝

✆

✝

✁

✟

☛

✁

a0

✆

✝

✆

✝

Figure 17(d) shows that the method is as lazy as possible, in the sense that each approximation uses only the roundoff terms needed to obtain an ✠✝ error bound. (Note that this is true at every level of the tree. It is apparent in the figure that every roundoff term produced is fed into a different calculation than the larger term produced with it.) However, the laziest approach is not necessarily the fastest approach. The cost of this method is unnecessarily large for two reasons. First, recall from Section 2.8 that F AST -E XPANSION -S UM is most effective when terms are summed in a balanced manner. The additions in Figure 17(d) are less well balanced than those in Figure 17(c). Second, and more importantly, there is a good deal of overhead for keeping track of many small pieces of the sum; the method sacrifices most of the advantages of the compressed form in which expansions are represented. Figure 17(d) does not fully reveal how convoluted this extreme form of adaptivity can become for larger expressions. In addition to having an unexpectedly large overhead, this method can be exasperating for the programmer.

✆

✝

✝

✠

✤

✎

✟

✄

✆

✝

✝

✟

✆

✝

✳

✝

✠

✞

✧

✗

a0

✝

✠

✝

a0

✝

✆

✝

✝

a0

✝

The final method for incremental adaptivity I shall present, which is used to derive the geometric predicates in Section 4, falls somewhere between the two described above. As in the first method, compute the sequence ✡✠ ✎✠☎✠☎✠☎ , and define also . We have seen that the error bound of each term may be improved from ✠✝ to ✠✝ by (exactly) adding to it. However, because the magnitude of itself is ✠✝ , the same effect can be achieved (with a slightly worse constant in the error bound) by computing with floating-point arithmetic and tolerating the roundoff error, rather than computing exactly. Hence, an approximation having an ✠✝ error bound is computed by summing and an inexpensive correctional term , which is merely the floating-point approximation to , as illustrated in Figure 18. is nearly as accurate as but takes much less work to compute. If is not sufficiently accurate, then it is thrown away, and the exact value of is computed and added to to form . This scheme reuses the work done in performing exact calculations, but does not reuse the correctional terms. (In practice, no speed can be gained by reusing the correctional terms.)

✤

✤

✳

✠

✁

✳

✤

a0

✝

✠

✝

✠

✝

✳

✤

The first value ( ) computed by this method is an approximation to ; if is sufficiently accurate, it is unnecessary to compute the terms, or use any exact arithmetic techniques, at all. (Recall that the terms are more expensive to compute than the terms.) This first test is identical to Fortune and Van Wyk's floating-point filter.

✤

✯

✯

✥

This method does more work during each stage of the computation than the first method, but typically terminates one stage earlier. It is slower when the exact result must be computed, but is faster in applications that rarely need an exact result. In some cases, it may be desirable to test certain members of both sequences and for accuracy; the predicates defined in Section 4 do so.

All three methods of making expressions adaptive are mechanical and can be automated. An expression compiler that converts expressions into code that evaluates these expressions adaptively, with automatically computed error bounds, would be valuable. (Fortune and Van Wyk [12] have produced such a compiler for integer operands, using straightforward floating-point filters instead of the more complicated adaptive methods described here. Their expression compiler is discussed in the next section.)

✠

✁

✞

a0

✝

✆

✟

✝

✁

✁

✝

✳

✤

✝

✟

✤

✝

✟

✟

✆

✞

✝

✁

✳

✤

✝

a0

✞

a0

a0

a0

✤

a0

✝

a0

a0

✝

✄

a0

a0

✁

✠

✝

a0

✝

✟

✝

✁

✝

✳

✤

<!-- image -->

a0

Figure 18: An adaptive method of intermediate complexity that is frequently more efficient than the other two. Each achieves an ✄✂ error bound by adding an inexpensive correctional term (labeled 'ct') to ✡✠ .

a0

✍

✁

✏

✍

✛

✍

✗

The reader may wonder if writing an expression in sum-of-products form isn't inefficient. In ordinary floating-point arithmetic it often is, but it seems to make little difference when using the exact arithmetic algorithms of Section 2. Indeed, the multiplication operation described in Section 2.8 multiplies two expansions by expanding the product into sum-of-products form.

These ideas are not exclusively applicable to the multiple-component approach to arbitrary precision arithmetic. They will work with multiple-digit formats as well, though the details differ.

## 4 Implementation of Geometric Predicates

## 4.1 Related Work in Robust Computational Geometry

Most geometric algorithms are not originally designed for robustness at all; they are based on the real RAM model , in which quantities are allowed to be arbitrary real numbers, and all arithmetic is exact. There are several ways a geometric algorithm that is correct within the real RAM model can go wrong in an encounter with roundoff error. The output might be incorrect, but be correct for some perturbation of its input. The result might be usable yet not be valid for any imaginable input. Or, the program may simply crash or fail to produce a result. To reflect these possibilities, geometric algorithms are divided into several classes with varying amounts of robustness: exact algorithms , which are always correct; robust algorithms , which are always correct for some perturbation of the input; stable algorithms , for which the perturbation is small; quasi-robust algorithms , whose results might be geometrically inconsistent, but nevertheless satisfy some weakened consistency criterion; and fragile algorithms , which are not guaranteed to produce any usable output at all. The next several pages are devoted to a discussion of representative research in each class, and of the circumstances in which exact arithmetic and other techniques are or are not applicable. For more extensive surveys of geometric robustness, see Fortune [9] and Hoffmann [15].

Exact algorithms. A geometric algorithm is exact if it is guaranteed to produce a correct result when given an exact input. (Of course, the input to a geometric algorithm may only be an approximation of some realworld configuration, but this difficulty is ignored here.) Exact algorithms use exact arithmetic in some form, whether in the form of a multiprecision library or in a more disguised form.

There are several exact arithmetic schemes designed specifically for computational geometry; most are methods for exactly evaluating the sign of a determinant, and hence can be used to perform the orientation and incircle tests. Clarkson [6] proposes an algorithm for using floating-point arithmetic to evaluate the sign of the determinant of a small matrix of integers. A variant of the modified Gram-Schmidt procedure is used to improve the conditioning of the matrix, so that the determinant can subsequently be evaluated safely by Gaussian elimination. The 53 bits of significand available in IEEE double precision numbers are sufficient to operate on matrices of 32-bit integers. Clarkson's algorithm is naturally adaptive; its running time is small for matrices whose determinants are not near zero .

a0

✁

a0

✛

✁

✛

✂

✂

✆

✂

✟

✔

✔

✗

✁

✔

✗

✄

Recently, Avnaim, Boissonnat, Devillers, Preparata, and Yvinec [1] proposed an algorithm to evaluate signs of determinants of and matrices of -bit integers using only and ✞✝ -bit arithmetic, respectively. Surprisingly, this is sufficient even to implement the insphere test (which is normally written as a or determinant), but with a handicap in bit complexity; 53-bit double precision arithmetic is sufficient to correctly perform the insphere test on points having 24-bit integer coordinates.

a0

✖

✁

✖

✂

✁

✂

Fortune and Van Wyk [12, 11] propose a more general approach (not specific to determinants, or even to predicates) that represents integers using a standard multiple-digit technique with digits of radix ✄☎✁ stored as double precision floating-point values. (53-bit double precision significands make it possible to add several products of 23-bit integers before it becomes necessary to normalize.) Rather than use a general-purpose arbitrary precision library, they have developed LN, an expression compiler that writes code to evaluate a specific expression exactly. The size of the operands is arbitrary, but is fixed when LN is run; an expression can be used to generate several functions, each for arguments of different bit lengths. Because the expression and the bit lengths of all operands are fixed in advance, LN can tune the exact arithmetic aggressively, eliminating loops, function calls, and memory management. The running time of a function produced by LN depends on the bit complexity of the inputs. Fortune and Van Wyk report an order-of-magnitude speed improvement over the use of multiprecision libraries (for equal bit complexity). Furthermore, LN gains another speed improvement by installing floating-point filters wherever appropriate, calculating error bounds automatically.

Karasick, Lieber, and Nackman [16] report their experiences optimizing a method for determinant evaluation using rational inputs. Their approach reduces the bit complexity of the inputs by performing arithmetic on intervals (with low precision bounds) rather than exact values. The determinant thus evaluated is also an interval; if it contains zero, the precision is increased and the determinant reevaluated. The procedure is repeated until the interval does not contain zero (or contains only zero), and the result is certain. Their approach is thus adaptive, although it does not appear to use the results of one iteration to speed the next.

Because the Clarkson and Avnaim et al. algorithms are effectively restricted to low precision integer coordinates, I do not compare their performance with that of my algorithms, though theirs may be faster.

The method presented in Clarkson's paper does not work correctly if the determinant is exactly zero, but Clarkson (personal communication) notes that it is easily fixed. 'By keeping track of the scaling done by the algorithm, an upper bound can be maintained for the magnitude of the determinant of the matrix. When that upper bound drops below one, the determinant must be zero, since the matrix entries are integers, and the algorithm can stop.'

a0

Floating-point inputs are more difficult to work with than integer inputs, partly because of the potential for the bit complexity of intermediate values to grow more quickly. (The Karasick et al. algorithm also suffers this difficulty, and is probably not competitive with the other techniques discussed here, although it may be the best existing alternative for algorithms that require rational numbers, such as those computing exact line intersections.) When it is necessary for an algorithm to use floating-point coordinates, the aforementioned methods are not currently an option (although it might be possible to adapt them using the techniques of Section 2). I am not aware of any prior literature on exact determinant evaluation that considers floatingpoint operands, except for one limited example: Ottmann, Thiemt, and Ullrich [22] advocate the use of an accurate scalar product operation, ideally implemented in hardware (though the software-level distillation algorithm described in Section 2.8 may also be used), as a way to evaluate some predicates such as the 2D orientation test.

✖

Exact determinant algorithms do not satisfy the needs of all applications. A program that computes line intersections requires rational arithmetic; an exact numerator and exact denominator must be stored. If the intersections may themselves become endpoints of lines that generate more intersections, then intersections of greater and greater bit complexity may be generated. Even exact rational arithmetic is not sufficient for all applications; a solid modeler, for instance, might need to determine the vertices of the intersection of two independent solids that have been rotated through arbitrary angles. Yet exact floating-point arithmetic can't even cope with rotating a square in the plane, because irrational vertex coordinates result. The problem of constructed irrational values has been partly attacked by the implementation of 'real' numbers in the LEDA library of algorithms [4]. Values derived from square roots (and other arithmetic operations) are stored in symbolic form when necessary. Comparisons with such numbers are resolved with great numerical care, albeit sometimes at great cost; separation bounds are computed where necessary to ensure that the sign of an expression is determined accurately. Floating-point filters and another form of adaptivity (approximating a result repeatedly, doubling the precision each time) are used as well.

✂

a0

For the remainder of this discussion, consideration is restricted to algorithms whose input is geometric (e.g. coordinates are specified) but whose output is purely combinatorial, such as the construction of a convex hull or an arrangement of hyperplanes.

Robust algorithms. There are algorithms that can be made correct with straightforward implementations of exact arithmetic, but suffer an unacceptable loss of speed. An alternative is to relax the requirement of a correct solution, and instead accept a solution that is 'close enough' in some sense that depends upon the application. Without exact arithmetic, an algorithm must somehow find a way to produce sensible output despite the fact that geometric tests occasionally tell it lies. No general techniques have emerged yet, although bandages have appeared for specific algorithms, usually ensuring robustness or quasi-robustness through painstaking design and error analysis. The lack of generality of these techniques is not the only limitation of the relaxed approach to robustness; there is a more fundamental difficulty that deserves careful discussion.

When disaster strikes and a real RAM-correct algorithm implemented in floating-point arithmetic fails to produce a meaningful result, it is often because the algorithm has performed tests whose results are mutually contradictory. Figure 19 shows an error that arose in a two-dimensional Delaunay triangulation program I wrote. The program, which employs a divide-and-conquer algorithm presented by Guibas and Stolfi [14], failed in a subroutine that merges two triangulations into one. The geometrically nonsensical triangulation in the illustration was produced.

On close inspection with a debugger, I found that the failure was caused by a single incorrect result of the incircle test. At the bottom of Figure 19 appear four nearly collinear points whose deviation from collinearity has been greatly exaggerated for clarity. The points , , , and had been sorted by their

☛

✄

✠

✁

☎

Figure 19: Top left: A Delaunay triangulation. Top right: An invalid triangulation created due to roundoff error. Bottom: Exaggerated view of the inconsistencies that led to the problem. The algorithm 'knew' that the point lay between the lines ✁a0 and , but an incorrect incircle test claimed that lay inside the circle .

<!-- image -->

✂

a0

☎

✁

✁

✂

✁

✥

-coordinates, and had been correctly established (by orientation tests) to lie below the line and above the line . In principle, a program could deduce from these facts that cannot fall inside the circle ✞✄ . Unfortunately, the incircle test incorrectly declared that lay inside, thereby leading to the invalid result.

✆

✄

☛

✠

☛

✁

☛

✁

✠

It is significant that the incircle test was not just wrong about these particular points; it was inconsistent with the 'known combinatorial facts.' A correct algorithm (that computes a purely combinatorial result) will produce a meaningful result if its test results are wrong but are consistent with each other, because there exists an input for which those test results are correct. Following Fortune [8], an algorithm is robust if it always produces the correct output under the real RAM model, and under approximate arithmetic always produces an output that is consistent with some hypothetical input that is a perturbation of the true input; it is stable if this perturbation is small. Typically, bounds on the perturbation are proven by backward error analysis. Using only approximate arithmetic, Fortune gives an algorithm that computes a planar convex hull that is correct for points that have been perturbed by a relative error of at most ✠✝ (where is defined as in Section 3.2), and an algorithm that maintains a triangulation that can be made planar by perturbing each vertex by a relative error of at most ✠✞ , where is the number of vertices. If it seems surprising that a 'stable' algorithm cannot keep a triangulation planar, consider the problem of inserting a new vertex so close to an existing edge that it is difficult to discern which side of the edge the vertex falls on. Only exact arithmetic can prevent the possibility of creating an 'inverted' triangle.

✟

✄

✝

✝

✞

One might wonder if my triangulation program can be made robust by avoiding any test whose result can be inferred from previous tests. Fortune [8] explains that

✎

[a]n algorithm is parsimonious if it never performs a test whose outcome has already been determined as the formal consequence of previous tests. A parsimonious algorithm is clearly robust, since any path through the algorithm must correspond to some geometric input; making an algorithm parsimonious is the most obvious way of making it robust. In principle it is possible to make an algorithm parsimonious: since all primitive tests are polynomial sign evaluations, the question of whether the current test is a logical consequence of previous tests can be phrased as a statement of the existential theory of the reals. This theory is at least NP-hard and is decidable in polynomial space [5]. Unfortunately, the full power of the theory seems to be necessary for some problems. An example is the line arrangement problem : given a set of lines (specified by real coordinates ✆☞☛ ✄✆✎ , so that ✿✥ ✄✑✯ ), compute the combinatorial structure of the resulting arrangement in the plane. It follows from recent work of Mnev [21] that the problem of deciding whether a combinatorial arrangement is actually realizable with lines is as hard as the existential theory of the reals. Hence a parsimonious algorithm for the line arrangement problem . . . seems to require the solution of NP-hard problems.

✠

✝

☛

✟

✧

✠

Because exact arithmetic does not require the solution of NP-hard problems, an intermediate course is possible; one could employ parsimony whenever it is efficient to do so, and resort to exact arithmetic otherwise. Consistency is guaranteed if exact tests are used to bootstrap the 'parsimony engine.' I am not aware of any algorithms in the literature that take this approach, although geometric algorithms are often designed by their authors to avoid the more obviously redundant tests.

Quasi-robust algorithms. The difficulty of determining whether a line arrangement is realizable suggests that, without exact arithmetic, robustness as defined above may be an unattainable goal. However, sometimes one can settle for an algorithm whose output might not be realizable. I place such algorithms in a bag labeled with the fuzzy term quasi-robust , which I apply to any algorithm whose output is somehow provably distinguishable from nonsense. Milenkovic [20] circumvents the aforementioned NP-hardness result while using approximate arithmetic by constructing pseudo-line arrangements; a pseudo-line is a curve

✟

✆

✝

✝

☛

✄

✁

✟

constrained to lie very close to an actual line. Fortune [10] presents a 2D Delaunay triangulation algorithm that constructs, using approximate arithmetic, a triangulation that is nearly Delaunay in a well-defined sense using the pseudo-line-like notion of pseudocircles. Unfortunately, the algorithm's running time is ✠✞ , which compares poorly with the ✠✞ ✂✆☎✞✝ time of optimal algorithms. Milenkovic's and Fortune's algorithms are both quasi-stable , having small error bounds. Milenkovic's algorithm can be thought of as a quasi-robust algorithm for line arrangements, or as a robust algorithm for pseudo-line arrangements.

✆

✝

✆

✞

✝

Barber [3] pioneered an approach in which uncertainty, including the imprecision of input data, is a part of each geometric entity. Boxes are structures that specify the location and the uncertainty in location of a vertex, edge, facet, or other geometric structure. Boxes may arise either as input or as algorithmic constructions; any uncertainty resulting from roundoff error is incorporated into their shapes and sizes. Barber presents algorithms for solving the point-in-polygon problem and for constructing convex hulls in any dimension. For the point-in-polygon problem, 'can't tell' is a valid answer if the uncertainty inherent in the input or introduced by roundoff error prevents a sure determination. The salient feature of Barber's Quickhull convex hull algorithm is that it merges hull facets that cannot be guaranteed (through error analysis) to be clearly locally convex. The box complex produced by the algorithm is guaranteed to contain the true convex hull, bounding it, if possible, both from within and without.

The degree of robustness required of an algorithm is typically determined by how its output is used. For instance, many point location algorithms can fail when given a non-planar triangulation. For this very reason, my triangulator crashed after producing the flawed triangulation in Figure 19.

The reader should take three lessons from this section. First, problems due to roundoff can be severe and difficult to solve. Second, even if the inputs are imprecise and the user isn't picky about the accuracy of the output, internal consistency may still be necessary if any output is to be produced at all; exact arithmetic may be required even when exact results aren't. Third, neither exact arithmetic nor clever handling of tests that tell falsehoods is a universal balm. However, exact arithmetic is attractive when it is applicable, because it can be employed by na¨ ıve program developers without the time-consuming need for careful analysis of a particular algorithm's behavior when faced with imprecision. (I occasionally hear of implementations where more than half the developers' time is spent solving problems of roundoff error and degeneracy.) Hence, efforts to improve the speed of exact arithmetic in computational geometry are well justified.

## 4.2 The Orientation and Incircle Tests

✄

☛

✠

✎

✠

✝

☛

✄

✠

Define also a procedure I N C IRCLE ✆☞☛ ✄✆✎ that returns a positive value if lies inside the oriented circle . By oriented circle , I mean the unique (and possibly degenerate) circle through , , and , with these points occurring in counterclockwise order about the circle. (If these points occur in clockwise order, I N C IRCLE will reverse the sign of its output, as if the circle's exterior were its interior.) I N C IRCLE returns zero if and only if all four points lie on a common circle. Both O RIENT 2D and I N C IRCLE have the symmetry property that interchanging any two of their parameters reverses the sign of their result.

☛

✠

☛

✄

✠

✎

✁

Let , , , and be four points in the plane. Define a procedure O RIENT 2D ✆☞☛ ✄✆✎ that returns a positive value if the points , , and are arranged in counterclockwise order, a negative value if the points are in clockwise order, and zero if the points are collinear. A more common (but less symmetric) interpretation is that O RIENT 2D returns a positive value if lies to the left of the directed line ; for this purpose the orientation test is used by many geometric algorithms.

☛

✎

✠

✎

✝

✄

✠

☛

✄

✠

These definitions extend trivially to arbitrary dimensions. For instance, O RIENT 3D ✆☞☛ ✪✎ ✄❁✎ returns a positive value if lies below the oriented plane passing through , , and . By oriented plane , I mean that , , and appear in counterclockwise order when viewed from above the plane. (One can apply a left-hand

✠

✁

✁

☛

✄

✠

✁

✝

✟

✄

a0

✤

✎

a0

✄

✁

✄

✁

☛

✠

✎✠☎✠☎✠☎❁✎ ✎✠☎✠☎✠☎✜✎ ❃✧ In any dimension, the orientation and incircle tests may be implemented as matrix determinants. For three dimensions:

rule : orient your left hand with fingers curled to follow the circular sequence . If your thumb points toward , O RIENT 3D returns a positive value.) To generalize the orientation test to dimensionality , let a0✂✁ be the unit vectors; O RIENT is defined so that O RIENT a0✂✁ .

✠

✁

✄

☛

☛

✄

✤

✄

✎

✔

✎

✗

✝

✆

<!-- formula-not-decoded -->

✄

✄

☛

☎

✄

✄

☛

✁

☛

☛

✁

✁

☛

✄

✄

☛

<!-- formula-not-decoded -->

✄

✄

☛

☛

☛

☛

✄

✟

✄

✟

☛

✄

✄

✄

✁

a0

✁

✄

✁

✄

✄

✝

✄

☛

✔

✄

✎

✠

✁

✎

✎

✝

✧

✧

✄

✟

✄

✔

✄

✄

✁

✁

✄

✠

✠

✁

✠

✄

a0

✟

✄

✄

✠

a0

✠

✄

✠

✁

✁

✁

<!-- formula-not-decoded -->

✆

✠

☛

✁

✝

✆

✠

☛

✁

✟

✁

✄

✟

✁

✄

a0

☛

☛

✁

✁

a0

✄

☛

✁

✁

✄

☎

☛

✁

✆

✠

✁

✝

☛

✁

✆

✁

a0

☛

✁

a0

✝

✄

✆

✄

☛

✁

✄

✝

✆

☎

☛

✁

✄

✄

✠

✠

✠

✁

✁

✁

These formulae generalize to other dimensions in the obvious way. Expressions 6 and 7 can be shown to be equivalent by simple algebraic transformations, as can Expressions 8 and 9 with a little more effort. These equivalences are unsurprising because one expects the result of any orientation or incircle test not to change if all the points undergo an identical translation in the plane. Expression 7, for instance, follows from Expression 6 by translating each point by .

✁

When computing these determinants using the techniques of Section 2, the choice between Expressions 6 and 7, or between 8 and 9, is not straightforward. In principle, Expression 6 seems preferable because it can only produce a 96-component expansion, whereas Expression 7 could produce an expansion having 192 components. These numbers are somewhat misleading, however, because with zero-elimination, expansions rarely grow longer than six components in real applications. Nevertheless, Expression 7 takes roughly 25% more time to compute in exact arithmetic, and Expression 9 takes about 50% more time than Expression 8. The disparity likely increases in higher dimensions.

☛

Nevertheless, the mechanics of error estimation turn the tide in the other direction. Important as a fast exact test is, it is equally important to avoid exact tests whenever possible. Expressions 7 and 9 tend to have smaller errors (and correspondingly smaller error estimates) because their errors are a function of the relative coordinates of the points, whereas the errors of Expressions 6 and 8 are a function of the absolute coordinates of the points.

In most geometric applications, the points that serve as parameters to geometric tests tend to be close to each other. Commonly, their absolute coordinates are much larger than the distances between them. By translating the points so they lie near the origin, working precision is freed for the subsequent calculations. Hence, the errors and error bounds for Expressions 7 and 9 are generally much smaller than for Expressions 6 and 8. Furthermore, the translation can often be done without roundoff error. Figure 20 demonstrates a toy

✄

✄

✄

✄

✄

✄

✄

✄

✄

✄

✄

✄

✄

✄

✄

✄

☛

✠

✄

✄

✔

✄

✄

✄

✄

<!-- formula-not-decoded -->

✄

a0

✟

✄

✄

✟

✄

✔

✄

✄

a0

a0

☛

✠

✄

✄

✄

☛

✠

✄

✠

☎

✄

☛

☛

✔

✄

a0

a0

✄

✄

✄

✟

✟

☛

✄

✠

✄

✄

✄

✄

✟

✟

☛

✄

☛

✄

☎

✄

☎

☎

☎

a0

a0

✧

✄

✄

✄

✄

✄

✄

☛

✄

✧

a0

☛

✄

✄

✄

✁

✁

a0

✁

a0

✠

✁

a0

✄

a0

a0

a0

☛

☛

☛

✄

✁

✄

✁

✄

✁

✄

✄

☛

a0

a0

✁

✁

✁

✁

✄

✄

✄

✄

☎

✄

☛

☛

✄

✔

✁

a0

✄

✁

☛

✄

✄

✄

✄

✄

✄

✟

✁

✁

✁

✄

✄

✄

✟

☛

✆

✄

✁

✁

a0

✄

☎

☎

☎

☛

✄

✄

✄

✔

✔

✔

✁

✁

a0

a0

a0

✄

✄

✄

✄

✄

✄

✄

✝

✝

✄

✄

✟

✟

✆

✄

✄

✁

✁

✄

✄

✄

✝

✝

✄

✄

✟

✟

✆

✄

☛

✁

✁

✝

✄

✄

✄

✄

✄

✄

✄

✄

✁

✁

Figure 20: Shaded triangles can be translated to the origin without incurring roundoff error (Lemma 5). In most triangulations, such triangles are the common case.

<!-- image -->

problem: suppose O RIENT 2D is used to find the orientation of each triangle in a triangulation. Thanks to Lemma 5, any shaded triangle can be translated so that one of its vertices lies at the origin without roundoff error; the white triangles may or may not suffer from roundoff during such translation. If the complete triangulation is much larger than the portion illustrated, only a small proportion of the triangles (those near a coordinate axis) will suffer roundoff. Because exact translation is the common case, my adaptive geometric predicates test for and exploit this case.

a0

Once a determinant has been chosen for evaluation, there are several methods to evaluate it. A number of methods are surveyed by Fortune and Van Wyk [11], and only their conclusion is repeated here. The cheapest method of evaluating the determinant of a ★✁ or smaller matrix seems to be by dynamic programming applied to cofactor expansion. Evaluate the ✂✁ determinants of all minors of the first two columns, then the ✂✁ determinants of all ✛✕✁ minors of the first three columns, and so on. All four of my predicates use this method.

## 4.3 O RIENT 2D

My implementation of O RIENT 2D computes a sequence of up to four results (labeled A through D) as illustrated in Figure 21. The exact result D may be as long as sixteen components, but zero elimination is used, so a length of two to six components is more common in practice.

A, B, and C are logical places to test the accuracy of the result before continuing. In most applications, the majority of calls to O RIENT 2D will end with the floating-point approximation A, which is computed

✂

a0

✁

✂

a0

a0

✛

✄

✁

Figure 21: Adaptive calculations used by the 2D orientation test. Dashed boxes represent nodes in the original expression tree.

<!-- image -->

✟

✆

✝

without resort to any exact arithmetic techniques. Although the four-component expansion B, like A, has an error of ✠✝ , it is an appropriate value to test because B is the exact result if the four subtractions at the bottom of the expression tree are performed without roundoff error (corresponding to the shaded triangles in Figure 20). Because this is the common case, O RIENT 2D explicitly tests for it; execution continues only if roundoff occurred during the translation of coordinates and B is smaller than its error bound. The corrected estimate C has an error bound of ✠✝ . If C is not sufficiently accurate, the exact determinant D is computed.

a0

✆

✝

a0

✟

✆

✝

✟

✆

✄

✝

There are two unusual features of this test, both of which arise because only the sign of the determinant is needed. First, the correctional term added to B to form C is not added exactly; instead, the A PPROXIMATE procedure of Section 2.7 is used to find an approximation B of B, and the correctional term is added to B with the possibility of roundoff error. The consequent errors may be of magnitude ✠✝ B , which would normally preclude obtaining an error bound of ✠✝ . However, the sign of the determinant is only questionable if B is of magnitude ✠✝ , so an ✠✝ error bound for C can be established.

✟

✆

✝

✟

✆

✄

✝

✟

✄

✁

a0

✥

a0

✟

✝

✝

✄

a0

a0

a0

✁

✁

✁

✖

✖

✖

✝

✝

The second interesting feature is that, if C is not sufficiently accurate, no more approximations are computed before computing the exact determinant. To understand why, consider three collinear points , , and ; the determinant defined by these points is zero. If a coordinate of one of these points is perturbed by a single ulp, the determinant typically increases to ✠✝ . Hence, one might guess that when a determinant is no larger than ✠✝ , it is probably zero. This intuition seems to hold in practice for all the predicates considered herein, on both random and 'practical' point sets. Determinants that don't stop with approximation C are nearly always zero.

✝

✖

✥

✁

☛

Let us consider the error bound for A. For each subexpression in the expression tree of the orientation test, denote its true (exact) value and its approximate value as follows.

✄

✟

✆

✝

✆

✝

The derivation of error bounds for these values is tricky, so an example is given here. The easiest way to apply forward error analysis to an expression whose value is calculated in floating-point arithmetic is to express the exact value of each subexpression in terms of the computed value plus an unknown error term whose magnitude is bounded. For instance, the error incurred by the computation ✏✄ is no larger than ✡✖ . Furthermore, the error is smaller than . Each of these bounds is useful under different circumstances. If represents the true value , an abbreviated way of expressing these notions is to write ✧✏✥ ✎✖ and ✧✏✥ . Henceforth, this notation will be used as shorthand for the relation ✧✲✥ ✁a0 for some that satisfies ✖✘✗ ✎✖ and .

✁

✭

☛

✠

a0

✥

✭

☛

✥

✁

✖

✖

✗

✖

✖

✖

<!-- formula-not-decoded -->

✄

a0

✁

✥

☎

✥

✤

✄

✁

✆

✥

☎

✥

✁

✁

✁

✄

✁

✧

✧

✁

✁

✁

✤

✁

✁

✆

From these definitions, it is clear that ✞✝ ; similar bounds hold for , , and . Observe also that ✧✏✥ ✧✏✥ . It follows that

✧

✄

✧

✥

✁

✤

✥

✄

✝

a0

✁

✝

✁

✁

✤

✁

✝

Similarly, ✓✆ ✝✏✖ .

✁

✧

✥

✁

a0

✖

✥

✝

✟

✖

✁

✁

✄

✧

✁

✁

☛

✁

<!-- formula-not-decoded -->

✧

✥

a0

✛

✥

✖

✄

✝

✄

✝

✟

✝

✁

✤

✝

✝

✆

✁

✖

✥

✁

✖

✝

✖

✥

✁

✖

✝

It may seem odd to be keeping track of terms smaller than ✠✝ , but the effort to find the smallest machine-representable coefficient for each error bound is justified if it ever prevents a determinant computation from becoming more expensive than necessary. An error bound for A can now be derived.

✧

✧

✥

✛

✥

✖

☎

✟

✝

✝

✝

✝

✁

✖

✟

✖

✥

✄

✄

✖

✝

✁

✄

✝

✟

✝

✄

✟

✝

✁

✛

✧

✝

✖

✖

a0

✖

✖

✛

✝

<!-- formula-not-decoded -->

✆

a0

✛

✛

✝

✆

✖

✥

✟

✄

✟

✁

✁

✁

☛

✁

✄

One can minimize the effect of the term A by taking advantage of the fact that we are only interested in the sign of ✄✂ . One can conclude with certainty that A has the correct sign if

✔

☛

✲

✛

✟

✛

✄

✟

✁

✥

✟

✥

✄

✝

✖

✆

✛

✝

✆

✖

✥

✁

<!-- formula-not-decoded -->

✆

✝

✝

✝

✝

✆

✖

✁

✖

✖

✖

✝

✖

✟

✖

✖

✝

✥

✥

✄

✝

✖

✥

✖

✤

✧

✥

✤

a0

✥

✤

✝

✖

a0

✄

✠

a0

✁

✁

✄

✤

✧

☛

☛

☛

✠

✠

✄

✄

✥

✥

✄

✤

✧

☛

✄

✄

✄

✠

✠

✄

✄

✝

✖

☛

✟

✄

✖

✥

☞

☛

✂

✠

✥

✁

✁

✁

✄

✧

✥

✤

✄

☎

a0

✥

✄

✛

✝

✖

✟

✤

✥

✁

✁

✄

✆

✁

✟

✧

✁

a0

✄

☛

☛

✁

✠

a0

✁

✟

✟

a0

✖

✟

a0

✖

✄

✟

✄

✥

✥

✆

✁

✧

✆

✝

a0

✄

✄

✥

✠

✄

a0

✄

✥

✁

✄

✁

✁

✁

✆

☛

✄

✂

✆

✝

✄

✝

✟

✁

✔

✖

✝

✝

✁

✝

✝

☎✓✆ ✖

✥

✥

✁

✁

✖

✖

✂✓✖

✥

✥

✄

✄

✖

✖

✝

✝

| Approximation   | Error bound     |
|-----------------|-----------------|
| A B             | ☎✓✆ ✂✓✖ ☎✓✆ ✂✓✖ |
| C               | B               |

a0

<!-- formula-not-decoded -->

✄

a0

✆

✛

✝

✟

a0

✝

✄

✝

☎

✖

a0

✖✏✂✓✆

✂

Table 1: Error bounds for the expansions calculated by O RIENT 2D . B is a -bit approximation of the expansion B, computed by the A PPROXIMATE procedure. Note that each coefficient is expressible in bits.

which is true if

✖

✖

✥

✁

✟

✄

✫

✝

✟

✁

✝

✄

✟

a0

✝

✁

This bound is not directly applicable, because its computation will incur roundoff error. To account for this, multiply the coefficient by (a factor of for the addition of and , and another such factor for the multiplication). Hence, we are secure that the sign of A is correct if

✖

✖

✥

✖

✝

✖

✆

✛

✝

✆

✔

✟

✟

✝

✝

<!-- formula-not-decoded -->

✛

✁

✄

✝

✟

a0

✝

✄

✟

a0

✝

✁

This bound is not directly applicable either, because the coefficient is not expressible in bits. Rounding up to the next -bit number, we have the coefficient , which should be exactly computed once at program initialization and reused during each call to O RIENT 2D.

a0

✂

✆

✝

✟

✔

✁

✝

✄

✝

Error bounds for A, B , and C are given in Table 1. The bound for B takes advantage of Theorem 23, which shows that B approximates B with relative error less than . (Recall from Section 2.7 that the largest component of B might have only one bit of precision.)

a0

a0

a0

✝

These bounds have the pleasing property that they are zero in the common case that all three input points lie on a horizontal or vertical line. Hence, although O RIENT 2D usually resorts to exact arithmetic when given collinear input points, it only performs the approximate test (A) in the two cases that occur most commonly in practice.

Compiler effects affect the implementation of O RIENT 2D. By separating the calculation of A and the remaining calculations into two procedures, with the former calling the latter if necessary, I reduced the time to compute A by 25%, presumably because of improvements in the compiler's ability to perform register allocation.

Table 2 lists timings for O RIENT 2D, given random inputs. Observe that the adaptive test, when it stops at the approximate result A, takes nearly twice as long as the approximate test because of the need to compute an error bound. The table includes a comparison with Bailey's MPFUN [2], chosen because it is the fastest portable and freely available arbitrary precision package I know of. O RIENT 2D coded with my (nonadaptive) algorithms is roughly thirteen times faster than O RIENT 2D coded with MPFUN.

Also included is a comparison with an orientation predicate for 53-bit integer inputs, created by Fortune and Van Wyk's LN. The LN-generated orientation predicate is quite fast because it takes advantage of the fact that it is restricted to bounded integer inputs. My exact tests cost less than twice as much as LN's; this seems like a reasonable price to pay for the ability to handle arbitrary exponents in the input.

These timings are not the whole story; LN's static error estimate is typically much larger than the runtime error estimate used for adaptive stage A, and LN uses only two stages of adaptivity, so the LN-generated predicates are slower in some applications, as Section 4.5 will demonstrate. It is significant that for 53-bit integer inputs, the multiple-stage predicates will rarely pass stage B because the initial translation is usually

✖

✆

✛

✔

✖

✝

✥

✖

✥

✖

✝

✧

✧

✆

✛

a0

✝

✟

✟

✔

✁

a0

✝

✄

✄

✝

✥

✁

✖

✥

✄

✖

✝

✖

✆

✔

✝

✆

✝

✖

✥

✁

✖

✖

✥

✄

✖

Table 2: Timings for O RIENT 2D on a DEC 3000/700 with a 225 MHz Alpha processor. All determinants use the 2D version of either Expression 6 or the more stable Expression 7 as indicated. The first two columns indicate input points generated from a uniform random distribution and a geometric random distribution. The third column considers two points chosen from one of the random distributions, and a third point chosen to be approximately collinear to the first two. Timings for the adaptive tests are categorized according to which result was the last generated. Each timing is an average of 60 or more randomly generated inputs. For each such input, time was measured by a Unix system call before and after 10,000 iterations of the predicate. Individual timings vary by approximately 10%. Timings of Bailey's MPFUN package and Fortune and Van Wyk's LN package are included for comparison.

| Double precision O RIENT 2D timings in microseconds   | Double precision O RIENT 2D timings in microseconds   | Double precision O RIENT 2D timings in microseconds   | Double precision O RIENT 2D timings in microseconds   |
|-------------------------------------------------------|-------------------------------------------------------|-------------------------------------------------------|-------------------------------------------------------|
| Points Method                                         | Uniform Random                                        | Geometric Random                                      | Nearly Collinear                                      |
| Approximate (7)                                       | 0.15                                                  | 0.15                                                  | 0.16                                                  |
| Exact (6)                                             | 6.56                                                  | 6.89                                                  | 6.31                                                  |
| Exact (7)                                             | 8.35                                                  | 8.48                                                  | 8.13                                                  |
| Exact (6), MPFUN                                      | 92.85                                                 | 94.03                                                 | 84.97                                                 |
| Adaptive A (7), approximate                           | 0.28                                                  | 0.27                                                  | 0.22                                                  |
| Adaptive B (7)                                        |                                                       |                                                       | 1.89                                                  |
| Adaptive C (7)                                        |                                                       |                                                       | 2.14                                                  |
| Adaptive D (7), exact                                 |                                                       |                                                       | 8.35                                                  |
| LN adaptive (7), approximate                          | 0.32                                                  | n/a                                                   |                                                       |
| LN adaptive (7), exact                                |                                                       | n/a                                                   | 4.43                                                  |

done without roundoff error; hence, the LN-generated O RIENT 2D usually takes more than twice as long to produce an exact result. It should be emphasized, however, that these are not inherent differences between LN's multiple-digit integer approach and my multiple-component floating-point approach; LN could, in principle, employ the same runtime error estimate and a similar multiple-stage adaptivity scheme.

## 4.4 O RIENT 3D , I N C IRCLE , and I N S PHERE

Figure 22 illustrates the implementation of O RIENT 3D, which is similar to the O RIENT 2D implementation. A is the standard floating-point result. B is exact if the subtractions at the bottom of the tree incur no roundoff. C represents a drop in the error bound from ✠✝ to ✠✝ . D is the exact determinant.

✟

✟

✄

Error bounds for the largest component of each of these expansions are given in Table 3, partly in terms of the variables , , and in Figure 22. The bounds are zero if all four input points share the same -, -, or -coordinate, so only the approximate test is needed in the most common instances of coplanarity.

✥

✤

✥

✄

✥

✁

✥

a0

Table 4 lists timings for O RIENT 3D, given random inputs. The error bound for A is expensive to compute, and increases the amount of time required to perform the approximate test in the adaptive case by a factor of two and a half. The gap between my exact algorithm and MPFUN is smaller than in the 2D case, but is still a factor of nearly eight.

✯

Oddly, the table reveals that D is calculated more quickly than the exact result is calculated by the nonadaptive version of O RIENT 3D. The explanation is probably that D is only computed when the determinant is zero or very close to zero, hence the lengths of the intermediate expansions are smaller than usual, and the

✆

✝

✆

✝

Figure 22: Adaptive calculations used by the 3D orientation test. Bold numbers indicate the length of an expansion. Only part of the expression tree is shown; two of the three cofactors are omitted, but their results appear as dashed components and expansions.

<!-- image -->

a0

a0

☎

✧

✧

✄

✄

✝

✂

a0

✄

✄

✖

a0

✖

✁

| Approximation   | Error bound                       |
|-----------------|-----------------------------------|
| A               | ☎✓✆ ✁a0 ✂a0 ✄a0✆☎ ☎✓✆ ✁a0 ✂a0 ✄a0 |
| B               |                                   |
| C               | B ✄✠✟ ✁a0 ✂a0 ✄a0                 |

a0

☎

✁

✧

✖

✖

✄

✁

✝

✄

✄

✂

☎

✄

a0

✝

✝

✟

✂

a0

a0

✁

✝

✝

✄

✝

☎

✂

✁

✂

☎

✝

✆

a0

<!-- formula-not-decoded -->

✖

✆

✝

☎✓✆

✁

✁

✄

✄

✄

✄

✝

✠

✖

✧

✄

✄

a0

✁

✄

✄

✖

✆

✁

Table 3: Error bounds for the expansions calculated by O RIENT 3D .

| Double precision O RIENT 3D timings in microseconds   | Double precision O RIENT 3D timings in microseconds   | Double precision O RIENT 3D timings in microseconds   | Double precision O RIENT 3D timings in microseconds   |
|-------------------------------------------------------|-------------------------------------------------------|-------------------------------------------------------|-------------------------------------------------------|
| Points Method                                         | Uniform Random                                        | Geometric Random                                      | Nearly Coplanar                                       |
| Approximate (7)                                       | 0.25                                                  | 0.25                                                  | 0.25                                                  |
| Exact (6)                                             | 33.30                                                 | 38.54                                                 | 32.90                                                 |
| Exact (7)                                             | 42.69                                                 | 48.21                                                 | 42.41                                                 |
| Exact (6), MPFUN                                      | 260.51                                                | 262.08                                                | 246.64                                                |
| Adaptive A (7), approximate                           | 0.61                                                  | 0.60                                                  | 0.62                                                  |
| Adaptive B (7)                                        |                                                       |                                                       | 12.98                                                 |
| Adaptive C (7)                                        |                                                       |                                                       | 15.59                                                 |
| Adaptive D (7), exact                                 |                                                       |                                                       | 27.29                                                 |
| LN adaptive (7), approximate LN adaptive (7), exact   | 0.85                                                  | n/a n/a                                               | 18.11                                                 |

Table 4: Timings for O RIENT 3D on a DEC 3000/700. All determinants are Expression 6 or the more stable Expression 7 as indicated. Each timing is an average of 120 or more randomly generated inputs. For each such input, time was measured by a Unix system call before and after 10,000 iterations of the predicate. Individual timings vary by approximately 10%.

computation time is less. Furthermore, when some of the point coordinates are translated without roundoff error, the adaptive predicate ignores branches of the expression tree that evaluate to zero.

I N C IRCLE is implemented similarly to O RIENT 3D, as the determinants are similar. The corresponding error bounds appear in Table 5, and timings appear in Table 6.

Timings for I N S PHERE appear in Table 7. This implementation differs from the other tests in that, due to programmer laziness, D is not computed incrementally from B; rather, if C is not accurate enough, D is computed from scratch. Fortunately, C is usually accurate enough.

The LN exact tests have an advantage of a factor of roughly 2.5 for I N C IRCLE and 4 for I N S PHERE , so the cost of handling floating-point operands is greater with the larger expressions. As with the orientation tests, this cost is mediated by better error bounds and four-stage adaptivity.

The timings for the exact versions of all four predicates show some sensitivity to the distribution of the

✄

✥

☛

✤

✄

✁

✁

✥

✖

☎

✄

✆

✖

✆

✠

✥

a0

✁

✄

✁

✁

a0

a0

✝

☎

✆

✠

✄

✄

✁

✁

✄

✄

✂

✖

✆

✠

✄

✁

✄

✝

☎

✆

✠

a0

✄

✁

✁

a0

a0

✖

✆

☎

☎

✖

✆

✛

✖

✝

✂

✟

✄

✖

a0

✝

✄

✖

✝

✝

☎

✖

a0

✖✞✂

✆

a0

✁

✝

✆

✛

✟

a0

a0

a0

✄

✝

✁

✄

✝

✝

☎

☎

✂

✂

✁

✁

✝

✂

✂

☎

✝

a0

a0

| Approximation   | Error bound                                   |
|-----------------|-----------------------------------------------|
| A B             | ☎✓✆ ✁a0 ✂a0 ✄a0✆☎ ☎✓✆ ✁a0 ✂a0 ✄a0 ✁a0 ✂a0 ✄a0 |
| C               | B ✄✠✟                                         |

✆

✝

✠

✄

✝

✄

✂

✖

✆

✠

✄

✝

✄

a0

✝

✄

✆

✔

✝

a0

✝

✁

✝

✂

✂

☎

✝

a0

✟

✆☞☛ ☎a0 ✂✓✆☞☛ ☎✄ ☎✓✆ ✖ ✆ ✂a0 ☎✓✆ ✝✏✖ ✂✄ ☎✓✆ ✝✏✖ ✝

✄

a0

✝

✆

✖

✂

✖

a0

✁

✝

✝

☎

✂

✁

✂

☎

✝

✝

✂a0 ✂✓✆ ✂✄ ☎✓✆ ✖ ✆ ☎✓✆☞☛ ☎✄ ✝✏✖ ✂✄ ☎✓✆☞☛ ☎a0 ✝✏✖ ✝

✄

✝

☎

✖

✖✞✂

✁

✆

✖✚✖

✁

✝

☎✓✆

☎

✂

✁

✂

✝

✁

✆

✛

✁

✂✓✆ ✆☞☛ ☎✓✆ ✂✄ ✝✏✖✞✂✓✖ ✆☞☛ ☎✓✆ ✂a0 ✝✏✖ ✝

✝

☎

✆

✖

a0

✄

✁

a0

✝

✄

✄

✁

✄

✄

✄

✁

✄

✝

✄

✄

✁

a0

☎

✧

✧

✆

✆

✆

✄

✄

✝

✄

✄

✝

✠

a0

✄

✁

a0

✝

✠

✄

✄

✁

✄

✝

✄

Table 5: Error bounds for the expansions calculated by I N C IRCLE . Squares are approximate.

| Double precision I N C IRCLE timings in microseconds   | Double precision I N C IRCLE timings in microseconds   | Double precision I N C IRCLE timings in microseconds   | Double precision I N C IRCLE timings in microseconds   |
|--------------------------------------------------------|--------------------------------------------------------|--------------------------------------------------------|--------------------------------------------------------|
| Points Method                                          | Uniform Random                                         | Geometric Random                                       | Nearly Cocircular                                      |
| Approximate (9)                                        | 0.31                                                   | 0.28                                                   | 0.30                                                   |
| Exact (8)                                              | 71.66                                                  | 83.01                                                  | 75.34                                                  |
| Exact (9)                                              | 91.71                                                  | 118.30                                                 | 104.44                                                 |
| Exact (8), MPFUN                                       | 350.77                                                 | 343.61                                                 | 348.55                                                 |
| Adaptive A (9), approximate                            | 0.64                                                   | 0.59                                                   | 0.64                                                   |
| Adaptive B (9)                                         |                                                        |                                                        | 44.56                                                  |
| Adaptive C (9)                                         |                                                        |                                                        | 48.80                                                  |
| Adaptive D (9), exact                                  |                                                        |                                                        | 78.06                                                  |
| LN adaptive (9), approximate                           | 1.33                                                   | n/a                                                    |                                                        |
| LN adaptive (9), exact                                 |                                                        | n/a                                                    | 32.44                                                  |

Table 6: Timings for I N C IRCLE on a DEC 3000/700. All determinants are the 2D version of either Expression 8 or the more stable Expression 9 as indicated. Each timing is an average of 100 or more randomly generated inputs, except adaptive stage D. (It is difficult to generate cases that reach stage D.) For each such input, time was measured by a Unix system call before and after 1,000 iterations of the predicate. Individual timings vary by approximately 10%.

operands; they take 5% to 30% longer to execute with geometrically distributed operands (whose exponents vary widely) than with uniformly distributed operands. This difference occurs because the intermediate and final expansions are larger when the operands have broadly distributed exponents. The exact orientation predicates are cheapest when their inputs are collinear/coplanar, because of the smaller expansions that result, but this effect does not occur for the exact incircle predicates.

## 4.5 Performance in Two Triangulation Programs

To evaluate the effectiveness of the adaptive tests in applications, I tested them in two of my Delaunay triangulation codes. Triangle [25] is a 2D Delaunay triangulator and mesh generator, publicly available from Netlib, that uses a divide-and-conquer algorithm [18, 14]. Pyramid is a 3D Delaunay tetrahedralizer that uses an incremental algorithm [27]. For both 2D and 3D, three types of inputs were tested: uniform random points, points lying (approximately) on the boundary of a circle or sphere, and a square or cubic

✁

a0

✄

✁

✄

✄

a0

✁

a0

✁

✄

✄

✁

✄

a0

☎

✁

✧

✆

✄

✁

a0

✝

✄

✗

✝

✟

✟

✂

✁

✝

✄

✄

☎

✁

✁

✄

✄

✝

✄

✝

✄

a0

✝

✠

✄

✄

✄

✂

✖

✆

✄

✄

✝

✠

a0

✄

✁

a0

a0

Table 7: Timings for I N S PHERE on a DEC 3000/700. All determinants are Expression 8 or the more stable Expression 9 as indicated. Each timing is an average of 25 or more randomly generated inputs, except adaptive stage D. For each such input, time was measured by a Unix system call before and after 1,000 iterations of the predicate. Individual timings vary by approximately 10%.

| Double precision I N S PHERE timings in microseconds   | Double precision I N S PHERE timings in microseconds   | Double precision I N S PHERE timings in microseconds   | Double precision I N S PHERE timings in microseconds   |
|--------------------------------------------------------|--------------------------------------------------------|--------------------------------------------------------|--------------------------------------------------------|
| Points Method                                          | Uniform Random                                         | Geometric Random                                       | Nearly Cospherical                                     |
| Approximate (9)                                        | 0.93                                                   | 0.95                                                   | 0.93                                                   |
| Exact (8)                                              | 324.22                                                 | 378.94                                                 | 347.16                                                 |
| Exact (9)                                              | 374.59                                                 | 480.28                                                 | 414.13                                                 |
| Exact (8), MPFUN                                       | 1,017.56                                               | 1,019.89                                               | 1,059.87                                               |
| Adaptive A (9), approximate                            | 2.13                                                   | 2.14                                                   | 2.14                                                   |
| Adaptive B (9)                                         |                                                        |                                                        | 166.21                                                 |
| Adaptive C (9)                                         |                                                        |                                                        | 171.74                                                 |
| Adaptive D (8), exact                                  |                                                        |                                                        | 463.96                                                 |
| LN adaptive (9), approximate LN adaptive (9), exact    | 2.35                                                   | n/a n/a                                                | 116.74                                                 |

grid of lattice points, tilted so as not to be aligned with the coordinate axes. The latter two were chosen for their nastiness. The lattices have been tilted using approximate arithmetic, so they are not perfectly cubical, and the exponents of their coordinates vary enough that LN cannot be used. (I have also tried perfect lattices with 53-bit integer coordinates, but O RIENT 3D and I N S PHERE never pass stage B; the perturbed lattices are preferred here because they occasionally force the predicates into stage C or D.)

The results for 2D, which appear in Table 8, indicate that the four-stage predicates add about 8% to the total running time for randomly distributed input points, mainly because of the error bound tests. For the more difficult point sets, the penalty may be as great as 30%. Of course, this penalty applies precisely for the point sets that are most likely to cause difficulties when exact arithmetic is not available.

The results for 3D, outlined in Table 9, are less pleasing. The four-stage predicates add about 35% to the total running time for randomly distributed input points; for points distributed approximately on the surface of a sphere, the penalty is a factor of eleven. Ominously, however, the penalty for the tilted grid is uncertain, because the tetrahedralization program using approximate arithmetic failed to terminate. A debugger revealed that the point location routine was stuck in an infinite loop because a geometric inconsistency had been introduced into the mesh due to roundoff error. Robust arithmetic is not always slower after all.

In these programs (and likely in any program), three of the four-stage predicates (I N S PHERE being the exception) are faster than their LN equivalents. This is a surprise, considering that the four-stage predicates accept 53-bit floating-point inputs whereas the LN-generated predicates are restricted to 53-bit integer inputs. However, the integer predicates would probably outperform the floating-point predicates if they were to adopt the same runtime error estimate and a similar four-stage adaptivity scheme.

## 5 Caveats

Unfortunately, the arbitrary precision arithmetic routines described herein are not universally portable; both hardware and compilers can prevent them from functioning correctly.

✁

| 2D divide-and-conquer Delaunay triangulation                    | 2D divide-and-conquer Delaunay triangulation   | 2D divide-and-conquer Delaunay triangulation   | 2D divide-and-conquer Delaunay triangulation   |
|-----------------------------------------------------------------|------------------------------------------------|------------------------------------------------|------------------------------------------------|
|                                                                 | Uniform Random                                 | Perimeter of Circle                            | Tilted Grid                                    |
| Input sites                                                     | 1,000,000                                      | 1,000,000                                      | 1,000,000                                      |
| O RIENT 2D calls                                                |                                                |                                                |                                                |
| Adaptive A, approximate Adaptive B Adaptive C Adaptive D, exact | 9,497,314                                      | 6,291,742                                      | 9,318,610 121,081 118 3                        |
| Average time, s                                                 | 0.32                                           | 0.38                                           | 0.33                                           |
| LN approximate LN exact                                         | 9,497,314                                      | 2,112,284 4,179,458                            | n/a n/a                                        |
| LN average time, s                                              | 0.35                                           | 3.16                                           | n/a                                            |
| I N C IRCLE calls                                               |                                                |                                                |                                                |
| Adaptive A, approximate Adaptive B Adaptive C Adaptive D, exact | 7,596,885                                      | 3,970,796 50,551 120                           | 7,201,317 176,470 47 4                         |
| Average time, s                                                 | 0.65                                           | 1.11                                           | 1.67                                           |
| LN approximate LN exact LN average time, s                      | 6,077,062 1,519,823 7.36                       | 0 4,021,467 32.78                              | n/a n/a n/a                                    |
| Program running time, seconds                                   | Program running time, seconds                  | Program running time, seconds                  | Program running time, seconds                  |
| Approximate version                                             | 57.3                                           | 59.9                                           | 48.3                                           |
| Robust version                                                  | 61.7                                           | 64.7                                           | 62.2                                           |
| LN robust version                                               | 116.0                                          | 214.6                                          | n/a                                            |

a0

Table 8: Statistics for 2D divide-and-conquer Delaunay triangulation of several point sets. Timings are accurate to within 10%.

✄

✄

Compilers can interfere by making invalid optimizations based on misconceptions about floating-point arithmetic. For instance, a clever but incorrect compiler might cause expansion arithmetic algorithms to fail by deriving the 'fact' that virtual , computed by Line 2 of F AST -T WO -S UM , is equal to , and optimizing the subtraction away. This optimization would be valid if computers stored arbitrary real numbers, but is incorrect for floating-point numbers. Unfortunately, not all compiler developers are aware of the importance of maintaining correct floating-point language semantics, but as a whole, they seem to be improving. Goldberg [13, 3.2.3] presents several related examples of how carefully designed numerical algorithms can be utterly ruined by incorrect optimizations.

Even floating-point units that use binary arithmetic with exact rounding, including those that conform to the IEEE 754 standard, can have subtle properties that undermine the assumptions of the algorithms. The most common such difficulty is the presence of extended precision internal floating-point registers, such as those on the Intel 80486 and Pentium processors. While such registers usually improve the stability of floating-point calculations, they cause the methods described herein for determining the roundoff of an operation to fail. There are several possible workarounds for this problem. In C, it is possible to designate a variable as volatile, implying that it must be stored to memory. This ensures that the variable is rounded

a0

a0

a0

✂

| 3D incremental Delaunay tetrahedralization                      | 3D incremental Delaunay tetrahedralization   | 3D incremental Delaunay tetrahedralization   | 3D incremental Delaunay tetrahedralization   |
|-----------------------------------------------------------------|----------------------------------------------|----------------------------------------------|----------------------------------------------|
|                                                                 | Uniform Random                               | Surface of Sphere                            | Tilted Grid                                  |
| Input sites                                                     | 10,000                                       | 10,000                                       | 10,000                                       |
| O RIENT 3D calls                                                |                                              |                                              |                                              |
| Adaptive A, approximate Adaptive B Adaptive C Adaptive D, exact | 2,735,668                                    | 1,935,978                                    | 5,542,567 602,344 1,267,423 28,185           |
| Average time, s                                                 | 0.72                                         | 0.72                                         | 4.12                                         |
| LN approximate LN exact                                         | 2,735,668                                    | 1,935,920 58                                 | n/a n/a                                      |
| LN average time, s                                              | 0.99                                         | 1.00                                         | n/a                                          |
| I N S PHERE calls                                               |                                              |                                              |                                              |
| Adaptive A, approximate Adaptive B Adaptive C Adaptive D, exact | 439,090                                      | 122,273 180,383 1,667                        | 3,080,312 267,162 548,063                    |
| Average time, s                                                 | 2.23                                         | 96.45                                        | 48.12                                        |
| LN approximate LN exact                                         | 438,194 896                                  | 104,616 199,707                              | n/a n/a                                      |
| LN average time, s                                              | 2.50                                         | 70.82                                        | n/a                                          |
| Program running time, seconds                                   | Program running time, seconds                | Program running time, seconds                | Program running time, seconds                |
| Approximate version Robust version                              | 4.3                                          | 3.0                                          |                                              |
|                                                                 | 5.8                                          | 34.1                                         | 108.5                                        |
| LN robust version                                               | 6.5                                          | 30.5                                         | n/a                                          |

✌

Table 9: Statistics for 3D incremental Delaunay tetrahedralization of several point sets. Timings are accurate to within 10%. The approximate code failed to terminate on the tilted grid input.

✂

to a -bit significand before it is used in another operation. Forcing intermediate values to be stored to memory and reloaded can slow down the algorithms significantly, and there is a worse consequence. Even a volatile variable could be doubly rounded , being rounded once to the internal extended precision format, then rounded again to single or double precision when it is stored to memory. The result after double rounding is not always the same as it would be if it had been correctly rounded to the final precision, and Priest [24, page 103] describes a case wherein the roundoff error produced by double rounding may not be expressible in bits. This might be alleviated by a more complex (and slower) version of F AST -T WO -S UM . A better solution is to configure one's processor to round internally to double precision. While most processors with internal extended precision registers can be thus configured, and most compilers provide support for manipulating processor control state, such support varies between compilers and is not portable. Nevertheless, the speed advantage of multiple-component methods makes it well worth the trouble to learn the right incantation to correctly configure your processor.

The algorithms do work correctly without special treatment on most current Unix workstations. Nevertheless, users should be careful when trying the routines, or moving to a new platform, to ensure that the

a0

a0

a0

a0

underlying assumptions of the method are not violated.

## 6 Conclusions

The algorithms presented herein are simple and fast; looking at Figure 8, it is difficult to imagine how expansions could be summed with fewer operations without special hardware assistance. Two features of these techniques account for the improvement in speed relative to other techniques, especially for numbers whose precision is only a few components in length. The first is the relaxation of the usual condition that numbers be normalized to fixed digit positions. Instead, one enforces the much weaker condition that expansions be nonoverlapping (or strongly nonoverlapping). Expansions can be summed and the resulting components made nonoverlapping at a cost of six floating-point operations and one comparison per component. It seems unlikely that normalization to fixed digit positions can be done so quickly in a portable way on current processors. The second feature to which I attribute the improved speed is the fact that most packages require expensive conversions between ordinary floating-point numbers and the packages' internal formats. With the techniques Priest and I describe, no conversions are necessary.

The reader may be misled and attribute the whole difference between my algorithms and MPFUN to the fact that I store double precision components, while MPFUN stores single precision digits, and imagine the difference would go away if MPFUN were reimplemented in double precision. Such a belief betrays a misunderstanding of how MPFUN works. MPFUN uses double precision arithmetic internally, and obtains exact results by using digits narrow enough that they can be multiplied exactly. Hence, MPFUN's halfprecision digits are an integral part of its approach: to calculate exactly by avoiding roundoff error. The surprise of multiple-component methods is that reasonable speed can be attained by allowing roundoff to happen, then accounting for it after the fact.

As well as being fast, multiple-component algorithms are also reasonably portable, making no assumptions other than that a machine has binary arithmetic with exact rounding (and round-to-even tiebreaking if F AST -E XPANSION -S UM is to be used instead of L INEAR -E XPANSION -S UM ). No representation-dependent tricks like bit-masking to extract exponent fields are used. There are still machines that cannot execute these algorithms correctly, but their numbers seem to be dwindling as the IEEE standard becomes entrenched.

✗

Perhaps the greatest limitation of the multiple-component approach is that while it easily extends the precision of floating-point numbers, there is no simple way to extend the exponent range without losing much of the speed. The obvious approach, associating a separate exponent field with each component, is sure to be too slow. A more promising approach is to express each multiprecision number as a multiexpansion consisting of digits of very large radix, where each digit is an expansion coupled with an exponent. In this scheme, the true exponent of a component is the sum of the component's own exponent and the exponent of the expansion that contains it. The fast algorithms described in this article can be used to add or multiply individual digits; digits are normalized by standard methods (such as those used by MPFUN). IEEE double precision values have an exponent range of ☛✕✔ a0✂a0 to , so one could multiply digits of radix ✞☎✞☎✞ with a simple expansion multiplication algorithm, or digits of radix ✄☎✞☎✞☎✞ with a slightly more complicated one that splits each digit in half before multiplying.

✔

✗

✛

✤

a0

The C code I have made publicly available might form the beginning of an extensive library of arithmetic routines similar to MPFUN, but a great deal of work remains to be done. In addition to the problem of expanding the exponent range, there is one problem that is particular to the multiple-component approach: it is not possible to use FFT-based multiplication algorithms without first renormalizing each expansion to a multiple-digit form. This normalization is not difficult to do, but it costs time and puts the multiple-

a0

a0

component method at a disadvantage relative to methods that keep numbers in digit form as a matter of course.

✁

As Priest points out, multiple-component algorithms can be used to implement extended (but finite) precision arithmetic as well as exact arithmetic; simply compress and then truncate each result to a fixed number of components. Perhaps the greatest potential of these algorithms lies not with arbitrary precision libraries, but in providing a fast and simple way to extend slightly the precision of critical variables in numerical algorithms. Hence, it would not be difficult to provide a routine that quickly computes the intersection point of two segments with double precision endpoints, correctly rounded to a double precision result. If an algorithm can be made significantly more stable by using double or quadruple precision for a few key values, it may save a researcher from spending a great deal of time devising and analyzing a stabler algorithm; Priest [24, 5.1] offers several examples. Speed considerations may make it untenable to accomplish this by calling a standard extended precision library. The techniques Priest and I have developed are simple enough to be coded directly in numerical algorithms, avoiding function call overhead and conversion costs.

A useful tool in coding such algorithms would be an expression compiler similar to Fortune and Van Wyk's LN [12, 11], which converts an expression into exact arithmetic code, complete with error bound derivation and floating-point filters. Such a tool could also automate the process of breaking an expression into adaptive stages as described in Section 3.

To see how adaptivity can be used for more than just determining the sign of an expression, suppose one wishes to find, with relative error no greater than 1%, the center of a circle that passes through the three points , , and . One may use the following expressions.

✁

✄

✄

✠

✄

✆

✠

a0

✝

✄

✟

✆

✠

✝

✄

✄

✄

✁

✄

✄

✠

a0

✆

✠

a0

✝

✄

✟

✆

✠

✄

✝

✄

✄

✄

☛

✄

☛

✠

✠

✄

a0

✠

a0

✝

✠

✄

✝

☛

a0

✠

a0

a0

✠

a0

✝

✁

<!-- formula-not-decoded -->

✄

✄

✄

☛

☛

✠

☛

☛

✠

✄

✄

✠

✄

✄

✄

✄

✄

☛

☛

✠

✠

a0

☛

☛

✠

✠

a0

✠

The denominator of these fractions is precisely the expression computed by O RIENT 2D. The computation of is unstable if , , and are nearly collinear; roundoff error in the denominator can dramatically change the result, or cause a division by zero. Disaster can be avoided, and the desired error bound enforced, by computing the denominator with a variant of O RIENT 2D that accepts an approximation only if its relative error is roughly half of one percent. A similar adaptive routine could accurately compute the numerators.

✁

✖

✁

☛

✠

It might be fruitful to explore whether the methods described by Clarkson [6] and Avnaim et al. [1] can be extended by fast multiprecision methods to handle arbitrary double precision floating-point inputs. One could certainly relax their constraints on the bit complexity of the inputs; for instance, the method of Avnaim et al. could be made to perform the I N S PHERE test on -bit inputs using expansions of length three. Unfortunately, it is not obvious how to adapt these integer-based techniques to inputs with wildly differing exponents. It is also not clear whether such hybrid algorithms would be faster than straightforward adaptivity. Nevertheless, Clarkson's approach looks promising for larger determinants. Although my methods work well for small determinants, they are unlikely to work well for sizes much larger than . Even if one uses Gaussian elimination rather than cofactor expansion (an important adjustment for matrices larger than ), the adaptivity technique does not scale well with determinants, because of the large number of terms in the expanded polynomial. Clarkson's technique may be the only economical approach for matrices larger than .

✠

✄

✝

a0

✂

✁

✂

✁✬✔ Whether or not these issues are resolved in the near future, researchers can make use today of tests for orientation and incircle in two and three dimensions that are correct, fast in most cases, and applicable

✠

a0

☛

✔

✗

✄

✄

✄

✄

✗

☛

a0

✄

✄

✄

✄

a0

✄

☛

☛

☛

a0

✄

✄

✄

✟

✄

✄

☛

✄

✄

✄

✄

✄

✄

✄

✎

✄

✧

✠

✟

✄

✄

✄

✄

☛

☛

a0

✄

✄

✄

✄

a0

✄

☛

☛

☛

a0

✄

✄

✄

✟

✄

✂

✄

✁

✄

✄

✄

✂

✄

✄

✄

✄

☎

a0

✠

✆

✁

✄

✁

✠

✠

✠

✠

<!-- image -->

✆

✂

✍

✍

Figure 23: Operation of L INEAR -E XPANSION -S UM . ✁a0 maintains an approximate running total. The F AST -T WO -S UM operations in the bottom row exist to clip a high-order bit off each term, if necessary, before outputting it.

✍

a0

to single or double precision floating-point inputs. I invite working computational geometers to try my code in their implementations, and hope that it will save them from worrying about robustness so they may concentrate on geometry.

## A Linear-Time Expansion Addition without Round-to-Even Tiebreaking

✁

✂

☎

✭

✤

✂

✟

a0

✠

✠

✁

a0

✭

✳

✠

✠

✛

- L INEAR -E XPANSION -S UM ☞✁ ✚✎ 1 Merge and into a single sequence , in order of nondecreasing magnitude (possibly with interspersed zeroes) 2 ☞a0 ☎✄ F AST -T WO -S UM ✁a0 ✂a0 3 for to 4 ☛✌✄ F AST -T WO -S UM ✁a0 ☎✄ 5 ☞a0 ☎✄ ✠✭ T WO -S UM ☞a0 6 ☛✄ 7 ✓☞ 8 return

✭

✟

✄

☎

✳

✩

✤

✠

a0

☎

✳

✩

✳

✩

✤

☎

✳

✩

☞

☎

✠

- is an approximate sum of the first components of ; see Figure 23.

✟

✄

✟

✜✭ Proof: At the end of each iteration of the for loop, the invariant ✕✭ ✠✭ ☛✌✄ holds. Certainly this invariant holds for after Line 2 is executed. From Lines 4 and 5, we have that ✠✭

✯

a0

✂

✝

✝

✠

✝

✂

✝

✝

a0

✝

✁

a0

✧

✂

✩

✭

✤

a0

✭

✁

a0

✝

✞

Theorem 24 Let ✬✧ ✞✝ ✠✭ and ✞✝ be nonoverlapping expansions of and ❄✂ -bit components, respectively, where . Suppose that the components of both and are sorted in order of increasing magnitude, except that any of the ✆✭ or may be zero. Then the following algorithm will produce a nonoverlapping expansion such that ✞✝ , where the components of are also in order of increasing magnitude, except that any of the may be zero.

✁

a0

a0

a0

✁

a0

✁

✄

✁

✁

a0

✄

a0

✁

a0

✁

✁

✁

✫

a0

a0

✆

✄

✆

a0

✧

✆

✄

a0

✆

✂

✭

☎

✤

a0

✩

✁

✁

✠

✆

✭

✭

a0

✧

a0

✁

✄

✁

✁

a0

✄

a0

✁

a0

✁

✁

✁

✤

a0

a0

✄

✄

✄

a0

✁

✄

a0

a0

✤

✠

✆

✄

✯

✎

✆

✆

☞

☛

✄

✄

✝

✭

✭

✛

✎

✎

✯

☞

✠

✧

✭

✝

✝

a0

☞

✝

✟

☞

✞

✝

✆

✆

✭

☛

✄

✎

✎

✄

✤

✝

✆

✭

✝

✭

a0

✎

✭

☛

✤

✝

✭

✤

✧

✭

✤

a0

✭

✟

✄

✟

✠

☛✌✄ ✄a0 ✜✭ ✘✭ ; the invariant follows by induction. (The use of F AST -T WO -S UM in Line 4 will be justified shortly.) This assures us that after Lines 6 and 7 are executed, , so the algorithm produces a correct sum.

✠

✠

a0

✟

✄

a0

✂

✝

☎

✝

✳

✩

✠

✝

✧

✂

✝

☎

✝

✳

✩

a0

✝

✭

✧

✭

✤

✄

✤

a0

The proof that is nonoverlapping and increasing relies on the fact that the components of are summed in order from smallest to largest, so the running total ✕✭ ✜✭ never grows much larger than the next component to be summed. Specifically, I prove by induction that the exponent of ✕✭ is at most one greater than the exponent of , and the components ✎✠☎✠☎✠☎❁✎ are nonoverlapping and in order of increasing magnitude (excepting zeros). This statement holds for ✦✧ because ✖✌✗ . To prove the statement in the general case, assume (for the inductive hypothesis) that the exponent of is at most one greater than the exponent of , and the components ☛✌✄ are nonoverlapping and increasing.

✄

☛

✤

✖

✭

✟

✤

✄

✖

✄

✆

✝

✖

✗

✠

✭

✖

✧

✖

a0

✟

✄

☛

✖

a0

✆

a0

✱

✟

✔

✆

✝

Assume without loss of generality that the exponent of is ✂✓☛ , so that ulp ✁a0 , and ✎✠☎✠☎✠☎✆✎ are bounded below ✔✱ . Because is formed by merging two nonoverlapping increasing expansions, ✁a0 . Consider, for instance, if ✚✭ ✗✚✗✚✗ (in four-bit arithmetic); then can be no greater than the sum of and .

✖

a0

a0

✟

a0

☛

✤

✟

✔

a0

✭

✖

a0

✭

✄

✭

✖

✗

✯

✤

✖

a0

✤

✖

✧

✖

a0

✂

a0

✖

a0

✖

✗

✖

a0

☛

✖

✤

✚✭ ✎✠☎✠☎✠☎❁✎ ✠✭ is the roundoff error of the T WO -S UM operation that produces ✕✭ , so ✠✭ ✖✘✗ ulp ☞a0 . This inequality and the inductive hypothesis imply that ❁✭ ulp ✁a0 ✘✭ , which justifies the use of a F AST -T WO -S UM operation in Line 4. This operation produces the sum ☛✌✄ ✘✭ ✠✭ ✞✝ ulp ✁a0 ✘✭ . Corollary 8(a) implies that ☛✌✄ ulp ✁a0 . Because ✎✠☎✠☎✠☎❁✎ ☛✌✄ are nonoverlapping, we have the bound ☛✌✄ ulp ✁a0 ✘✭ ulp ✁a0 ✘✭ .

✎

✤

✤

✔

✧

✔

✭

✭

a0

✆

✝

✱

✟

a0

✱

☛

✖

a0

✖

✖

a0

✄

✖

✖

a0

✭

✳

✖

a0

✔✚✔✚✔✚✔✘☎✢✔✚✔✚✔✚✔ ☎✠☎✠☎ ✔✚✔✚✔✘☎✢✔✚✔✚✔✚✔ ☎✠☎✠☎ Substituting these bounds into the invariant given at the beginning of this proof, we have ☛✌✄ ✔✱ ✔✱ , which confirms that the exponent of is at most one greater than the exponent of .

✭

☛

✤

✠

a0

a0

✠

✂

✝

✭

✝

✠

✝

✖

✟

✖

✂

✝

✭

✝

a0

a0

✝

✖

To show that is larger than previous components of (or is zero) and does not overlap them, observe from Figure 23 that is formed (for ) by summing ✚✭ , , and . It can be shown that all three of these are either equal to zero or too large to overlap ☛✌✄ , and hence so is . We have already seen that ☛✌✄ ulp ✁a0 ✘✭ , which is bounded in turn by ulp ✁a0 ✚✭ . It is clear that ☛✌✄ is too small to overlap because both are produced by a F AST -T WO -S UM operation. Finally, ☛✌✄ is too small to overlap because ☛✌✄ ✖✌✗ (applying Lemma 1 to Line 4), and ulp ☞a0 .

a0

✆

✤

✝

✆

✳

✭

✝

✤

✤

✖

✠

✭

✭

✖

✤

✄

✖

✠

✭

✖

a0

✠✭ ❁✭ The foregoing discussion assumes that none of the input components is zero. If any of the is zero, the corresponding output component ☛✌✄ is also zero, and the accumulator values and are unchanged ( , ✜✭✮✧ ✜✭ ).

✭

✭

✧

a0

✭

☛

✤

✄

✄

☛

✤

## B Why the Tiebreaking Rule is Important

Theorem 13 is complicated by the need to consider the tiebreaking rule. This appendix gives an example that proves that this complication is necessary to ensure that F AST -E XPANSION -S UM will produce nonoverlapping output. If one's processor does not use round-to-even tiebreaking, one might use instead an algorithm that is independent of the tiebreaking rule, such as the slower L INEAR -E XPANSION -S UM in Appendix A.

Section 2.4 gave examples that demonstrate that F AST -E XPANSION -S UM does not preserve the nonoverlapping or nonadjacent properties. The following example demonstrates that, in the absence of any assumption about the tiebreaking rule, F AST -E XPANSION -S UM does not preserve any property that implies the nonoverlapping property. (As we have seen, the round-to-even rule ensures that F AST -E XPANSION -S UM preserves the strongly nonoverlapping property.)

✄

✝

✖

✂

✝

a0

✖

a0

a0

✟

☛

✤

✄

☛

✤

✄

✤

✭

☛

✤

✖

✂

✝

✭

✝

✤

✠

✝

✖

☛

a0

a0

✭

✟

✳

✆

✠

☛

✝

✟

✗

✖

✆

✠

✭

✳

✤

a0

✖

✝

a0

✆

✭

✝

✭

☛

☛

a0

✠

✤

✠

✤

a0

✠

✭

✳

✠

✄

✭

a0

✤

✄

✖

✖

✂

✤

✝

✭

✝

✤

✤

a0

a0

✖

✭

☛

✤

✭

✝

✭

✤

✠

✤

✤

✝

✭

✖

✳

✠

✤

✭

✠

✭

☛

✖

✄

☛

✤

✤

✖

✠

✯

✫

✛

✠

✳

✤

✧

✤

✳

✔

✖

✄

✄

☛

✭

✤

✖

✗

✤

✄

✤

a0

a0

✤

✖

✭

✠

✄

☛

✭

✆

✭

✖

☛

a0

a0

✳

✤

a0

✭

✝

☛

✆

✤

✁

✤

a0

✭

✝

✭

✗

✔

✁

a0

✤

✤

✟

a0

✤

✞

✟

✁

a0

✁

✟

a0

✆

✟

a0

✁

✟

a0

✔

✄

✟

✔

a0

a0

✤

✤

✁

✆

✟

a0

✔

a0

✔

✁

✤

✤

✍

✄

✔

✁

✟

a0

✔✚✔ ✔✚✔✚✔✚✔ Switching to the round-to-even rule, suppose F AST -E XPANSION -S UM is used to sum two copies of this expansion. The resulting 'expansion' is ✔✚✔✚✔ , which contains a pair of overlapping components. Hence, it is not safe to mix the round-toward-zero and round-to-even rules, and it is not possible to prove that F AST -E XPANSION -S UM produces nonoverlapping expansions for any tiebreaking rule.

For simplicity, assume that four-bit arithmetic is used. Suppose the round-toward-zero rule is initially in effect. The incompressible expansions and can each be formed by summing their components with any expansion addition algorithm. Summing these two expansions, F AST -E XPANSION -S UM (with zero elimination) yields the expansion ✗✚✗ . Similarly, one can form the expansion ✗✚✗ ✞✠✟✡a0 ✟✡a0 ✁✠✟ . Summing these two in turn yields , which is nonoverlapping but not strongly nonoverlapping.

✁

✞

✟

☛

a0

✁

✟

a0

✁

✟

☛

a0

a0

✤

✁

✟

☛

a0

✤

✤

✟

a0

✤

✤

Although the expansion above is not nonoverlapping, it is not particularly bad, in the sense that A P -PROXIMATE will nonetheless produce an accurate approximation of the expansion's value. It can be proven that, regardless of tiebreaking rule, F AST -E XPANSION -S UM preserves what I call the weakly nonoverlapping property, which allows only a small amount of overlap between components, easily fixed by compression. (Details are omitted here, but I am quite certain of the result. I produced a proof similar to that of Theorem 13, and rivalling it in complexity, before I discovered the strongly nonoverlapping property.) I conjecture that the geometric predicates of Section 4 work correctly regardless of tiebreaking rule.

## References

- [1] Francis Avnaim, Jean-Daniel Boissonnat, Olivier Devillers, Franco P. Preparata, and Mariette Yvinec. Evaluating Signs of Determinants Using Single-Precision Arithmetic . Algorithmica 17 (2):111-132, February 1997.
- [2] David H. Bailey. A Portable High Performance Multiprecision Package . Technical Report RNR-90022, NASA Ames Research Center, Moffett Field, California, May 1993.
- [3] C. Bradford Barber. Computational Geometry with Imprecise Data and Arithmetic . Ph.D. thesis, Department of Computer Science, Princeton University, Princeton, New Jersey, October 1992. Available as Technical Report CS-TR-377-92.
- [4] Christoph Burnikel, Jochen K¨ onemann, Kurt Mehlhorn, Stefan N¨ aher, Stefan Schirra, and Christian Uhrig. Exact Geometric Computation in LEDA . Eleventh Annual Symposium on Computational Geometry (Vancouver, British Columbia, Canada), pages C18-C19. Association for Computing Machinery, June 1995.
- [5] John Canny. Some Algebraic and Geometric Computations in PSPACE . 20th Annual Symposium on the Theory of Computing (Chicago, Illinois), pages 460-467. Association for Computing Machinery, May 1988.
- [6] Kenneth L. Clarkson. Safe and Effective Determinant Evaluation . 33rd Annual Symposium on Foundations of Computer Science (Pittsburgh, Pennsylvania), pages 387-395. IEEE Computer Society Press, October 1992.
- [7] T. J. Dekker. A Floating-Point Technique for Extending the Available Precision . Numerische Mathematik 18 :224-242, 1971.

✟

✔

✤

a0

✍

✟

a0

✆

✟

✁

✔

a0

✤

✤

✟

a0

a0

✄

✟

✟

a0

✄

a0

✟

a0

✟

a0

✆

✟

a0

✄

✟

✤

- [8] Steven Fortune. Stable Maintenance of Point Set Triangulations in Two Dimensions . 30th Annual Symposium on Foundations of Computer Science, pages 494-499. IEEE Computer Society Press, 1989.
- [9] . Progress in Computational Geometry . Directions in Geometric Computing (R. Martin, editor), chapter 3, pages 81-128. Information Geometers Ltd., 1993.
- [10] . Numerical Stability of Algorithms for 2D Delaunay Triangulations . International Journal of Computational Geometry &amp; Applications 5 (1-2):193-213, March-June 1995.
- [11] Steven Fortune and Christopher J. Van Wyk. Efficient Exact Arithmetic for Computational Geometry . Proceedings of the Ninth Annual Symposium on Computational Geometry, pages 163-172. Association for Computing Machinery, May 1993.
- [12] . Static Analysis Yields Efficient Exact Integer Arithmetic for Computational Geometry . ACM Transactions on Graphics 15 (3):223-248, July 1996.
- [13] David Goldberg. What Every Computer Scientist Should Know About Floating-Point Arithmetic . ACM Computing Surveys 23 (1):5-48, March 1991.
- [14] Leonidas J. Guibas and Jorge Stolfi. Primitives for the Manipulation of General Subdivisions and the Computation of Vorono¨ ı Diagrams . ACM Transactions on Graphics 4 (2):74-123, April 1985.
- [15] Christoph M. Hoffmann. The Problems of Accuracy and Robustness in Geometric Computation . Computer 22 (3):31-41, March 1989.
- [16] Michael Karasick, Derek Lieber, and Lee R. Nackman. Efficient Delaunay Triangulation Using Rational Arithmetic . ACMTransactions on Graphics 10 (1):71-91, January 1991.
- [17] Donald Ervin Knuth. The Art of Computer Programming: Seminumerical Algorithms , second edition, volume 2. Addison Wesley, Reading, Massachusetts, 1981.
- [18] Der-Tsai Lee and Bruce J. Schachter. Two Algorithms for Constructing a Delaunay Triangulation . International Journal of Computer and Information Sciences 9 (3):219-242, 1980.
- [19] Seppo Linnainmaa. Analysis of Some Known Methods of Improving the Accuracy of Floating-Point Sums . BIT 14 :167-202, 1974.
- [20] Victor Milenkovic. Double Precision Geometry: A General Technique for Calculating Line and Segment Intersections using Rounded Arithmetic . 30th Annual Symposium on Foundations of Computer Science, pages 500-505. IEEE Computer Society Press, 1989.
- [21] N. E. Mnev. The Universality Theorems on the Classification Problem of Configuration Varieties and Convex Polytopes Varieties . Topology and Geometry - Rohlin Seminar (O. Ya. Viro, editor), Lecture Notes in Mathematics, volume 1346, pages 527-543. Springer-Verlag, 1988.
- [22] Thomas Ottmann, Gerald Thiemt, and Christian Ullrich. Numerical Stability of Geometric Algorithms . Proceedings of the Third Annual Symposium on Computational Geometry, pages 119-125. Association for Computing Machinery, June 1987.
- [23] Douglas M. Priest. Algorithms for Arbitrary Precision Floating Point Arithmetic . Tenth Symposium on Computer Arithmetic (Los Alamitos, California), pages 132-143. IEEE Computer Society Press, 1991.

- [24] . On Properties of Floating Point Arithmetics: Numerical Stability and the Cost of Accurate Computations . Ph.D. thesis, Department of Mathematics, University of California at Berkeley, Berkeley, California, November 1992. Available by anonymous FTP to ftp.icsi.berkeley.edu as pub/theory/priest-thesis.ps.Z .
- [25] Jonathan Richard Shewchuk. Triangle: Engineering a 2D Quality Mesh Generator and Delaunay Triangulator . Applied Computational Geometry: Towards Geometric Engineering (Ming C. Lin and Dinesh Manocha, editors), Lecture Notes in Computer Science, volume 1148, pages 203-222. SpringerVerlag, May 1996. From the First ACM Workshop on Applied Computational Geometry.
- [26] Pat H. Sterbenz. Floating-Point Computatation . Prentice-Hall, Englewood Cliffs, New Jersey, 1974.

✞

- [27] David F. Watson. Computing the -dimensional Delaunay Tessellation with Application to Vorono¨ ı Polytopes . Computer Journal 24 (2):167-172, 1981.
- [28] James Hardy Wilkinson. Rounding Errors in Algebraic Processes . Prentice-Hall, Englewood Cliffs, New Jersey, 1963.