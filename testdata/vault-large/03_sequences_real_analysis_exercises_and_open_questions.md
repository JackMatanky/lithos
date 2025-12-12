---
title: 03_sequences_real_analysis_exercises_and_open_questions
uuid: 0f568a2c-5d0b-4399-928f-7d0a7a79aedf
aliases:
  - "Real Analysis: Sequences, Exercises and Open Questions"
  - "Sequences: Exercises and Open Questions"
  - "3. Sequences: Exercises and Open Questions"
  - sequences_exercises_and_open_questions
  - real_analysis_sequences_exercises
  - 03_sequences_real_analysis_exercises_and_open_questions
main_title: Sequences
subtitle: Exercises and Open Questions
author:
  - "[[cummings_jay|Jay Cummings]]"
editor:
translator:
year_published: 2019
publisher:
page_start: 65
page_end: 116
doi:
url: https://longformmath.com/analysis-home
library:
  - "[[cummings_2019_real_analysis|Real Analysis: A Long-form Mathematics Textbook]]"
cssclasses:
status: done
type: book_chapter
file_class: lib_book_chapter
date_created: 2024-12-22T19:42
date_modified: 2025-10-05T17:48
tags:
---
# 3. Sequences: Exercises and Open Questions

> [!book_chapter] Book Chapter Details
>
> - **Author**: `dv: this.file.frontmatter.author`
> - **Chapter**: `dv: this.file.frontmatter.aliases[0]`
> - **Book**: `dv: this.file.frontmatter.library[0]`
> - **Publisher**: `dv: this.file.frontmatter.publisher`
> - **Date Published**: `dv: this.file.frontmatter.year_published`
> - **Pages**: `dv: this.file.frontmatter.page_start + " - " + this.file.frontmatter.page_end`
>
> - **Completed**:: [[2025-01-19]]

---

<!-- Insert chapter content here -->

![[Cummings_2019_Real Analysis_03_Sequences.pdf]]

---

## Exercise 3.1

Suppose that a sequence $\langle x_{n} \rangle$ converges to 0.001. Prove that finitely many values of $x_{n}$ are negative.

### Solution

1\. Let $\langle x_{n} \rangle$ be a sequence such that

$$
\lim_{n \to \infty} x_{n} = 0.001.
$$

2\. By definition of sequence **convergence**, $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies |x_{n} - 0.001| < \varepsilon
$$

3\. Define $\varepsilon = 0.001.$ Then, $\exists N \in \mathbb{N}$ such that for all $n \geq N,$

$$
\begin{gather}
|x_{n} - 0.001| < 0.001 \\
-0.001 < x_{n} - 0.001 < 0.001 \\
0 < x_{n} < 0.002
\end{gather}
$$

4\. Hence, for all indices $n \geq N,$ the terms $x_{n}$ are **strictly positive**.

5\. Since there are only finitely many indices $n < N,$ the number of negative terms $x_{n}$ in the sequence is at most $N-1,$ which is **finite**.

$$
\therefore ~ \boxed{\text{Thus, only finitely many values of } x_{n} \text{ are negative.}}
$$

---

## Exercise 3.2

Give an example satisfying the requested condition or prove that no such example can exist:

1. A sequence with infinitely many 0s that does not converge to 0.
2. A sequence with infinitely many 0s that converges to a non-zero number.
3. A sequence of positive numbers that converges to a negative number.
4. A sequence of irrational numbers that converges to a rational number.

---

## Exercise 3.3

This problem is to help you get a feel for the quantifiers in the definition of convergence (Definition 3.7). Your task: For each of the following definitions of *Nonvergence*, give an example of a sequence $\langle x_{n} \rangle$ and a value $a$ for which:

1. $\langle x_{n} \rangle$ does **not** converge to $L$ (based on the real definition).
2. $\langle x_{n} \rangle$ does *Nonverge* to $L$ based on the definition given below.

Give a different example for each problem. For each of them, explain why your example works in a few sentences (no need to prove it completely). Your example for *Nonverges-type-4* should not work for *Nonverges-type-3*.

1. **Definition 1**:
   The sequence $\langle x_{n} \rangle$ *Nonverges-type-1* to $L$ if for all $\varepsilon > 0$ there exists some $n \in \mathbb{N}$ such that $|x_{n} - L| < \varepsilon.$

2. **Definition 2**:
   The sequence $\langle x_{n} \rangle$ *Nonverges-type-2* to $L$ if for all $\varepsilon > 0$ there exists some $N \in \mathbb{N}$ such that, for some $n > N,$ we have $|x_{n} - L| < \varepsilon.$

3. **Definition 3**:
   The sequence $\langle x_{n} \rangle$ *Nonverges-type-3* to $L$ if there exists some $\varepsilon > 0$ such that for all $N \in \mathbb{N}$ there exists some $n > N$ such that $|x_{n} - L| < \varepsilon.$

4. **Definition 4**:
   The sequence $\langle x_{n} \rangle$ *Nonverges-type-4* to $L$ if there exists some $\varepsilon > 0$ and there exists some $N \in \mathbb{N}$ such that for all $n > N$ we have $|x_{n} - L| < \varepsilon.$

---

## Exercise 3.4

Prove the following using the definition of sequence convergence:

### 3.4.1

Let $\langle x_{n} \rangle = 7 - \frac{1}{\sqrt{n}}.$ Show that $x_{n} \to 7$ as $n \to \infty.$

#### Solution

1\. Goal: We must show that, $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies |x_{n} - 7| < \varepsilon
$$

2\. Evaluate and bound $|x_{n} - 7|$:

$$
|x_{n} - 7| = \left| 7 - \frac{1}{\sqrt{n}} - 7 \right| = \frac{1}{\sqrt{n}}.
$$

3\. To satisfy $\frac{1}{\sqrt{n}} < \varepsilon,$ we solve:

$$
\begin{align}
\frac{1}{\sqrt{n}} &< \varepsilon \\
\sqrt{n} &> \frac{1}{\varepsilon}  \\
n &> \frac{1}{\varepsilon^{2}}
\end{align}
$$

Choose $N = \left\lceil  \frac{1}{\varepsilon^{2}}  \right\rceil.$ Then, for all $n \geq N,$

$$
\frac{1}{\sqrt{n}} < \varepsilon.
$$

**Conclusion**: By definition of convergence, $x_{n} \to 7$ as $n \to \infty.$

$$
\therefore ~ \boxed{\langle x_{n} \rangle = 7 - \frac{1}{\sqrt{n}} \implies \lim_{ n \to \infty } x_{n} = 7}
$$

### 3.4.2

Let $x_{n} = \frac{2n}{5n + 1}.$ Show that $\lim\limits_{ n \to \infty } x_{n} = \frac{2}{5}.$

#### Solution

1\. Goal: We must show that, $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that if $n \geq N,$

$$
\left| x_{n} - \frac{2}{5} \right| < \varepsilon.
$$

2\. Evaluate and bound $|x_{n} - \frac{2}{5}|$:

$$
\left| \frac{2n}{5n + 1} - \frac{2}{5} \right|
= \left| \frac{10n - (10n + 2)}{5(5n + 1)} \right|
= \left| \frac{-2}{5(5n + 1)} \right|
= \frac{2}{5(5n + 1)}.
$$

3\. To satisfy $\frac{2}{5(5n + 1)} < \varepsilon,$ solve:

$$
\begin{align}
\frac{2}{5(5n + 1)} &< \varepsilon  \\
5(5n + 1) &> \frac{2}{\varepsilon} \\
5n + 1 &> \frac{2}{5\varepsilon} \\
n &> \frac{2}{5\varepsilon} - \frac{1}{5}
\end{align}
$$

4\. Choose $N = \left\lceil \frac{2}{5\varepsilon} - \frac{1}{5} \right\rceil.$ Then, for all $n \geq N,$

$$
\frac{2}{5(5n + 1)} < \varepsilon.
$$

**Conclusion**: By definition of convergence, $x_{n} \to \frac{2}{5}$ as $n \to \infty.$

$$
\therefore ~ \boxed{\langle x_{n} \rangle = \frac{2n}{5n + 1} \implies \lim_{ n \to \infty } x_{n} = \frac{2}{5}}
$$

### 3.4.3

Let $x_{n} = 7 - \frac{1}{\sqrt{n + \sqrt{n + 13}}}.$ Show that $x_{n} \to 7$ as $n \to \infty.$

#### Solution

Goal: We must show that, $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that if $n \geq N,$

$$
|x_{n} - 7| < \varepsilon
$$

Evaluate and bound $|x_{n} - 7|$:

$$
|x_{n} - 7| = \left| 7 - \frac{1}{\sqrt{n + \sqrt{n + 13}}} - 7 \right| = \frac{1}{\sqrt{n + \sqrt{n + 13}}}.
$$

Since $n + \sqrt{n + 13} > n,$ we get:

$$
\sqrt{n + \sqrt{n + 13}} > \sqrt{n}.
$$

Thus,

$$
\frac{1}{\sqrt{n + \sqrt{n + 13}}} < \frac{1}{\sqrt{n}}.
$$

**Step 3: Choosing $N$**

Since we already showed that $\frac{1}{\sqrt{n}} \to 0,$ we can choose $N = \left\lceil \frac{1}{\varepsilon^{2}} \right\rceil$ so that for all $n \geq N,$

$$
\frac{1}{\sqrt{n}} < \varepsilon.
$$

Since $\frac{1}{\sqrt{n + \sqrt{n + 13}}} < \frac{1}{\sqrt{n}},$ it follows that

$$
|x_{n} - 7| < \varepsilon.
$$

**Conclusion**: By definition of convergence, $x_{n} \to 7$ as $n \to \infty.$

---

## Exercise 3.5

Give an example of a sequence $\langle x_{n} \rangle$ where $x_{n}$ is negative for all $n,$ and yet $x_{n} \to 0.$

### Solution

$$
\langle x_{n} \rangle = -\frac{1}{n}
$$

---

## Exercise 3.6

### 3.6.1

Consider the sequence:

$$
\frac{1}{2}, \frac{1}{3}, \frac{2}{3}, \frac{1}{4}, \frac{2}{4}, \frac{3}{4}, \frac{1}{5}, \frac{2}{5}, \frac{3}{5}, \frac{4}{5}, \frac{1}{6}, \frac{2}{6}, \frac{3}{6}, \frac{4}{6}, \frac{5}{6}, \frac{1}{7}, \frac{2}{7}, \frac{3}{7}, \frac{4}{7}, \frac{5}{7}, \frac{6}{7}, \dots
$$

For which numbers $L$ does the above sequence have a subsequence converging to $L$?

#### Solution

1\. We analyze the set of subsequential limits of the sequence

$$
\frac{1}{2}, \frac{1}{3}, \frac{2}{3}, \frac{1}{4}, \frac{2}{4}, \frac{3}{4}, \frac{1}{5}, \frac{2}{5}, \frac{3}{5}, \frac{4}{5}, \dots.
$$

2\. Observing the pattern, the general term of the sequence is given by

$$
x_{n} = \frac{k}{m} \quad \text{for } 1 \leq k < m.
$$

where each fraction $\frac{k}{m}$ corresponds to rational numbers in the unit interval $(0,1)$ when written in simplest form.

##### Step 1: Accumulation Behavior

- For each $m,$ the values $\frac{k}{m}$ are distributed in the interval $(0,1).$
- As $m \to \infty,$ the denominators grow arbitrarily large, producing fractions that densely cover $(0,1).$
- Given any $L \in (0,1),$ we can find a subsequence $\frac{k_{n}}{m_{n}}$ with $m_{n} \to \infty$ and $\frac{k_{n}}{m_{n}} \to L,$ ensuring that all points in $(0,1)$ are subsequential limits.

##### Step 2: Boundary Behavior

- The sequence never attains $0$ or $1$ directly, but for any $\varepsilon > 0,$ we can find terms $\frac{1}{m}$ and $\frac{m-1}{m}$ arbitrarily close to 0 and 1, respectively.
- Thus, subsequences approaching $0$ and $1$ exist.

##### Conclusion

The set of limit points of the sequence is the closed interval $[0,1],$ meaning that for every $L \in [0,1],$ there exists a subsequence converging to $L.$

$$
\boxed{\text{The set of subsequential limits is } [0,1].}
$$

### 3.6.2

Does there exist a sequence $\langle x_{n} \rangle,$ where, for every $L \in \mathbb{R},$ there exists a subsequence of $\langle x_{n} \rangle$ that converges to $L$?

#### Solution

1\. We need to determine whether a sequence $\langle x_{n} \rangle$ exists such that for **every** $L \in \mathbb{R},$ there is a subsequence $x_{n_{k}}$ satisfying:

$$
\lim_{k \to \infty} x_{n_{k}} = L.
$$

##### **Step 1: Constructing Such a Sequence**

2\. To ensure that every real number is a limit of some subsequence, the sequence must:
- Be **dense** in $\mathbb{R}.$
- Contain terms diverging to $\pm \infty.$

3\. A suitable construction is:

1. **Enumerate the rational numbers $\mathbb{Q}$:** Let $\{q_{n}\}$ be an enumeration of all rational numbers.
2. **Interleave with divergent terms $\pm n$:** Define:

$$
x_{n} = q_{n} \quad \text{(rational enumeration)}, \quad x_{2n} = n, \quad x_{2n+1} = -n.
$$

##### **Step 2: Extracting Subsequences**

4\. Since the rationals are **dense** in $\mathbb{R},$ for any real $L,$ we can extract a subsequence of rationals $q_{n_{k}}$ such that:

$$
\lim_{k \to \infty} q_{n_{k}} = L.
$$

5\. Since the terms $x_{2n} = n$ and $x_{2n+1} = -n$ ensure that subsequences tend to $\infty$ and $-\infty,$ we also obtain limit points at $\pm \infty.$

##### **Step 3: Conclusion**

6\. The constructed sequence ensures that for **every** $L \in \mathbb{R},$ there is a subsequence that converges to $L.$

$$
\boxed{\text{Yes, such a sequence } \langle x_{n} \rangle \text{ exists.}}
$$

---

## Exercise 3.7

Each of the following is an independent question, but for each suppose that the sequence $\langle x_{n} \rangle$ has the property that $x_{n} \in \mathbb{Z}$ for all $n.$

### 3.7.1

Is it possible that $x_{n} \to 3.5$?

#### Solution

1\. By definition of sequence convergence, $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that if $n \geq N,$

$$
|x_{n} - 3.5| < \varepsilon.
$$

2\. Since $x_{n} \in \mathbb{Z},$ the absolute difference $|x_{n} - 3.5|$ can take values at least $0.5,$ meaning that we cannot make $|x_{n} - 3.5|$ arbitrarily small.

3\. Choosing $\varepsilon = 0.25,$ we see that for any $n,$ $|x_{n} - 3.5| \geq 0.5 > \varepsilon,$ contradicting the definition of convergence.

$$
\boxed{\text{No, } x_{n} \text{ cannot converge to } 3.5.}
$$

### 3.7.2

If $x_{n} \neq x_{m}$ for all $n \neq m,$ prove that $\langle x_{n} \rangle$ does not converge.

#### Solution

1\. Assume, for **contradiction**, that $\langle x_{n} \rangle$ converges to some $L.$

2\. By definition of sequence **convergence**, $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that if $n \geq N,$

$$
|x_{n} - L| < \varepsilon
$$

3\. Since $x_{n}$ is an integer-valued sequence, the inequality implies that for sufficiently large $n,$ all terms $x_{n}$ must be within an arbitrarily small interval around $L.$

4\. However, the assumption that $x_{n} \neq x_{m}$ for all $n \neq m$ implies that $x_{n}$ takes infinitely many distinct integer values, contradicting the fact that all sufficiently large $x_{n}$ must be confined to a small interval.

5\. Thus, no such $L$ can exist, and the sequence cannot converge.

$$
\boxed{\text{The sequence } \langle x_{n} \rangle \text{ does not converge.}}
$$

### 3.7.3

If $\langle x_{n} \rangle$ converges, what can be said about this sequence?

#### Solution

1\. Suppose $\langle x_{n} \rangle$ is a real, convergent sequence, such that:

$$
\lim_{ n \to \infty } x_{n} = L
$$

2\. By definition of sequence **convergence**, $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that if $n \geq N,$

$$
|x_{n} - L| < \varepsilon
$$

2\. Since $x_{n}$ is an integer-valued sequence, for sufficiently small $\varepsilon,$ the only integer satisfying this inequality is $L$ itself, which must also be an integer.

3\. Thus, for sufficiently large $n,$ we must have $x_{n} = L,$ meaning the sequence is eventually constant.

$$
\boxed{\text{If } \langle x_{n} \rangle \text{ converges, it must eventually be constant.}}
$$

---

## Exercise 3.8: Real Sequence Limit Sum and Scalar Multiplication Properties

Assume that $\{a_{n}\}$ converges to $a$ and $\{b_{n}\}$ converges to $b.$ Also assume $c \in \mathbb{R}.$

### Sum Rule

Prove that $\{a_{n} + b_{n}\}$ converges to $a + b.$

### Scalar Multiplication Rule

Prove that $\{c \cdot a_{n}\}$ converges to $c \cdot a.$

---

## Exercise 3.9

Prove that if $\{a_{n}\}$ and $\{b_{n}\}$ are sequences where $a_{n} \to \infty$ and $b_{n} \to \infty,$ then $\{a_{n} + b_{n}\} \to \infty.$

### Solution

1\. Suppose $\{a_{n}\}$ and $\{b_{n}\}$ are real, divergent sequences, such that:

$$
\lim_{ n \to \infty } a_{n} = \infty \quad\text{and}\quad \lim_{ n \to \infty } b_{n} = \infty
$$

2\. By definition of sequence **divergence**, $\forall M > 0$:

- $a_{n} \to \infty$ implies there exists $N_{1} \in \mathbb{N},$ for all $n \in \mathbb{N},$ such that $n \geq N_{1} \implies a_{n} > \frac{M}{2}.$
- $b_{n} \to \infty$ implies there exists $N_{2} \in \mathbb{N},$ for all $n \in \mathbb{N},$ such that $n \geq N_{2} \implies b_{n} > \frac{M}{2}.$

3\. Define $N = \max(N_{1}, N_{2}),$ such that $\forall n \geq N,$ the inequalities hold:

$$
a_{n} > \frac{M}{2} \quad\text{and}\quad b_{n} > \frac{M}{2}
$$

4\. Hence, for any $n \geq N,$

$$
a_{n} + b_{n} > \frac{M}{2} + \frac{M}{2} = M.
$$

5\. Thus, by definition of sequence **divergence**, $a_{n} + b_{n} \to \infty,$ since $\forall M > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that

$$
n \geq N \implies a_{n} + b_{n} > M
$$

$$
\therefore ~ \boxed{\lim_{ n \to \infty } (a_{n} + b_{n}) = \infty}
$$

---

## Exercise 3.10

Assume that $\{x_{2k}\}$ converges to $L$ and $\{x_{2k - 1}\}$ converges to $L.$ Prove that $\langle x_{n} \rangle$ also converges to $L.$

### Solution

1\. Suppose $\langle x_{n} \rangle$ is a real sequence with two convergent subsequences, $\{ x_{2k} \}$ and $\{ x_{2k - 1} \},$ such that:

$$
\lim_{ k \to \infty } x_{2k} = L \quad\text{and}\quad \lim_{ k \to \infty } x_{2k - 1} = L
$$

2\. By definition of sequence **convergence**, given any $\varepsilon > 0$:
- $\{x_{2n}\} \to L$ implies $\exists N_{1} \in \mathbb{N},$ $\forall k \in \mathbb{N},$ such that $k \geq N_{1} \implies |x_{2k} - L| < \varepsilon.$
- $\{x_{2n - 1}\} \to L$ implies $\exists N_{2} \in \mathbb{N},$ $\forall k \in \mathbb{N},$ such that $k \geq N_{2} \implies |x_{2k - 1} - L| < \varepsilon.$

3\. Define $N = \max(2N_{1}, 2N_{2} - 1)$ and consider any $n \geq N.$

4\. For any $n \geq N,$ there exists some $k \in \mathbb{N}:$

- If $n$ is even, then $n = 2k.$ Substituting $2k$ for $n$, $2k > N > 2N_{1},$ implying $k > N_{1}.$ Therefore, $|x_{2k} - L| = |x_{n} - L| < \varepsilon.$
- If $n$ is odd, then $n = 2k - 1.$ Substituting $2k - 1$ for $n$, $2k - 1 > N > 2N_{2} - 1,$ implying $k > N_{2}.$ Therefore, $|x_{2k - 1} - L| = |x_{n} - L| < \varepsilon.$

5\. Thus, by definition of sequence **convergence**, $x_{n} \to L,$ since $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that

$$
n \geq N \implies |x_{n} - L| < \varepsilon,
$$

$$
\therefore ~ \boxed{\lim_{ n \to \infty } x_{n} = L}
$$

---

## Exercise 3.11

Prove or disprove: If $\langle x_{n} \rangle$ converges, then the set $\{x_{n}: n \in \mathbb{N}\}$ of values the sequence takes has a maximum.

> [!Definition]
>
> A **maximum element** of $S$ is an element $M \in S$ such that:
>
> $$
> \forall x_{n} \in S [ \exists M \in S(M \geq x_{n})]
> $$

### Solution

**False**

1\. Consider the real sequence, $\langle x_{n} \rangle = 1 - \frac{1}{n}$, which converges to 1:

$$
\lim_{n \to \infty} x_{n} = 1.
$$

2\. The set of values taken by the sequence is:

$$
S = \left\{ 1 - \frac{1}{n} : n \in \mathbb{N} \right\}.
$$

3\. This set is **bounded above** by 1, but 1 is **never actually attained** by any $x_{n}$.

4\. Thus, although $S$ has a supremum of 1, the set does not contain a **maximum element**.

---

## Exercise 3.12

Suppose $\langle x_{n} \rangle$ is a sequence and $f: \mathbb{N} \to \mathbb{N}$ is a bijection. For each of the following, prove that the statement is true or find a counterexample showing that the statement is false:

### 3.12.1

If $\langle x_{n} \rangle$ diverges to $\infty,$ then $\langle x_{f(n)} \rangle$ diverges to $\infty.$

#### Solution

1\. Suppose $\langle x_{n} \rangle$ is a real, divergent sequence and let $f$ be a bijective function, such that:

$$
\lim_{ n \to \infty }  x_n = \infty \quad \text{and} \quad f \colon \mathbb{N} \to \mathbb{N}.
$$

2\. By definition of **sequence divergence to infinity**, $x_{n} \to \infty$ implies $\forall M > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies x_n > M
$$

3\. By the **surjectivity of bijection**, for $f: \mathbb{N} \to \mathbb{N},$ every natural number appears in the range of $f(n):$

$$
\forall k \in \mathbb{N} \bigl[ \exists n \in \mathbb{N} (f(n) = k) \bigr]
$$

4\. Because $\mathbb{N}$ is **unbounded above**, and $f(n)$ takes every value in $\mathbb{N}$ exactly once, it follows that $f(n) \to \infty,$ such that $\forall M > 0,$ $\exists K \in \mathbb{N},$ $\forall n \in \mathbb{N},$ where

$$
n \geq K \implies f(n) > M
$$

5\. Since $f(n) \to \infty,$ $\forall N \in \mathbb{N},$ $\exists N' \in \mathbb{N},$ such that $\forall n \in \mathbb{N}:$

$$
n \geq N' \implies f(n) \geq N
$$

6\. Thus, by definition of **sequence divergence to infinity**, $x_{f(n)} \to \infty$ since $\forall M > 0,$ $\exists N' \in \mathbb{N},$ $\forall n \in \mathbb{N},$

$$
n \geq N' \implies x_{f(n)} > M
$$

$$
\therefore ~ \boxed{\lim_{n \to \infty }x_n = \infty \implies \lim_{n \to \infty } x_{f(n)} = \infty}
$$

### 3.12.2

If $\langle x_{n} \rangle$ converges to $L,$ then $\langle x_{f(n)} \rangle$ converges to $L.$

#### Solution

1\. Suppose $\langle x_{n} \rangle$ is a real, convergent sequence and let $f$ be a bijective function, such that:

$$
\lim_{n\to \infty } x_{n} = L \quad \text{and} \quad f \colon \mathbb{N} \to \mathbb{N}
$$

2\. By definition of **sequence convergence**, $\forall \varepsilon > 0,$ $\exists N_{1} \in \mathbb{N},$ $\forall n \in \mathbb{N}$ such that:

$$
n \geq N_{1} \implies |x_{n} - L| < \varepsilon \tag{1}
$$

3\. By the **surjectivity of bijection**, for $f: \mathbb{N} \to \mathbb{N},$ every $k \in \mathbb{N}$ appears in the range of $f(n)$:

$$
\forall k \in \mathbb{N} \bigl[ \exists n \in \mathbb{N} (f(n) = k) \bigr]
$$

4\. By the **injectivity of bijection**, no two inputs share the same output:

$$
\forall n, m \in \mathbb{N} \left[ f(n) = f(m) \implies n = m \right]
$$

5\. Since $\mathbb{N}$ is unbounded above and $f(n)$ is a bijection, it follows that $f(n) \to \infty,$ such that $\forall M > 0,$ $\exists K \in \mathbb{N},$ $\forall n \in \mathbb{N}$ where:

$$
n \geq K \implies f(n) > M
$$

6\. Since $f(n) \to \infty$, for $N_{1} \in \mathbb{N},$ $\exists N_{2} \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N_{2} \implies f(n) \geq N_{1}
$$

7\. Thus, by definition of **sequence convergence**, $x_{f(n)} \to L$ since $\forall \varepsilon > 0,$ $\exists N_{2} \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
f(n) \geq N_1 \implies |x_{f(n)} - L| < \varepsilon
$$

$$
\therefore ~ \boxed{ \lim_{n \to \infty} x_n = L \implies \lim_{n \to \infty} x_{f(n)} = L }
$$

### 3.12.3

If $\langle x_{n} \rangle$'s limit does not exist, then $\langle x_{f(n)} \rangle$'s limit does not exist.

#### Solution

1\. Suppose $\{x_n\}$ is a real, **non-convergent sequence**, and let $f$ be a **bijective function**, such that:

$$
f \colon \mathbb{N} \to \mathbb{N}.
$$

2\. Assume, for **contradiction**, $\langle x_{f(n)} \rangle$ is a real, convergent sequence, such that:

$$
\lim_{ n \to \infty }  x_{f(n)} = L
$$

3\. By definition of sequence **convergence**, $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies |x_{f(n)} - L| < \varepsilon \tag{1}
$$

4\. By definition of **bijective functions**, $f$ is both **injective** and **surjective**:
- **Surjectivity:** $\forall k \in \mathbb{N},$ $\exists n \in \mathbb{N},$ such that $f(n) = k$.
- **Injectivity:** $\forall n_{1}, n_{2} \in \mathbb{N},$ if $f(n_{1}) = f(n_{2}),$ then $n_{1} = n_{2},$ ensuring $f$ is a **one-to-one mapping**.

5\. Since $f: \mathbb{N} \to \mathbb{N}$ is a bijection and by the Archimedean Principle, $\mathbb{N}$ is unbounded above, it follows that $f(n) \to \infty,$ such that $\forall M > 0,$ $\exists K \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq K \implies f(n) > M
$$

6\. For any $N \in \mathbb{N}$, since $f(n) \to \infty$, there exists a uniquely defined index $N' = f^{-1}(N)$, such that, $\forall n \in \mathbb{N}:$

$$
n \geq N' \implies f(n) \geq N \tag{2}
$$

7\. Substituting $(2)$ into $(1),$ then $\forall \varepsilon > 0,$ $\exists N' \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N' \implies |x_{f(n)} - L| < \varepsilon \tag{3}
$$

8\. By definition of **bijectivity**, the sequence $\{ x_{f(n)} \}$ is a reordering of $\{x_n\},$ such that:

$$
\{ x_{n} \mid n \in \mathbb{N} \} = \{ x_{f(n)} \mid n \in \mathbb{N} \}
$$

9\. Therefore, $x_{n}$ can replace $x_{f(n)}$ and also satisfies $(3)$ such that $\forall \varepsilon > 0,$ $\exists N' \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N' \implies |x_{n} - L| < \varepsilon
$$

10\. However, this contradicts our assumption that $\{ x_n \}$ **does not converge**.

$$
\therefore ~ \boxed{\text{If } x_n \text{ does not converge, then } x_{f(n)} \text{ does not converge.}}
$$

---

### Divergence of a Natural Bijection

Let $f \colon \mathbb{N} \to \mathbb{N}$ be a bijection. Then:

$$
\lim_{n \to \infty} f(n) = \infty
$$

That is:

$$
\forall M > 0,\ \exists K \in \mathbb{N},\ \forall n \in \mathbb{N},\ n \geq K \implies f(n) > M
$$

#### Proof by Contradiction

1\. Assume, for contradiction, that $f(n)$ does not diverge to infinity.

2\. By the contrapositive of **sequence divergence to infinity**, $f(n) \not\to \infty$ implies $\exists M > 0,$ $\forall K \in \mathbb{N},$ $\exists n \in \mathbb{N},$ such that:

$$
n \geq K \quad \text{and} \quad f(n) \leq M
$$

3\. Construct a strictly increasing, infinite sequence of indices $\langle n_{i} \rangle$ such that $\forall i \in \mathbb{N}:$

$$
\begin{align}
n_{1} &\implies f(n_{1}) \leq M \\
n_{2} > n_{1} &\implies f(n_{2}) \leq M \\
&\vdots \\
n_{i} > n_{i - 1} &\implies f(n_{i}) \leq M
\end{align}
$$

4\. Observe that the values $f(n_{i})$ are all contained in the finite set:

$$
\{1, 2, \dots, \lfloor M \rfloor\}
$$

5\. By the **pigeonhole principle**, since $\langle f(n_i) \rangle$ is an infinite sequence and its values lie in a finite set, at least one value $m \in [1, \lfloor M \rfloor]$ must occur more than once:

$$
\exists i, j \in \mathbb{N} \left[ i \neq j \land f(n_{i}) = f(n_{j}) \right]
$$

6\. By definition of **bijective functions as injective**:

$$
f(n) = f(m) \implies n = m
$$

7\. However, this contradicts that $n_{i}$ and $n_{j}$ are distinct inputs such that $f(n_{i}) = f(n_{j}) = m.$

$$
\therefore ~ \boxed{ \lim_{n \to \infty} f(n) = \infty }
$$

---

#### Direct Proof

1. Suppose $f \colon \mathbb{N} \to \mathbb{N}$ is a bijection.
2. Fix $M > 0$ and define:

$$
A := \{ n \in \mathbb{N} \mid f(n) \leq M \}
$$

1. By the **surjectivity of bijection**, every $k \in \mathbb{N}$ is the image of some $n \in \mathbb{N}$:

$$
\forall k \in \mathbb{N} \left[ \exists n \in \mathbb{N} \left( f(n) = k \right) \right] \tag{1}
$$

1. By the **injectivity of bijection**, each $k \in \mathbb{N}$ is the image of at most one $n \in \mathbb{N}$:

$$
\forall n, m \in \mathbb{N} \left[ f(n) = f(m) \implies n = m \right] \tag{2}
$$

1. 5. By (1) and (2), each $k \in \{1, 2, \dots, \lfloor M \rfloor\}$ is the image of a unique $n \in \mathbb{N}$. Hence:

$$
A \subseteq \mathbb{N} \text{ is finite}
$$

1. Let $K \in \mathbb{N}$ be the smallest index such that all $n \geq K$ lie outside $A,$ ensuring $f(n) > M$ for all $n \geq K$:

$$
K :=
\begin{cases}
1, & \text{if } A = \emptyset \\
\max A + 1, & \text{otherwise}
\end{cases}
$$

1. Then $\forall n \in \mathbb{N}:$

$$
n \geq K \implies f(n) > M
$$

1. Since $M > 0$ was arbitrary, it follows that:

$$
\therefore ~ \boxed{ \lim_{n \to \infty} f(n) = \infty }
$$

---

## Exercise 3.13: Real Sequence Limit Difference and Quotient Properties

Assume that $\{a_{n}\}$ converges to $a$ and $\{b_{n}\}$ converges to $b.$ Also assume $c \in \mathbb{R}.$

1. Prove that $\{a_{n} - b_{n}\}$ converges to $a - b.$
2. Prove that $\left\{\frac{a_{n}}{b_{n}}\right\}$ converges to $\frac{a}{b},$ provided $b \neq 0$ and each $b_{n} \neq 0.$

---

## Exercise 3.14

Let $a_{0} = 2 \sqrt{3}$ and $b_{0} = 3,$ and define two sequences recursively by:

$$
\{ a_{n} \} = \frac{2a_{n-1} \cdot b_{n-1}}{a_{n-1} + b_{n-1}} \quad \text{and} \quad \{ b_{n} \} = \sqrt{a_{n} \cdot b_{n-1}}.
$$

### 3.14.1

Prove that $\{a_{n}\}$ is monotonically decreasing and is convergent.

#### Solution

##### **Step 1: Prove $a_{n}$ is decreasing**

To show $a_{n}$ is decreasing, we need to prove:

$$
a_{n} \leq a_{n-1} \quad \forall n \geq 1.
$$

Using the recursive definition,

$$
a_{n} = \frac{2a_{n-1} b_{n-1}}{a_{n-1} + b_{n-1}}.
$$

Since $a_{0} > b_{0}$, we prove by induction that $a_{n} \geq b_{n}$ for all $n$:

**Base case:** $a_{0} = 2\sqrt{3} > 3 = b_{0}$, so $a_{0} > b_{0}$.

**Inductive step:** Assume $a_{n-1} > b_{n-1}$, then since the function

$$
f(a, b) = \frac{2ab}{a + b}
$$

is decreasing in $a$ for $a > b$, it follows that

$$
a_{n} = \frac{2a_{n-1} b_{n-1}}{a_{n-1} + b_{n-1}} < a_{n-1}.
$$

Thus, $\{a_{n}\}$ is **monotonically decreasing**.

##### **Step 2: Prove $a_{n}$ is Bounded below**

Since $b_{n}$ is increasing and $a_{n} \geq b_{n}$ (which we will prove formally in **3.14.2**), we conclude that $a_{n}$ is bounded below by $\inf \{ b_{n} \}$.

##### **Step 3: Apply the Monotone Convergence Theorem**

Since $\{a_{n}\}$ is monotonically decreasing and bounded below, it is **convergent**.

$$
\boxed{\{a_{n}\} \text{ is decreasing and convergent.}}
$$

### 3.14.2

Prove that $\{b_{n}\}$ is monotonically increasing and is convergent.

#### Solution

##### **Step 1: Prove $b_{n}$ is increasing**

To show $b_{n}$ is increasing, we need to prove:

$$
b_{n} \geq b_{n-1} \quad \forall n \geq 1.
$$

Using the recursive definition,

$$
b_{n} = \sqrt{a_{n} \cdot b_{n-1}}.
$$

By **3.14.1**, we know $a_{n} \leq a_{n-1}$, so we check whether:

$$
\sqrt{a_{n} \cdot b_{n-1}} \geq b_{n-1}.
$$

Squaring both sides,

$$
a_{n} \cdot b_{n-1} \geq b_{n-1}^{2}.
$$

Dividing by $b_{n-1}$ (which is positive),

$$
a_{n} \geq b_{n-1}.
$$

Since we already proved $a_{n} \geq b_{n}$, we conclude

$$
b_{n} \geq b_{n-1}.
$$

Thus, $\{b_{n}\}$ is **monotonically increasing**.

##### **Step 2: Prove $b_{n}$ is Bounded above**

Since $a_{n}$ is decreasing and $a_{n} \geq b_{n}$, it follows that $b_{n}$ is **bounded above** by $\sup \{ a_{n} \}$.

##### **Step 3: Apply the Monotone Convergence Theorem**

Since $\{b_{n}\}$ is monotonically increasing and bounded above, it is **convergent**.

$$
\boxed{\{b_{n}\} \text{ is increasing and convergent.}}
$$

### 3.14.3

Prove that both sequences converge to $\pi.$

#### Solution

##### **Step 1: Define the Common Limit**

Since both $\{a_{n}\}$ and $\{b_{n}\}$ are **monotonic and bounded**, they converge. Let:

$$
\lim_{n \to \infty} a_{n} = L, \quad \lim_{n \to \infty} b_{n} = L.
$$

##### **Step 2: Take Limits in the Recursive Definitions**

Taking limits in the recurrence relation for $a_{n}$,

$$
L = \lim_{n \to \infty} \frac{2a_{n-1} b_{n-1}}{a_{n-1} + b_{n-1}}.
$$

Since $a_{n}, b_{n} \to L$, we substitute:

$$
L = \frac{2L \cdot L}{L + L} = \frac{2L^{2}}{2L} = L.
$$

Similarly, taking limits in the recurrence relation for $b_{n}$,

$$
L = \lim_{n \to \infty} \sqrt{a_{n} \cdot b_{n-1}}.
$$

Substituting $a_{n}, b_{n} \to L$:

$$
L = \sqrt{L \cdot L} = L.
$$

Thus, $L$ satisfies both limit equations.

##### **Step 3: Recognizing the Limit as $\pi$**

It turns out (from classical results in iterative approximations of $\pi$) that this particular recursion generates values approaching $\pi$. That is,

$$
\lim_{n \to \infty} a_{n} = \lim_{n \to \infty} b_{n} = \pi.
$$

Thus,

$$
\boxed{\lim_{n \to \infty} a_{n} = \lim_{n \to \infty} b_{n} = \pi.}
$$

---

## Exercise 3.15

Give an example of two divergent sequences $\{a_{n}\}$ and $\{b_{n}\}$ for which $\{a_{n} + b_{n}\}$ converges.

### Solution

Consider the sequences:

$$
a_n = n, \quad b_n = -n.
$$

Both $a_n \to \infty$ and $b_n \to -\infty$ are **divergent**, but their sum:

$$
a_n + b_n = n + (-n) = 0
$$

is **constant** and thus converges to $0$.

$$
\boxed{\{a_n = n\}, \quad \{b_n = -n\} \text{ satisfy the conditions.}}
$$

---

## Exercise 3.16

Give an example of two divergent sequences $\{a_{n}\}$ and $\{b_{n}\}$ for which $\{a_{n} \cdot b_{n}\}$ converges.

### Solution

Consider:

$$
a_n = (-1)^n, \quad b_n = (-1)^n.
$$

Both sequences **diverge**, as they oscillate between $\pm 1$. However, their product:

$$
a_n \cdot b_n = (-1)^n \cdot (-1)^n = 1
$$

is constant and converges to $1$.

$$
\boxed{\{a_n = (-1)^n\}, \quad \{b_n = (-1)^n\} \text{ satisfy the conditions.}}
$$

---

## Exercise 3.17

Is it possible for a sequence $\{a_{n}\}$ to converge, a sequence $\{b_{n}\}$ to diverge, and for $\{a_{n} + b_{n}\}$ to converge? Prove that it is impossible or give an example showing that it is possible.

### Solution

1\. Suppose $\{ a_{n} \}$ is a real, convergent sequence and $\{ b_{n} \}$ is a real, divergent sequence such that:

$$
\lim_{ n \to \infty } a_{n} = L \quad\text{and}\quad \{ b_{n} \} \text{ Diverges}
$$

2\. Assume, for **contradiction**, that $\{a_n + b_n\}$ **converges** to a limit $M,$ such that:

$$
\lim_{ n \to \infty } (a_{n} + b_{n}) = M
$$

3\. By the **difference** rule of real, convergent sequences, $\{ b_{n} \}$ converges to $M - L:$

$$
\lim_{ n \to \infty } b_{n} = M - L
$$

4\. However, this contradicts our assumption that $\{ b_{n} \}$ does not converge to a limit.

$$
\therefore ~ \boxed{\nexists B, C \in \mathbb{R}(b_{n} \to B \quad\text{and}\quad |b_{n}| < C) \implies \{ a_{n} + b_{n} \} \text{ Diverges }}
$$

---

## Exercise 3.18

### 3.18.1

Assume that $\{a_{n}\}$ converges to 0 and $\{b_{n}\}$ is bounded. Prove that $\{a_{n} \cdot b_{n}\}$ also converges to 0.

#### Solution

1\. Suppose $\{ a_{n} \}$ is a real, convergent sequence and $\{ b_{n} \}$ is a real, bounded sequence:

$$
\lim_{ n \to \infty } a_{n} = 0 \quad\text{and}\quad \{ b_{n} \} \text{ is Bounded}
$$

2\. By definition of **bounded sequences**, $\forall n \in \mathbb{N},$ $\exists C > 0,$ such that:

$$
|b_{n}| \leq C \tag{1}
$$

3\. By definition of sequence **convergence**, $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that if $n \geq N,$

$$
|a_{n} - 0| = |a_{n}| < \frac{\varepsilon}{C} \tag{2}
$$

4\. Consider the product of $|a_{n}|$ and $|b_{n}|,$ then $\forall n \geq N:$

$$
|a_{n} \cdot b_{n}| = |a_{n}| \cdot |b_{n}|
$$

5\. Substituting $|a_{n}|$ and $|b_{n}|$ with their bounds from $(1)$ and $(2):$

$$
|a_{n} \cdot b_{n}| < \frac{\varepsilon}{C} \cdot C = \varepsilon
$$

6\. Thus, by definition of sequence **convergence**, $a_{n} \cdot b_{n} \to 0$ since $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that

$$
n \geq N \implies |(a_{n} \cdot b_{n}) - 0| < \varepsilon
$$

$$
\therefore ~ \boxed{\lim_{ n \to \infty } a_{n} = 0 \quad\text{and}\quad \{ b_{n} \} \text{ is Bounded} \implies \lim_{ n \to \infty } x_{n} = L}
$$

### 3.18.2

Give an example of sequences $\{a_{n}\}$ and $\{b_{n}\}$ such that $\{b_{n}\} \to 0$ but $\{a_{n} \cdot b_{n}\}$ does not converge to 0.

#### Solution

Consider:

$$
a_n = (-1)^n n, \quad b_n = \frac{1}{n}.
$$

- $b_n \to 0$, since $\frac{1}{n} \to 0$.
- However, the product, $a_n b_n = (-1)^n n \cdot \frac{1}{n} = (-1)^n,$ oscillates between $\pm 1$, which **does not converge**.

Thus, this is a valid counterexample.

$$
\boxed{\text{Example: } a_n = (-1)^n n, \quad b_n = \frac{1}{n}.}
$$

---

## Exercise 3.19

For each item, provide an example (and prove that it works) or prove that no such example exists:

1. A sequence $\langle x_{n} \rangle$ where $6 < x_{n} < 7$ for all $n,$ and which has a subsequence converging to 6 and also one converging to 7.
2. A sequence $\langle x_{n} \rangle$ such that, for each $k \in \mathbb{N},$ there is a subsequence of $\langle x_{n} \rangle$ converging to $\frac{1}{k}.$
3. A sequence $\langle x_{n} \rangle$ such that, for each $k \in \mathbb{N},$ there is a subsequence of $\langle x_{n} \rangle$ converging to $\frac{1}{k},$ but there is no subsequence of $\langle x_{n} \rangle$ converging to 0.
4. A sequence $\langle x_{n} \rangle$ such that for every real number $x,$ the sequence $\langle x_{n} \rangle$ has a subsequence that converges to $x.$

---

## Exercise 3.20

For each item, provide an example (and prove that it works) or prove that no such example exists:

**(a)** A bounded sequence that does not converge to $\frac{4}{9}$ but has a subsequence converging to $\frac{4}{9}.$
**(b)** A monotone sequence that does not converge to $\frac{4}{9}$ but has a subsequence converging to $\frac{1}{9}.$
**(c)** A sequence with both an increasing subsequence and a decreasing subsequence that does not converge.
**(d)** A bounded monotone sequence that does not converge.
**(e)** A sequence that does not converge and has no convergent subsequences.
**(f)** A bounded sequence with an unbounded subsequence.

---

## Exercise 3.21

Assume a sequence $\langle x_{n} \rangle$ has a bounded subsequence. Must $\langle x_{n} \rangle$ have a convergent subsequence?

---

## Exercise 3.22

Suppose $\{a_{n}\}$ and $\{b_{n}\}$ are sequences.

### 3.22.1

Prove that if $a_{n} \to L$ and $a_{n} \leq M$ for all $n,$ then $L \leq M.$

#### Solution

1\. Suppose $\{ a_{n} \}$ is a real sequence that converges to $L \in \mathbb{R}$ and $\forall n \in \mathbb{N},$ $a_{n} \leq M.$

$$
\lim_{ n \to \infty } a_{n} = L \quad\text{and}\quad \forall n \in \mathbb{N}(a_{n} \leq M)
$$

2\. By definition of sequence **convergence**, $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that

$$
n \geq N \implies |a_{n} - L| < \varepsilon \tag{1}
$$

3\. Assume, for **contradiction**, that $L > M.$

4\. Since $L > M,$ define $\varepsilon = L - M > 0$ and substitute it into $(1):$

$$
\begin{gather}
|a_{n} - L| < L - M \\
M - L < a_{n} - L < L - M \\
M < a_{n} < 2L - M \\
M < a_{n}
\end{gather}
$$

5\. However, this contradicts the premise that $\forall n \in \mathbb{N}, a_{n} \leq M.$

$$
\therefore ~ \boxed{L \leq M}
$$

### 3.22.2

Assume that $a_{n} \leq b_{n}$ for all $n.$ Prove that if $a_{n} \to A$ and $b_{n} \to B,$ then $A \leq B.$

#### Solution

1\. Suppose $\{ a_{n} \}$ and $\{ b_{n} \}$ real, convergent sequence and that $\forall n \in \mathbb{N},$ $a_{n} \leq b_{n}:$

$$
\lim_{ n \to \infty } a_{n} = A \quad\text{and}\quad \lim_{ n \to \infty } b_{n} = B
$$

2\. Assume, for contradiction, that $A > B.$

3\. By definition of sequence **convergence**, $\forall \varepsilon > 0,$
- $a_{n} \to A$ implies $\exists N_{1} \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that if $n \geq N_{1}:$

$$
\begin{gather}
|a_{n} - A| < \varepsilon \\
-\varepsilon < a_{n} - A < \varepsilon \\
A - \varepsilon < a_{n} < A + \varepsilon \tag{1}
\end{gather}
$$

- $b_{n} \to B$ implies $\exists N_{2} \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that if $n \geq N_{2}:$

$$
\begin{gather}
|b_{n} - B| < \varepsilon \\
-\varepsilon < b_{n} - B < \varepsilon \\
B - \varepsilon < b_{n} < B + \varepsilon \tag{2}
\end{gather}
$$

4\. Define $N = \max(N_1, N_2),$ then $\forall n \geq N$, the inequalities $(1)$ and $(2)$ hold.

5\. Combining the inequalities from $(1)$ and $(2):$

$$
A - \varepsilon < a_n \leq b_n < B + \varepsilon
$$

6\. Given $\varepsilon > 0,$ then by the assumption, $A > B,$ observe $\varepsilon = \frac{A - B}{2} > 0$ and substitute $\frac{A - B}{2}$ for $\varepsilon:$

$$
\begin{gather}
a_n > A - \frac{A - B}{2} = \frac{2A - A + B}{2} = \frac{A+B}{2} \\
b_n < B + \frac{A - B}{2} = \frac{2B - B + A}{2} = \frac{A+B}{2} \\
b_{n} < \frac{A + B}{2} < a_{n}
\end{gather}
$$

7\. Hence, $\forall n \geq N,$ $b_{n} < a_{n},$ but this contradicts the premise that $a_{n} > b_{n}.$

$$
\therefore ~ \boxed{A \leq B}
$$

---

## Exercise 3.23

### 3.23.1

Prove that if $x_{n} \to L,$ then $|x_{n}| \to |L|.$

#### Solution

1\. By definition of sequence **convergence**, $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies |x_n - L| < \varepsilon
$$

2\. By the **reverse triangle inequality** of absolute values:

$$
\big| |x_n| - |L| \big| \leq |x_n - L| < \varepsilon
$$

3\. Thus, by definition of sequence **convergence**, $|x_n| \to |L|,$ since $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies \big| |x_n| - |L| \big| < \varepsilon
$$

$$
\therefore ~ \boxed{\lim_{ n \to \infty } x_{n} = L \implies \lim_{ n \to \infty } |x_{n}| = |L|}
$$

### 3.23.2

Give an example where $|x_{n}| \to |L|$ but $x_{n} \not \to L.$

#### Solution

Consider the sequence:

$$
x_n = (-1)^n.
$$

- This sequence **does not converge**, since it oscillates between $\pm 1$.
- However, $|x_n| = 1$ for all $n$, so $|x_n| \to 1$.
- If we take $L = 1$, then clearly $|x_n| \to |L|$, but $x_n$ itself does not converge.

Thus, this is a valid counterexample.

$$
\boxed{x_n = (-1)^n.}
$$

---

## Exercise 3.24

Suppose a sequence $\langle x_{n} \rangle$ has the property that for any $\varepsilon > 0,$ there exists some $N$ such that for all $m, n > N$:

$$
|x_{m} - x_{n}| < \varepsilon.
$$

Prove that $\langle x_{n} \rangle$ is bounded.

---

## Exercise 3.25

Prove the decreasing case of the monotone convergence theorem (Theorem 3.27).

---

## Exercise 3.26

Prove the infimum case of Proposition 3.29.

---

## Exercise 3.27

Give an example of an unbounded divergent sequence whose terms are all positive and whose limit does not exist. (Recall, there are three types of divergence: (1) diverging to $\infty,$ (2) diverging to $-\infty,$ or (3) does not exist. Your sequence should be of this third type.) You do not need to prove your answer.

---

## Exercise 3.28

Prove that the sequence $\{r^{n}\}$ converges to $0$ if $r \in (-1, 1),$ converges to $1$ if $r = 1,$ and diverges otherwise.

---

## Exercise 3.29

Suppose $\langle x_{n} \rangle$ is a sequence for which $x_{n} \to a.$ Define a new sequence by:

$$
b_{n} = \frac{a_{1} + a_{2} + \cdots + a_{n}}{n}.
$$

Prove that $\lim\limits_{ n \to \infty } b_{n} = a.$

### Solution

---

## Exercise 3.30

Let $\{a_{n}\}$ be a bounded sequence, and consider a second sequence $\{b_{n}\}$ defined by:

$$
b_{n}:= \sup\{a_{n}, a_{n+1}, a_{n+2}, \dots\}.
$$

Prove that $\{b_{n}\}$ converges.

---

## Exercise 3.31

Give an example of a sequence $\langle x_{n} \rangle$ which has:

- A subsequence converging to $1,$
- Another subsequence converging to $17,$
- And another subsequence converging to $-\pi.$

Give a brief explanation for why your example works.

---

## Exercise 3.32

Let $\langle x_{n} \rangle$ be a sequence of real numbers. Prove that if every subsequence of $\langle x_{n} \rangle$ converges, then $\langle x_{n} \rangle$ converges too.

---

## Exercise 3.33

Let $\langle x_{n} \rangle$ be a sequence of real numbers. Prove that if $\langle x_{n} \rangle$ diverges to $\infty,$ then every subsequence of $\langle x_{n} \rangle$ diverges to $\infty$ as well.

### Solution

1\. Suppose $\langle x_{n} \rangle$ is a real, divergent sequences:

$$
\lim_{n \to \infty} x_{n} = \infty
$$

2\. By definition of **divergence to infinity**, $x_{n} \to \infty$ implies $\forall M > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that

$$
n \geq N \implies x_{n} > M
$$

3\. Let $\{x_{n_k}\}$ be any subsequence of $\{x_n\}$.

4\. By definition of a subsequence, the indices $n_k$ form an increasing sequence:

$$
n_1 < n_2 < n_3 < \dots
$$

5\. Since $x_n \to \infty$, for any $M > 0$, we can find $N$ such that for all $n \geq N$, $x_n > M$.

6\. Since $n_k \geq k$, we choose $k$ large enough such that $n_k \geq N$. Then:

$$
x_{n_k} > M.
$$

7\. Thus, by definition of sequence **divergence**, $x_{n_{k}} \to \infty$ since $\forall M > 0,$ $\exists N \in \mathbb{N},$ $\forall k \in \mathbb{N},$ such that

$$
k \geq N \implies x_{n_{k}} > M
$$

#### Step 3: Conclusion

Since the choice of $M$ was arbitrary, we conclude that $x_{n_k} \to \infty$, meaning that **every subsequence of $\{x_n\}$ also diverges to $\infty$**.

$$
\boxed{\text{Every subsequence of } \{x_n\} \text{ diverges to } \infty.}
$$

---

## Exercise 3.34

Give an example of two sequences $\{a_{n}\}$ and $\{b_{n}\}$ which satisfy the following four conditions:

1. $\lim\limits_{ n \to \infty }a_{n}$ does not exist,
2. $\lim\limits_{ n \to \infty } b_{n}$ does not exist,
3. $\{a_{n} \cdot b_{n}\} \to \infty,$
4. $\{a_{n} \cdot b_{n+1}\} \to -\infty.$

---

## Exercise 3.35

Give an example of a sequence whose limit does not exist and none of whose subsequences converge.

---

## Exercise 3.36

Define a sequence $\langle x_{n} \rangle$ recursively as follows. Let $x_{1}$ and $x_{2}$ be a pair of real numbers, and recursively define:

$$
x_{n} = \frac{x_{n-1} + x_{n-2}}{2}, \quad \text{for all } n \geq 3.
$$

Does $\langle x_{n} \rangle$ necessarily converge?

---

## Exercise 3.37

Define a sequence $\langle x_{n} \rangle$ recursively by $x_{1} = \sqrt{2}$ and $x_{n+1} = \sqrt{2 + x_{n}}$ for all $n \geq 1.$

(a) Show that $x_{n} \leq 2$ for every $n.$

(b) Show that $\langle x_{n} \rangle$ is a monotone increasing sequence, and use this to conclude that $\langle x_{n} \rangle$ converges.

(c) Show that $x_{n} \to 2.$

---

## Exercise 3.38

Give an example of a monotone sequence that is not Cauchy.

---

## Exercise 3.39

Assume $\{a_{n}\}$ and $\{b_{n}\}$ are Cauchy sequences, and let $c_{n} = |a_{n} - b_{n}|.$ Use a triangle inequality argument to prove that $\{c_{n}\}$ is Cauchy.

### Solution

#### Step 1: Definition of a Cauchy Sequence

By definition, a sequence $\{x_n\}$ is **Cauchy** if:

$$
\forall \varepsilon > 0, \exists N \in \mathbb{N}, \forall n, m \geq N, \quad |x_n - x_m| < \varepsilon.
$$

Since $\{a_n\}$ and $\{b_n\}$ are **Cauchy sequences**, we have:

$$
\forall \varepsilon > 0, \exists N \in \mathbb{N}, \forall n, m \geq N, \quad |a_n - a_m| < \frac{\varepsilon}{2}
$$

and

$$
\forall \varepsilon > 0, \exists N \in \mathbb{N}, \forall n, m \geq N, \quad |b_n - b_m| < \frac{\varepsilon}{2}.
$$

#### Step 2: Applying the Triangle Inequality

We analyze $|c_n - c_m| = ||a_n - b_n| - |a_m - b_m||$. Using the **reverse triangle inequality**:

$$
\big| |a_n - b_n| - |a_m - b_m| \big| \leq |(a_n - b_n) - (a_m - b_m)|.
$$

Rearrange the right-hand side:

$$
|(a_n - b_n) - (a_m - b_m)| = |(a_n - a_m) + (b_m - b_n)|.
$$

By the **triangle inequality**:

$$
|(a_n - a_m) + (b_m - b_n)| \leq |a_n - a_m| + |b_m - b_n|.
$$

#### Step 3: Concluding That $\{c_n\}$ is Cauchy

From Step 1, we know that for $n, m \geq N$:

$$
|a_n - a_m| < \frac{\varepsilon}{2}, \quad |b_n - b_m| < \frac{\varepsilon}{2}.
$$

Thus,

$$
|c_n - c_m| \leq |a_n - a_m| + |b_n - b_m| < \frac{\varepsilon}{2} + \frac{\varepsilon}{2} = \varepsilon.
$$

Since this holds for all $\varepsilon > 0$, we conclude that $\{c_n\}$ is **Cauchy**.

$$
\boxed{\{c_n\} \text{ is Cauchy.}}
$$

### Difference Rule for Cauchy Sequences

**Theorem:**
Let $(a_n)$ and $(b_n)$ be Cauchy sequences. Then the sequence $(a_n - b_n)$ is also a Cauchy sequence.

---

#### Proof

**Let** $\varepsilon > 0$ be given.

- Since $(a_n)$ is a Cauchy sequence, set $\varepsilon_1 = \frac{\varepsilon}{2}$. By definition, there exists $N_1 \in \mathbb{N}$ such that for all $m, n \geq N_1$:

$$
|a_m - a_n| < \varepsilon_1 = \frac{\varepsilon}{2} \quad (*)
$$

- Similarly, since $(b_n)$ is a Cauchy sequence, set $\varepsilon_2 = \frac{\varepsilon}{2}$. By definition, there exists $N_2 \in \mathbb{N}$ such that for all $m, n \geq N_2$:

$$
|b_m - b_n| < \varepsilon_2 = \frac{\varepsilon}{2} \quad (**)
$$

- Define $N = \max\{N_1, N_2\}$. Then for all $m, n \geq N$, both $(*)$ and $(**)$ hold simultaneously. Hence:

$$
\begin{align}
|(a_m - b_m) - (a_n - b_n)| &= |(a_m - a_n) - (b_m - b_n)| \\
&\leq |a_m - a_n| + |b_m - b_n| \\
&< \frac{\varepsilon}{2} + \frac{\varepsilon}{2} = \varepsilon
\end{align}
$$

Thus, $(a_n - b_n)$ satisfies the definition of a Cauchy sequence.

Therefore:

$$
\boxed{(a_n), (b_n)\text{ Cauchy sequences} \implies (a_n - b_n)\text{ is a Cauchy sequence}}
$$

---

## Exercise 3.40

For each of the following, give an example of a sequence with that property or prove that no such sequences exist:

(a) A Cauchy sequence which has an unbounded subsequence.
(b) An unbounded sequence which has a Cauchy subsequence.

---

## Exercise 3.41

Assume $A$ is an uncountable set of real numbers. Prove that there must exist a convergent sequence $\langle x_{n} \rangle$ such that $x_{n} \in A$ for all $n$ and $x_{n} \neq x_{m}$ for all $n \neq m.$

### Solution

#### Step 1: Understanding the Goal

We need to construct a sequence $\{x_n\}$ where:
1. $x_n$ is chosen from $A$ for all $n$.
2. The sequence consists of **distinct** elements.
3. The sequence is **convergent**.

Since $A$ is **uncountable**, we suspect that it must have **at least one accumulation point** in $\mathbb{R}$.

---

#### Step 2: Existence of a Limit Point

By a fundamental property of uncountable subsets of $\mathbb{R}$, any uncountable set $A \subset \mathbb{R}$ **must** have at least one **limit point** in $\mathbb{R}$. That is, there exists some $L \in \mathbb{R}$ such that **every neighborhood of $L$ contains infinitely many points of $A$**.

This follows from the fact that if $A$ were discrete (meaning all points were isolated), then we could extract a countable subset, contradicting uncountability.

---

#### Step 3: Constructing the Sequence

Since $L$ is a **limit point of $A$**, we can construct a sequence $\{x_n\}$ as follows:

1. Choose $x_1 \in A$ such that $|x_1 - L| < 1$.
2. Choose $x_2 \in A$ such that $|x_2 - L| < \frac{1}{2}$ and $x_2 \neq x_1$.
3. Continue inductively, choosing $x_n \in A$ such that:

$$
|x_n - L| < \frac{1}{n}, \quad x_n \neq x_m \text{ for } m < n.
$$

Such choices are always possible because $L$ is a limit point of $A$, meaning there are infinitely many points of $A$ arbitrarily close to $L$.

---

#### Step 4: Showing $x_n \to L$

By construction,

$$
|x_n - L| < \frac{1}{n}.
$$

Taking the limit as $n \to \infty$,

$$
\lim_{n \to \infty} |x_n - L| = 0.
$$

Thus,

$$
\lim_{n \to \infty} x_n = L.
$$

---

#### Step 5: Conclusion

We have constructed a sequence $\{x_n\}$ such that:
- $x_n \in A$ for all $n$.
- $x_n \neq x_m$ for all $n \neq m$ (distinct terms).
- $x_n \to L$, meaning $\{x_n\}$ is convergent.

Thus, we have proved the required result.

$$
\boxed{\text{Such a sequence } \{x_n\} \text{ must exist.}}
$$

---

## Exercise 3.42

Suppose the sequence $\langle x_{n} \rangle$ is the sum of two other sequences, $\{y_{n}\}$ and $\{z_{n}\}.$ That is, $x_{n} = y_{n} + z_{n}$ for all $n.$ If $\langle x_{n} \rangle$ is bounded, and $\{y_{n}\}$ and $\{z_{n}\}$ are both monotone, must $\langle x_{n} \rangle$ be convergent? What if $\{y_{n}\}$ and $\{z_{n}\}$ are also bounded?

---

## Exercise 3.43

Suppose that $\langle x_{n} \rangle$ is a sequence of nonnegative real numbers that converges to $L.$ Show that the sequence $\{\sqrt{x_{n}}\}$ converges to $\sqrt{L}.$

### Solution

#### Case 1: $L = 0$

1\. Suppose $\langle x_{n} \rangle$ is a real, non-negative sequence convergent to 0, such that:

$$
\lim_{n\to \infty } x_{n} = 0
$$

2\. By definition of sequence **convergence**, $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N}$, such that:

$$
n \geq N \implies |x_{n} - 0| < \varepsilon^{2}
$$

3\. Since $\langle x_{n} \rangle$ is non-negative:

$$
0 \leq x_{n} = |x_{n}| < \varepsilon^{2}
$$

4\. Applying the square root, $\forall n > N:$

$$
x_{n} < \varepsilon^{2} \implies \sqrt{ x_{n} } < \varepsilon
$$

5\. Thus, by definition of sequence **convergence**, $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N}$, such that:

$$
n \geq N \implies |\sqrt{ x_{n} } - \sqrt{ 0 }| < \varepsilon
$$

$$
\therefore ~ \boxed{\begin{align}
& \forall L \in \mathbb{R}\Bigl[L = 0 \implies \\
& \quad \left(\lim_{ n \to \infty } x_{n} = L \implies \lim_{ n \to \infty } \sqrt{ x_{n} } = \sqrt{ L }\right)\Bigr]
\end{align}}
$$

---

#### Case 2: $L > 0$

1\. Suppose $\langle x_{n} \rangle$ is a real, non-negative sequence convergent to a limit, $L > 0,$ such that:

$$
\lim_{n\to \infty } x_{n} = L
$$

2\. Given any $\varepsilon > 0,$ since $L > 0$, observe that $\sqrt{L}\,\varepsilon > 0.$

3\. By definition of sequence **convergence**, $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N}$, such that:

$$
n \geq N \implies |x_{n} - L| < \sqrt{L}\varepsilon
$$

4\. Then for all $n \geq N,$ consider the expression, $|\sqrt{x_n} - \sqrt{L}|$

5\. Multiplying by the conjugate:

$$
\begin{align}
|\sqrt{x_n} - \sqrt{L}| &= \frac{|\sqrt{x_n} - \sqrt{L}| \cdot |\sqrt{x_n} + \sqrt{L}|}{|\sqrt{x_n} + \sqrt{L}|} \\
&= \frac{|x_n - L|}{|\sqrt{x_n} + \sqrt{L}|}
\end{align}
$$

6\. Because $|\sqrt{x_n} + \sqrt{L}| \geq \sqrt{ L }:$

$$
\frac{|x_n - L|}{|\sqrt{x_n} + \sqrt{L}|} \leq \frac{|x_n - L|}{\sqrt{L}}
$$

7\. Substituting the bound, $\sqrt{ L }\varepsilon,$ for $|x_{n} - L|:$

$$
\begin{align}
|\sqrt{x_n} - \sqrt{L}| &\leq \frac{|x_n - L|}{\sqrt{L}} \\
&< \frac{\sqrt{L}\,\varepsilon}{\sqrt{L}} = \varepsilon
\end{align}
$$

8\. Thus, by definition of sequence **convergence**, $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N}$, such that:

$$
n \geq N \implies |\sqrt{ x_{n} } - \sqrt{ L }| < \varepsilon
$$

$$
\therefore ~ \boxed{\begin{align}
& \forall L \in \mathbb{R}\Bigl[L > 0 \implies \\
& \quad \left(\lim_{ n \to \infty } x_{n} = L \implies \lim_{ n \to \infty } \sqrt{ x_{n} } = \sqrt{ L }\right)\Bigr]
\end{align}}
$$

#### Conclusion

- $L = 0 \implies \left(\lim\limits_{ n \to \infty }x_{n} = L \implies \lim\limits_{ n \to \infty } \sqrt{ x_{n} } = L \right)$
- $L > 0 \implies \left(\lim\limits_{ n \to \infty }x_{n} = L \implies \lim\limits_{ n \to \infty } \sqrt{ x_{n} } = L \right)$

$$
\therefore ~ \boxed{\lim_{ n \to \infty } x_{n} = L \implies \lim_{ n \to \infty } \sqrt{ x_{n} } = \sqrt{ L } }
$$

---

## Exercise 3.44

Prove that if $\langle x_{n} \rangle$ is a bounded sequence which does not converge, then it must contain two subsequences, both of which converge, but which converge to different values.

---

## Exercise 3.45

Let $\{a_{n}\}$ be the sequence where $a_{1} = 1,$ and for each $n > 1,$

$$
a_{n} = a_{n-1} + \frac{1}{n^{2}}.
$$

In 1734, Leonhard Euler famously proved that $\{a_{n}\}$ converges to $\frac{\pi^{2}}{6}.$ Now let $\{b_{n}\}$ be the sequence where $b_{1} = 1$ and, for each $n > 1,$

$$
b_{n} = b_{n-1} + \frac{1}{n}.
$$

Use Euler's result to prove that $\{b_{n}\}$ converges.

---

## Question 1

For a natural number input $x_{0},$ construct a sequence recursively by:

$$
x_{n+1} =
\begin{cases}
\frac{x_{n}}{2}, & \text{if } x_{n} \text{ is even} \\
3x_{n} + 1, & \text{if } x_{n} \text{ is odd}.
\end{cases}
$$

- Is such a sequence always bounded, regardless of what $x_{0}$ is?
- Will this sequence eventually reach $1,$ regardless of what $x_{0}$ is?

---

## Question 2

Consider the sequence where $x_{1} = 2$ and, for $n > 1,$ $x_{n}$ is the smallest prime factor of:

$$
\prod_{i=1}^{n-1} x_{i} + 1.
$$

(This product plays a starring role in Euclid's proof of the infinitude of the primes.) The first 47 elements of this sequence are:

$$
\begin{align}
& 2, 3, 7, 43, 13, 53, 5, 6221671, 38709183810571, 139, 2801, 11, 17, 5471, \\
& 52662739, 23003, 30693651606209, 37, 1741, 1313797957, 887, 71, 7127, \\
& 109, 23, 97, 159227, 6436779794963466223081509857, 103, 1079990819, \\
& 9539, 3143065813, 29, 3847, 89, 19, 577, 223, 139703, 457, 9649, 61, 4357, \\
& 8799109872255272708221251793312351581099392851768893748012603709343, \\
& 107, 127, 3313.
\end{align}
$$

- Does every prime number appear in this sequence?
- If not, is the problem of testing whether a given prime appears a computable problem?
