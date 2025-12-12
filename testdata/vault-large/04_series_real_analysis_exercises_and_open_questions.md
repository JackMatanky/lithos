---
title: 04_series_real_analysis_exercises_and_open_questions
uuid: 162e4147-13a6-4602-88de-fa55a70e7784
aliases:
  - "Real Analysis: Series, Exercises and Open Questions"
  - "Series: Exercises and Open Questions"
  - "4. Series: Exercises and Open Questions"
  - series exercises and open questions
  - series_exercises_and_open_questions
  - real_analysis_series_exercises_and_open_questions
  - 04_series_real_analysis_exercises_and_open_questions
main_title: Series
subtitle: Exercises and Open Questions
author:
  - "[[cummings_jay|Jay Cummings]]"
editor:
translator:
year_published: 2019
publisher:
page_start: 139
page_end: 147
doi:
url: https://longformmath.com/analysis-home
library:
  - "[[cummings_2019_real_analysis|Real Analysis: A Long-form Mathematics Textbook]]"
cssclasses:
status: in_progress
type: book_chapter
file_class: lib_book_chapter
date_created: 2024-12-22T19:42
date_modified: 2025-10-05T17:48
tags:
---
# 4. Series: Exercises and Open Questions

> [!book_chapter] Book Chapter Details
>
> - **Author**: `dv: this.file.frontmatter.author`
> - **Chapter**: `dv: this.file.frontmatter.aliases[0]`
> - **Book**: `dv: this.file.frontmatter.library[0]`
> - **Publisher**: `dv: this.file.frontmatter.publisher`
> - **Date Published**: `dv: this.file.frontmatter.year_published`
> - **Pages**: `dv: this.file.frontmatter.page_start + " - " + this.file.frontmatter.page_end`
>
> **Completed**::

---

<!-- Insert chapter content here -->

![[Cummings_2019_Real Analysis_04_Series.pdf|Real Analysis: Series, by Jay Cummings]]

---

## Select Solutions

[Solutions to Real Analysis: A Long-Form Mathematics Textbook Chapter 4](https://linearalgebras.com/solution-analysis-chapter-4.html#ex4-4)

---

## Exercise 4.1

Determine whether each of the following converges conditionally, converges absolutely, or diverges. You do not need to prove your answers, but state which of the following tests gives the answers: the $k^{\text{th}}$-term test, the geometric series test, and the alternating series test.

Determine whether each of the following converges conditionally, converges absolutely, or diverges. state which of the following tests gives the answers: the $k^{\text{th}}$-term test, the geometric series test, and the alternating series test.

### 4.1.1

$$
\sum_{k=1}^{\infty} (-1)^{k} \frac{1}{\sqrt{k}}
$$

#### Solution

**Step 1 (Check absolute convergence)**
Consider the series of absolute values:

$$
\sum_{k=1}^{\infty} \left| (-1)^{k} \frac{1}{\sqrt{k}} \right| = \sum_{k=1}^{\infty} \frac{1}{\sqrt{k}}.
$$

This is a $p$-series with $p = \frac{1}{2} \leq 1$, which is known to diverge.

**Step 2 (Check conditional convergence using the Alternating Series Test)**
Define $a_{k} = \frac{1}{\sqrt{k}}$. We check the conditions:
1. $\lim_{k \to \infty} a_{k} = 0$, since $\frac{1}{\sqrt{k}} \to 0$.
2. The sequence $\{ a_{k} \}$ is monotonically decreasing for all sufficiently large $k$.

Since both conditions hold, the Alternating Series Test implies convergence.

**Conclusion:** The series **converges conditionally** by the Alternating Series Test.

---

### 4.1.2

$$
\sum_{k=1}^{\infty} (-1)^{k} \frac{k}{k+7}
$$

#### Solution

**Step 1 (Check the limit of the general term)**
Consider the sequence of terms:

$$
a_{k} = (-1)^{k} \frac{k}{k+7}.
$$

Compute:

$$
\lim_{k\to\infty} \frac{k}{k+7} = 1 \neq 0.
$$

Since the general term does not tend to zero, the $k^{\text{th}}$-term test implies divergence.

**Conclusion:** The series **diverges** by the $k^{\text{th}}$-term test.

---

### 4.1.3

$$
\sum_{k=1}^{\infty} \frac{1}{(\ln(4))^{k}}
$$

#### Solution

**Step 1 (Identify as a geometric series)**
This series is geometric with common ratio:

$$
r = \frac{1}{\ln(4)}.
$$

Since $\ln(4) > 1$, we have:

$$
|r| = \frac{1}{\ln(4)} < 1.
$$

By the geometric series test, the series converges absolutely.

**Conclusion:** The series **converges absolutely** by the geometric series test.

---

### 4.1.4

$$
\sum_{k=1}^{\infty} \frac{1}{k!}
$$

#### Solution

**Step 1 (Comparison to exponential series)**
We use the standard result that:

$$
\sum_{k=1}^{\infty} \frac{1}{k!}
$$

is a known absolutely convergent series (related to the Taylor series of $e^{x}$).

Alternatively, the ratio test confirms convergence:

$$
\lim_{k\to\infty} \frac{a_{k+1}}{a_{k}} = \lim_{k\to\infty} \frac{1}{(k+1)!} \cdot k! = \lim_{k\to\infty} \frac{1}{k+1} = 0.
$$

Since $0 < 1$, the ratio test confirms absolute convergence.

**Conclusion:** The series **converges absolutely** by the ratio test.

---

### 4.1.5

$$
\sum_{k=1}^{\infty} (\sqrt{k+1} - \sqrt{k})
$$

#### Solution

**Step 1 (Telescoping series analysis)**
Consider the partial sum:

$$
S_{n} = \sum_{k=1}^{n} (\sqrt{k+1} - \sqrt{k}).
$$

This simplifies to:

$$
S_{n} = \sqrt{n+1} - \sqrt{1}.
$$

**Step 2 (Evaluate the limit of partial sums)**

$$
\lim_{n\to\infty} S_{n} = \lim_{n\to\infty} (\sqrt{n+1} - 1) = \infty.
$$

Since the sequence of partial sums diverges to infinity, the series diverges.

**Conclusion:** The series **diverges** by the telescoping sum analysis.

---

### 4.1.6

$$
\sum_{k=1}^{\infty} \frac{1}{2}
$$

#### Solution

**Step 1 (Check the general term limit)**
The general term is constant:

$$
a_{k} = \frac{1}{2}.
$$

Since:

$$
\lim_{k\to\infty} a_{k} = \frac{1}{2} \neq 0,
$$

the $k^{\text{th}}$-term test implies divergence.

**Conclusion:** The series **diverges** by the $k^{\text{th}}$-term test.

---

### 4.1.7

$$
\sum_{k=1}^{\infty} (-1)^{k} \frac{1}{\sqrt[3]{k}}
$$

#### Solution

**Step 1 (Check absolute convergence)**
Consider the absolute series:

$$
\sum_{k=1}^{\infty} \frac{1}{\sqrt[3]{k}}.
$$

This is a $p$-series with $p = \frac{1}{3} \leq 1$, and thus diverges.

**Step 2 (Check conditional convergence with the Alternating Series Test)**
Define $a_{k} = \frac{1}{\sqrt[3]{k}}$, and verify:
1. $\lim_{k\to\infty} a_{k} = 0$.
2. The sequence $\{ a_{k} \}$ is decreasing.

Since both conditions hold, the Alternating Series Test confirms convergence.

**Conclusion:** The series **converges conditionally** by the Alternating Series Test.

---

### 4.1.8

$$
\sum_{k=55}^{\infty} \frac{\ln(k)}{\ln(\ln(k))}
$$

#### Solution

**Step 1 (Check the limit of terms)**
Consider the general term:

$$
a_{k} = \frac{\ln(k)}{\ln(\ln(k))}.
$$

Compute the limit:

$$
\lim_{k\to\infty} a_{k} = \infty.
$$

Since the terms do not approach zero, the $k^{\text{th}}$-term test implies divergence.

**Conclusion:** The series **diverges** by the $k^{\text{th}}$-term test.

---

### 4.1.9

$$
\sum_{k=7}^{\infty} \left( \frac{2}{k} + \frac{3}{k^{2}} \right)
$$

#### Solution

**Step 1 (Separate the series)**
Rewriting the given series:

$$
\sum_{k=7}^{\infty} \frac{2}{k} + \sum_{k=7}^{\infty} \frac{3}{k^{2}}.
$$

- The series $\sum_{k=7}^{\infty} \frac{3}{k^{2}}$ converges absolutely ($p$-series with $p = 2 > 1$).
- The series $\sum_{k=7}^{\infty} \frac{2}{k}$ is a constant multiple of the harmonic series, which diverges.

Since one component diverges, the entire series diverges.

**Conclusion:** The series **diverges** by comparison with the harmonic series.

---

## Exercise 4.2

Does the series $\sum_{k=1}^{\infty} (-1)^{k}$ converge or diverge? Justify your answer.

### Solution

---

## Exercise 4.3 (Absolute Convergence)

Prove Proposition 4.18. That is, prove that if $\sum_{k=1}^{\infty} |x_{k}|$ converges, then $\sum_{k=1}^{\infty} x_{k}$ converges too.

### Solution

1\. Suppose $\sum_{n=1}^{\infty} |x_{n}|$ is a real, convergent series.

2\. Define a new sequence $\langle b_{n} \rangle,$ such that for all $n \in \mathbb{N}$:

$$
b_{n} = x_{n} + |x_{n}|
$$

3\. Observe that the terms of $b_{n}$ are non-negative, $b_{n} \geq 0,$ for any $n \in \mathbb{N}:$

$$
b_n =
\begin{cases}
2x_{n}, & \text{if } x_n \geq 0 \\
0, & \text{if } x_n < 0
\end{cases}
$$

4\. By the **non-negativity** of absolute values:

$$
b_{n} = x_{n} + |x_{n}| \leq |x_{n}| + |x_{n}| = 2 \cdot |x_{n}|
$$

5\. By the **Comparison Test for Real Series**, since $\sum_{n=1}^{\infty} |x_{n}|$ converges and $b_{n} \leq 2|x_{n}|$, $\sum_{n = 1}^{\infty} b_{n}$ also converges.

6\. Thus, by the **difference rule of convergent series**, if both $\sum_{n = 1}^{\infty} |x_{n}|$ and $\sum_{n = 1}^{\infty} b_{n}$ converge, then $\sum_{n = 1}^{\infty} x_{n}$ also converges

$$
\begin{align}
&x_{n} = b_{n} - |x_{n}| \\
& \quad \implies \sum_{n = 1}^{\infty}  x_{n} = \sum_{n = 1}^{\infty}  b_{n} - \sum_{n = 1}^{\infty}  |x_{n}|
\end{align}
$$

$$
\sum_{n=1}^{\infty} |x_{n}| ~ \text{Converges}  \implies \sum_{n=1}^{\infty} x_{n} ~ \text{Converges Absolutely}
$$

---

## Exercise 4.4

### 4.4.1

Give an example of a series with nonnegative terms where $\sum_{k=1}^{\infty} x_{k}$ diverges, but $\sum_{k=1}^{\infty} x_{k}^{2}$ converges.

#### Solution

### 4.4.2

Prove that if $\sum_{k=1}^{\infty} x_{k}$ converges where each $x_{k} > 0$, then $\sum_{k=1}^{\infty} x_{k}^{2}$ converges.

#### Solution

### 4.4.3

(c) Show by example that part (b) is not true if we do not insist that each $x_{k} > 0$.

#### Solution

---

## Exercise 4.5

Give an example of a series where $\sum_{k=1}^{\infty} x_{k}$ converges, but $\sum_{k=1}^{\infty} x_{2k}$ diverges.

### Solution

---

## Exercise 4.6 (Telescoping Convergence)

Let $\langle x_{k} \rangle$ be a sequence which converges to 0. Prove that the series $\sum_{k=1}^{\infty} (x_{k} - x_{k+1})$ converges to $x_{1}$.

### Solution

We define the $n$-th partial sum of the series:

$$
S_n := \sum_{k=1}^{n} (x_k - x_{k+1}).
$$

Observe that this is a **telescoping sum**. That is, most terms cancel:

$$
\begin{align}
S_n &= (x_1 - x_2) + (x_2 - x_3) + (x_3 - x_4) + \cdots + (x_n - x_{n+1}) \\
&= x_1 - x_{n+1}.
\end{align}
$$

By **Shift Theorem for Real Sequences**,

$$
\lim_{k \to \infty } x_{k} = 0 \implies  \lim_{n \to \infty} x_{n+1} = 0.
$$

Hence, taking the limit of the partial sums:

$$
\lim_{n \to \infty} S_n = \lim_{n \to \infty} (x_1 - x_{n+1}) = x_1 - 0 = x_1.
$$

Therefore, the series converges and

$$
\sum_{k=1}^{\infty} (x_k - x_{k+1}) = x_1
$$

---

## Exercise 4.7

Give an example of a series $\sum_{k=1}^{\infty} x_{k}$ where:

- $\sum_{k=1}^{\infty} x_{k}$ converges
- $\sum_{k=1}^{\infty} x_{k}^{2}$ diverges
- $\sum_{k=1}^{\infty} x_{k}^{3}$ converges

---

## Exercise 4.8

Find an estimate for $\sum_{k=1}^{\infty} (-1)^{k} \frac{1}{7k}$ that is accurate to 0.01.

### Solution

---

## Exercise 4.9 (Geometric Series Test)

The geometric series test ([[sum_04_series_real_analysis_full#Geometric Series Test (Page 123, 4.9)|Proposition 4.9]]) says that a geometric series diverges if $r \geq 1$. Recall that a series can diverge to $\infty,$ to $-\infty,$ or can be "does not exist." Which form of divergence is it for a geometric series? Note that your answer may depend on $r$.

---

## Exercise 4.10 (Cauchy)

Let $r \in (-1,1)$. Show directly (without appealing to anything we proved in this chapter) that the sequence of partial sums of the geometric series $\sum_{k=0}^{\infty} r^{k}$ is Cauchy.

### Solution

---

## Exercise 4.11

(a) Find a way to write $77.77777777 \ldots$ as a geometric series, and then prove this number is rational by using the geometric series test to write this number as a fraction with integers in the numerator and denominator.

(b) Write $77.77777777 \ldots$ as a different geometric series, and use the geometric series test to write this number as a fraction with integers in the numerator and denominator. Are your two fractions the same?

(c) A number $q$ has a **repeating decimal** if the non-integer portion of its decimal expansion is repetitive. For example, $72.578578578578578 \ldots$ has a repeating decimal. Prove that if a number $q$ has a repeating decimal, then $q$ is rational.

### Solution

---

## Exercise 4.12

### 4.12.1

Prove that if $\sum_{k=1}^{\infty} a_{k}$ converges absolutely, and $\langle b_{k} \rangle$ is a subsequence of $\langle a_{k} \rangle$, then $\sum_{k=1}^{\infty} b_{k}$ also converges absolutely.

#### Solution

### 4.12.2

Give an example demonstrating that it is necessary to assume that $\sum_{k=1}^{\infty} a_{k}$ converges absolutely.

#### Solution

---

## Exercise 4.13

### 4.13.1

Prove that if $\langle k \cdot x_{k} \rangle$ converges to a nonzero real number $L$, then the series $\sum_{k=1}^{\infty} x_{k}$ diverges. Give an example to show that the converse is false.

#### Solution

### 4.13.2

Prove that if $\langle k^{2} \cdot x_{k} \rangle$ converges (to any real number), then the series $\sum_{k=1}^{\infty} x_{k}$ converges. Give an example to show that the converse is false.

#### Solution

---

## Exercise 4.14

Prove that if $x_{k} > 0$ for all $k$ and $\sum_{k=1}^{\infty} x_{k}^{2}$ converges, then $\sum_{k=1}^{\infty} \frac{x_{k}}{k}$ converges.

### Solution

---

## Exercise 4.15 ([Cauchy Condensation Test](https://en.wikipedia.org/wiki/Cauchy_condensation_test))

Prove the **Cauchy condensation test**. That is, suppose that $\langle x_{k} \rangle$ is a decreasing sequence for which $x_{k} \to 0$. Prove that $\sum_{k=1}^{\infty} x_{k}$ converges if and only if $\sum_{k=1}^{\infty} 2^{k} \cdot x_{2^{k}}$ converges.

### Solution

---

## Exercise 4.16

The converses of (i) and (ii) of the comparison test ([[sum_04_series_real_analysis_full#Comparison Test (Page 125, 4.12)|Proposition 4.12]]) are both false. Give a pair of examples demonstrating this.

---

## Exercise 4.17 (Ratio Test)

Prove the **ratio test** via the following steps. Given a series $\sum_{k=1}^{\infty} x_{k}$ with $x_{k} \neq 0$, assume that

$$
\lim_{k \to \infty} \left| \frac{x_{k+1}}{x_{k}} \right| =: r < 1.
$$

We will prove that the series converges absolutely.

### 4.17.1

Let $q$ be such that $r < q < 1$. Explain why there is some $N$ such that $n \geq N$ implies that $|x_{k+1}| \leq |x_{k}| \cdot q$.

#### Solution

##### Step 1: Define the Limit Condition

1\. We are given that:

$$
\lim_{k \to \infty} \left| \frac{x_{k+1}}{x_{k}} \right| =: r < 1.
$$

2\. Since $r < 1$, we can **choose** any $q$ such that:

$$
r < q < 1.
$$

##### Step 2: Use the Definition of a Limit

3\. By the **definition of a limit**, there exists an index $N \in \mathbb{N}$ such that for all $k \geq N$:

$$
\left| \frac{x_{k+1}}{x_{k}} \right| \leq q.
$$

##### Step 3: Establish the Recurrence Relationship

4\. Multiplying both sides by $|x_{k}|$, we obtain:

$$
|x_{k+1}| \leq q |x_{k}|.
$$

5\. Applying this **iteratively** for $k \geq N$:

$$
\begin{gather}
|x_{N+1}| \leq q |x_{N}|, \\
|x_{N+2}| \leq q |x_{N+1}| \leq q^{2} |x_{N}|, \\
|x_{N+3}| \leq q |x_{N+2}| \leq q^{3} |x_{N}|, \\
\vdots \\
|x_{N+k}| \leq q^{k} |x_{N}|.
\end{gather}
$$

### 4.17.2

(b) Explain why $\sum_{k=1}^{\infty} |x_{N}| \cdot q^{k}$ necessarily converges.

#### Solution

##### Step 1: Recognizing a Geometric Series

1\. The sequence $|x_{N+k}|$ is bounded above by:

$$
|x_{N}| \cdot q^{k}.
$$

2\. Thus, we consider the series:

$$
\sum_{k=1}^{\infty} |x_{N}| \cdot q^{k}.
$$

##### Step 2: Applying the Geometric Series Test

3\. The sum:

$$
\sum_{k=1}^{\infty} q^{k}
$$

is a **geometric series** with ratio $q$, where $0 < q < 1$.

4\. A geometric series of the form $\sum q^{k}$ **converges** whenever $|q| < 1$, with sum:

$$
\sum_{k=1}^{\infty} q^{k} = \frac{q}{1 - q}.
$$

5\. Since $|x_{N}|$ is a constant, we conclude that:

$$
\sum_{k=1}^{\infty} |x_{N}| \cdot q^{k} \quad \text{Converges}
$$

### 4.17.3

Finally, use part (b) to prove that $\sum_{k=1}^{\infty} |x_{k}|$ converges.

#### Solution

##### Step 1: Applying the Comparison Test

1\. From **Step 1**, we have:

$$
|x_{N+k}| \leq |x_{N}| \cdot q^{k}.
$$

2\. The series:

$$
\sum_{k=1}^{\infty} |x_{N}| \cdot q^{k}
$$

**converges**, as shown in **Step 2**.

3\. Since $|x_{k}|$ is bounded above by a term of a **convergent geometric series**, we apply the **comparison test**.

##### Step 2: Concluding Absolute Convergence

4\. By the **comparison test**, if $0 \leq |x_{k}| \leq C_{k}$ for all sufficiently large $k$ and $\sum C_{k}$ **converges**, then $\sum |x_{k}|$ also **converges**.

5\. Since $\sum_{k=1}^{\infty} |x_{N}| \cdot q^{k}$ **converges**, we conclude that:

$$
\sum_{k=1}^{\infty} |x_{k}|
$$

**converges absolutely**.

#### Final Summary

- **Step 1:** We established that $|x_{k}|$ satisfies the inequality $|x_{N+k}| \leq q^{k} |x_{N}|$.
- **Step 2:** We showed that the geometric series $\sum |x_{N}| \cdot q^{k}$ **converges**.
- **Step 3:** Using the **comparison test**, we concluded that $\sum |x_{k}|$ **converges absolutely**, completing the proof of the ratio test.

---

### The Ratio Test for Positive Series of Real Numbers - ([Math Online Wiki](http://mathonline.wikidot.com/the-ratio-test-for-positive-series-of-real-numbers))

Let ${\langle x_{n} \rangle}_{n = 1}^{\infty}$ be a positive sequence of real numbers, and define:

$$
\lim_{n \to \infty} \frac{x_{n+1}}{x_{n}} = \rho
$$

Then:

1. **If** $0 \leq \rho < 1$, then the series $\sum x_{n}$ **converges**.
2. **If** $1 < \rho \leq \infty$, then the series $\sum x_{n}$ **diverges**.
3. **If** $\rho = 1$, then the test is **inconclusive**.

---

#### Proof of (1): Convergence for $0 \leq \rho < 1$

1\. Suppose that $0 \leq \rho < 1$.

2\. By definition, we have:

$$
\lim_{n \to \infty} \frac{x_{n+1}}{x_{n}} = \rho.
$$

3\. **Introduce an Upper Bound for $\rho$**: Since $\rho < 1$, choose a number $r$ such that:

$$
\rho < r < 1.
$$

4\. By the **definition of a limit**, there exists an index $N \in \mathbb{N}$ such that for all $n \geq N$:

$$
\frac{x_{n+1}}{x_{n}} \leq r.
$$

5\. **Establish an Upper Bound for $x_{n}$**: Rewriting the inequality:

$$
x_{n+1} \leq r x_{n}.
$$

6\. Applying this iteratively:

$$
\begin{gather}
x_{N+1} \leq r x_{N}  \\
x_{N+2} \leq r x_{N+1} \leq r^{2} x_{N} \\
x_{N+3} \leq r x_{N+2} \leq r^{3} x_{N} \\
\vdots \\
x_{N+k} \leq r^{k} x_{N}
\end{gather}
$$

7\. **Compare with a Geometric Series**: Consider the geometric series:

$$
\sum_{k=1}^{\infty} r^{k} x_{N}.
$$

8\. Since $0 \leq r < 1$, this **geometric series converges**.

9\. Since is bounded above by a term of a convergent geometric series, by the comparison test, the subseries:

$$
\sum_{n=N+1}^{\infty} x_{n} ~ \text{Converges}
$$

10\. Since removing a finite number of terms does not affect convergence, the full series $\sum x_{n}$ **converges**.

#### Proof of (2): Divergence for $1 < \rho \leq \infty$

1\. Suppose that $1 < \rho \leq \infty$.

2\. By definition:

$$
\lim_{n \to \infty} \frac{x_{n+1}}{x_{n}} = \rho.
$$

3\. **Introduce a Lower Bound for $\rho$**: Since $\rho > 1$, choose a number $r$ such that:

$$
1 < r < \rho.
$$

4\. By the **definition of a limit**, there exists an index $N \in \mathbb{N}$ such that for all $n \geq N$:

$$
r \leq \frac{x_{n+1}}{x_{n}}.
$$

5\. **Establish a Lower Bound for $x_{n}$**: Rewriting the inequality:

$$
r x_{n} \leq x_{n+1}.
$$

6\. Applying this iteratively:

$$
\begin{gather}
r x_{N} \leq x_{N+1} \\
r x_{N+1} \leq x_{N+2} \leq r^{2} x_{N} \\
r x_{N+2} \leq x_{N+3} \leq r^{3} x_{N} \\
\vdots \\
r^{k} x_{N} \leq x_{N+k}
\end{gather}
$$

7\. **Compare with a Geometric Series**: Consider the geometric series:

$$
\sum_{k=1}^{\infty} r^{k} x_{N}.
$$

8\. Since $r > 1$, this **geometric series diverges**.

9\. Since $x_{n}$ is bounded below by a term of a divergent geometric series, by the **comparison test**, the subseries:

$$
\sum_{n=N+1}^{\infty} x_{n} ~ \text{Diverges}
$$

10\. Thus, the full series $\sum x_{n}$ **diverges**.

---

#### Proof of (3): Inconclusiveness for $\rho = 1$

1\. Suppose:

$$
\lim_{n \to \infty} \frac{x_{n+1}}{x_{n}} = 1.
$$

2\. This test does not provide information about convergence.

3\. **Provide a Convergent Counterexample**: Consider the **p-series**:

$$
\sum_{n=1}^{\infty} \frac{1}{n^{2}}.
$$

4\. Using the ratio test:

$$
\rho = \lim_{n \to \infty} \frac{x_{n+1}}{x_{n}} = \lim_{n \to \infty} \frac{n^{2}}{(n+1)^{2}} = 1.
$$

5\. This series **converges**, despite $\rho = 1$.

6\. **Provide a Divergent Counterexample**: Consider the **harmonic series**:

$$
\sum_{n=1}^{\infty} \frac{1}{n}.
$$

7\. Using the ratio test:

$$
\rho = \lim_{n \to \infty} \frac{x_{n+1}}{x_{n}} = \lim_{n \to \infty} \frac{n}{n+1} = 1.
$$

8\. This series **diverges**, despite $\rho = 1$.

9\. Since $\rho = 1$ can occur in both convergent and divergent cases, the **ratio test is inconclusive** when $\rho = 1$.

---

#### Final Summary

- If $\rho < 1$, the series **converges**.
- If $\rho > 1$, the series **diverges**.
- If $\rho = 1$, the test is **inconclusive**.

---

## Exercise 4.18 (Root Test)

Prove the **root test** via the following steps.

Given a series $\sum_{k=1}^{\infty} x_{k}$ where each $x_{k} \geq 0$, assume that the limit $\lim\limits_{ k \to \infty }(x_{k})^{1/k}$ exists. Call this limit $\rho$. Then this series converges if $\rho < 1$ and diverges if $\rho > 1$. (The test is inconclusive if $\rho = 1$.)

Prove the **root test** via the following steps. Given a series $\sum_{k=1}^{\infty} x_{k}$ where each $x_{k} \geq 0$, assume that the limit:

$$
\lim_{k \to \infty} (x_{k})^{1/k} = \rho
$$

Then:

- The series **converges** if $\rho < 1$.
- The series **diverges** if $\rho > 1$.
- The test is **inconclusive** if $\rho = 1$.

---

### 4.18.1

Suppose $\rho < 1$. Let $\varepsilon = \frac{1-\rho}{2}$ and $\rho_{1} = \rho + \varepsilon$. Prove that there is some $N$ for which $(x_{n})^{1/n} < \rho_{1}$ for all $n > N$.

#### Solution

##### Step 1: Define the Limit Condition

1\. We are given that:

$$
\lim_{k \to \infty} (x_{k})^{1/k} = \rho.
$$

2\. Since $\rho < 1$, we define:

$$
\varepsilon = \frac{1 - \rho}{2}.
$$

3\. Define $\rho_{1}$ as:

$$
\rho_{1} = \rho + \varepsilon.
$$

4\. Since $\rho + \varepsilon < 1$, we now show that $(x_{k})^{1/k} < \rho_{1}$ for sufficiently large $k$.

##### Step 2: Use the Definition of a Limit

5\. By the **definition of a limit**, for any $\varepsilon > 0$, there exists an index $N \in \mathbb{N}$ such that for all $k \geq N$:

$$
(x_{k})^{1/k} < \rho_{1}.
$$

6\. This completes the proof of part (a).

### 4.18.2

Prove that $\sum_{k=N}^{\infty} x_{k}$ converges by comparing it to a geometric series. Then conclude that $\sum_{k=1}^{\infty} x_{k}$ also converges.

#### Solution

##### Step 1: Express $x_{k}$ in Terms of $\rho_{1}$

1\. From **Step 1**, we have:

$$
(x_{k})^{1/k} < \rho_{1}.
$$

2\. Raising both sides to the power of $k$:

$$
x_{k} < \rho_{1}^{k}.
$$

##### Step 2: Compare to a Geometric Series

3\. The series:

$$
\sum_{k=N}^{\infty} \rho_{1}^{k}
$$

is a **geometric series** with ratio $\rho_{1}$, where $0 < \rho_{1} < 1$.

4\. Since a geometric series of the form $\sum \rho_{1}^{k}$ **converges** when $|\rho_{1}| < 1$, we conclude that:

$$
\sum_{k=N}^{\infty} \rho_{1}^{k}
$$

**converges**.

##### Step 3: Apply the Comparison Test

5\. Since $0 \leq x_{k} \leq \rho_{1}^{k}$ for sufficiently large $k$, by the **comparison test**, the series:

$$
\sum_{k=N}^{\infty} x_{k}
$$

**converges**.

6\. Since removing a **finite** number of terms does not affect convergence, the entire series:

$$
\sum_{k=1}^{\infty} x_{k}
$$

**converges**.

### 4.18.3

Suppose $\rho > 1$. Let $\varepsilon = \frac{\rho-1}{2}$ and $\rho_{2} = \rho - \varepsilon$. Prove that there is some $N$ for which $(x_{n})^{1/n} > \rho_{2}$ for all $n > N$.

#### Solution

##### Step 1: Define the Limit Condition

1\. We are given that:

$$
\lim_{k \to \infty} (x_{k})^{1/k} = \rho.
$$

2\. Since $\rho > 1$, we define:

$$
\varepsilon = \frac{\rho - 1}{2}.
$$

3\. Define $\rho_{2}$ as:

$$
\rho_{2} = \rho - \varepsilon.
$$

4\. Since $\rho_{2} > 1$, we now show that $(x_{k})^{1/k} > \rho_{2}$ for sufficiently large $k$.

##### Step 2: Use the Definition of a Limit

5\. By the **definition of a limit**, for any $\varepsilon > 0$, there exists an index $N \in \mathbb{N}$ such that for all $k \geq N$:

$$
(x_{k})^{1/k} > \rho_{2}.
$$

6\. This completes the proof of part (c).

### 4.18.4

Use this to argue that $\sum_{k=N}^{\infty} x_{k}$ diverges by using the **$k$th-term test** ([[sum_04_series_real_analysis_full#$k$-th Term (Divergence) Test (Page 121, 4.5)|Proposition 4.5]]). Then conclude that $\sum_{k=1}^{\infty} x_{k}$ also diverges.

#### Solution

##### Step 1: Express $x_{k}$ in Terms of $\rho_{2}$

1\. From **Step 1**, we have:

$$
(x_{k})^{1/k} > \rho_{2}.
$$

2\. Raising both sides to the power of $k$:

$$
x_{k} > \rho_{2}^{k}.
$$

##### Step 2: Compare to a Geometric Series

3\. The series:

$$
\sum_{k=N}^{\infty} \rho_{2}^{k}
$$

is a **geometric series** with ratio $\rho_{2}$, where $\rho_{2} > 1$.

4\. Since a geometric series of the form $\sum \rho_{2}^{k}$ **diverges** when $|\rho_{2}| > 1$, we conclude that:

$$
\sum_{k=N}^{\infty} \rho_{2}^{k}
$$

**diverges**.

##### Step 3: Apply the Comparison Test

5\. Since $0 \leq \rho_{2}^{k} \leq x_{k}$ for sufficiently large $k$, by the **comparison test**, the series:

$$
\sum_{k=N}^{\infty} x_{k}
$$

**diverges**.

6\. Since removing a **finite** number of terms does not affect divergence, the entire series:

$$
\sum_{k=1}^{\infty} x_{k}
$$

**diverges**.

### Final Summary

- **Step 1:** If $\rho < 1$, we showed that $\sum x_{k}$ **converges**.
- **Step 2:** If $\rho > 1$, we showed that $\sum x_{k}$ **diverges**.
- **Step 3:** If $\rho = 1$, the test is **inconclusive**.

---

### The Root Test for Positive Series of Real Numbers ([Math Online Wiki](http://mathonline.wikidot.com/the-root-test-for-positive-series-of-real-numbers))

Let $(x_k)$ be a positive sequence of real numbers, and define the series:

$$
\sum x_k = x_1 + x_2 + x_3 + \dots
$$

with the **sequence of partial sums**:

$$
S_{n} = \sum_{k=1}^{n} x_k.
$$

Suppose that the following limit exists:

$$
\lim_{k \to \infty} (x_k)^{1/k} = L.
$$

Then:

- **If** $0 \leq L < 1$, then the series $\sum x_k$ **converges**.
- **If** $1 < L \leq \infty$, then the series $\sum x_k$ **diverges**.
- **If** $L = 1$, then this test is **inconclusive**.

#### **Proof Of (a): Convergence for $0 \leq L < 1$**

##### Step 1: Define the Limit Condition

1\. We assume that:

$$
\lim_{k \to \infty} (x_k)^{1/k} = L.
$$

2\. Since $0 \leq L < 1$, we choose a number $r$ such that:

$$
L < r < 1.
$$

##### Step 2: Use the Definition of a Limit

3\. By the **definition of a limit**, there exists an index $N \in \mathbb{N}$ such that for all $k \geq N$:

$$
(x_k)^{1/k} \leq r.
$$

##### Step 3: Establish the Recurrence Relationship

4\. Raising both sides to the power of $k$:

$$
x_k \leq r^k.
$$

##### Step 4: Compare to a Geometric Series

5\. The series:

$$
\sum_{k=N}^{\infty} r^k
$$

is a **geometric series** with ratio $r$, where $0 < r < 1$.

6\. Since a geometric series of the form $\sum r^k$ **converges** when $|r| < 1$, we conclude that:

$$
\sum_{k=N}^{\infty} r^k \quad \text{Converges}
$$

##### Step 5: Apply the Comparison Test

7\. Since $0 \leq x_k \leq r^k$ for sufficiently large $k$, by the **comparison test**, the series:

$$
\sum_{k=N}^{\infty} x_k \quad \text{Converges}
$$

8\. Since removing a **finite** number of terms does not affect convergence, the entire series:

$$
\sum_{k=1}^{\infty} x_k \quad \text{Converges}
$$

#### **Proof Of (b): Divergence for $1 < L \leq \infty$**

##### Step 1: Define the Limit Condition

1\. We assume that:

$$
\lim_{k \to \infty} (x_k)^{1/k} = L.
$$

2\. Since $L > 1$, we choose a number $r$ such that:

$$
1 < r < L.
$$

##### Step 2: Use the Definition of a Limit

3\. By the **definition of a limit**, there exists an index $N \in \mathbb{N}$ such that for all $k \geq N$:

$$
r \leq (x_k)^{1/k}.
$$

##### Step 3: Establish the Recurrence Relationship

4\. Raising both sides to the power of $k$:

$$
r^k \leq x_k.
$$

##### Step 4: Compare to a Geometric Series

5\. The series:

$$
\sum_{k=N}^{\infty} r^k
$$

is a **geometric series** with ratio $r$, where $r > 1$.

6\. Since a geometric series of the form $\sum r^k$ **diverges** when $r > 1$, we conclude that:

$$
\sum_{k=N}^{\infty} r^k \quad \text{Diverges}
$$

##### Step 5: Apply the Comparison Test

7\. Since $0 \leq r^k \leq x_k$ for sufficiently large $k$, by the **comparison test**, the series:

$$
\sum_{k=N}^{\infty} x_k \quad \text{Diverges}
$$

8\. Since removing a **finite** number of terms does not affect divergence, the entire series:

$$
\sum_{k=1}^{\infty} x_k \quad \text{Diverges}
$$

#### **Final Summary**

- **Step 1:** If $L < 1$, we showed that $\sum x_k$ **converges**.
- **Step 2:** If $L > 1$, we showed that $\sum x_k$ **diverges**.
- **Step 3:** If $L = 1$, the test is **inconclusive**.

This completes the proof of the **Root Test**.

---

## Exercise 4.19

Give an example of a divergent series $\sum_{k=1}^{\infty} x_{k}$ for which $\lim\limits_{ k \to \infty }(x_{k+1} - x_{k}) = 0$.

### Solution

---

## Exercise 4.20

### 4.20.1

Give an example of two divergent series $\sum_{k=1}^{\infty} a_{k}$ and $\sum_{k=1}^{\infty} b_{k}$ such that $\sum_{k=1}^{\infty} a_{k} b_{k}$ converges.

#### Solution

### 4.20.2

Give an example of two convergent series $\sum_{k=1}^{\infty} a_{k}$ and $\sum_{k=1}^{\infty} b_{k}$ such that $\sum_{k=1}^{\infty} a_{k} b_{k}$ diverges.

#### Solution

### 4.20.3

Prove that if $\sum_{k=1}^{\infty} a_{k}$ and $\sum_{k=1}^{\infty} b_{k}$ converge absolutely, then $\sum_{k=1}^{\infty} a_{k} b_{k}$ converges absolutely.

#### Solution

---

## Exercise 4.21

Give an example of a divergent series $\sum_{k=1}^{\infty} a_{k}$ and a convergent series $\sum_{k=1}^{\infty} b_{k}$ where $a_{k} \leq b_{k}$ for all $k$.

---

## Exercise 4.22 ([Cesaro Summation](https://en.wikipedia.org/wiki/Ces%C3%A0ro_summation))

Consider the sum $\sum_{k=1}^{\infty} x_{k}$ and define $s_{n}$ to be this series' $n^{\text{th}}$ partial sum; that is, $s_{n} = \sum_{k=1}^{n} x_{k}$. The series $\sum_{k=1}^{\infty} x_{k}$ is called **Cesaro summable** if

$$
\lim_{n \to \infty} \frac{s_{1} + s_{2} + \dots + s_{n}}{n}
$$

converges.

### 4.22.1

Prove that if $\sum_{k=1}^{\infty} x_{k}$ converges, then this series is Cesaro summable.

#### Solution

### 4.22.2

Prove by example that if $\sum_{k=1}^{\infty} x_{k}$ is Cesaro summable, this does **not** imply that $\sum_{k=1}^{\infty} x_{k}$ converges.

#### Solution

---

## Exercise 4.23

Show that if $\sum_{k=1}^{\infty} x_{k}$ is conditionally convergent, then there exists a rearrangement of this sum which diverges to $\infty$.

### Solution

---

## Exercise 4.24

Show that if $\sum_{k=1}^{\infty} x_{k}$ is conditionally convergent, then there exists a rearrangement of this sum whose limit does not exist.

### Solution

---

## Exercise 4.25

Assume that $\sum_{k=1}^{\infty} x_{k}$ is conditionally convergent. Define the **limit superior** of a sequence $\{ s_{n} \}$ (Notation: "$\limsup\limits_{n \to \infty} s_{n}$") to be

$$
\limsup_{n \to \infty} s_{n} = \lim_{n \to \infty} \left( \sup_{m > n} s_{m} \right).
$$

And define the **limit inferior** of a sequence $\{ s_{n} \}$ (Notation: "$\liminf\limits_{ n \to \infty } s_{n}$") to be

$$
\liminf_{n \to \infty} s_{n} = \lim_{n \to \infty} \left( \inf_{m > n} s_{m} \right).
$$

Prove that for any $\alpha, \beta \in \mathbb{R} \cup \{\pm \infty\}$ with $\alpha \leq \beta$, there is a rearrangement of $\sum_{k=1}^{\infty} x_{k}$ whose sequence of partial sums, $\{ s_{n} \}$, have

$$
\limsup_{n \to \infty} s_{n} = \beta \quad \text{and} \quad \liminf_{n \to \infty} s_{n} = \alpha.
$$

### Solution

---

## Exercise 4.26

**Statement**: Prove that if $\sum_{k=1}^{\infty} x_{k}$ **converges absolutely** to $L$, then any **rearrangement** of this sum also converges to $L$.

### Solution

---

## Exercise 4.27

**Statement**: Prove that if each $x_{k} \geq 0$ and $\sum_{k=1}^{\infty} x_{k} = \infty$, then any **rearrangement** of this sum also diverges to $\infty$.

### Solution

---

## Exercise 4.28 ([The Partial Summation Formula for Series of Real Numbers - Mathonline](http://mathonline.wikidot.com/the-partial-summation-formula-for-series-of-real-numbers))

**Statement**: Prove the **summation by parts** formula. That is, prove that if $\langle a_{k} \rangle$ and $\langle b_{k} \rangle$ are sequences and $s_{n} = a_{1} + a_{2} + \dots + a_{n}$ then:

$$
\sum_{k=j+1}^{n} a_{k} b_{k} = s_{n} b_{n+1} - s_{j} b_{j+1} + \sum_{k=j+1}^{n} s_{k} (b_{k} - b_{k+1}).
$$

### Solution

---

## Exercise 4.29: Decomposition into Distinct Unit Fractions

**Statement:** Every positive rational number can be written as a finite sum of distinct fractions of the form $\frac{1}{n}$.

**Example**

For instance, the rational number $\frac{7243}{4140}$ can be expressed as:

$$
\frac{7243}{4140} = 1 + \frac{1}{2} + \frac{1}{5} + \frac{1}{21} + \frac{1}{527} + \frac{1}{3,054,492}.
$$

**Decomposition Process**

The following steps outline the process of finding such a decomposition for $\frac{7243}{4140}$:

1\.**Extract the integer part**:

$$
7243 \div 4140 = 1 \implies \frac{7243}{4140} = 1 + \frac{3103}{4140}.
$$

2\. **Continue extracting fractions**:

$$
\frac{3103}{4140} = \frac{1}{2} + \frac{1033}{4140}.
$$

3\. **Repeat until the fraction is fully decomposed**:

$$
\frac{1033}{4140} = \frac{1}{5} + \frac{828}{4140},
$$

$$
\frac{828}{4140} = \frac{1}{21} + \frac{11}{4140},
$$

$$
\frac{11}{4140} = \frac{1}{527} + \frac{1}{3,054,492}.
$$

This process results in a finite sum of distinct unit fractions.

---

### (a) Addressing Large Initial Values

If the starting fraction is much greater than 1 (e.g., $\frac{244,406,536}{1}$), why does this not pose an issue?
- By repeatedly subtracting the largest possible unit fraction (1, then $\frac{1}{2}$, then $\frac{1}{3}$, etc.), we eventually reach a remaining fraction small enough to be decomposed into unit fractions.

### (b) Proof That the Numerator Decreases

Show that applying this method to a fraction $\frac{p}{q}$ where $p < q$ always reduces the numerator.
- At each step, we subtract the largest possible unit fraction $\frac{1}{n}$ such that:

$$
\frac{1}{n+1} < \frac{p}{q} < \frac{1}{n}.
$$

- Since $\frac{p}{q} - \frac{1}{n}$ has a strictly smaller numerator than $\frac{p}{q}$, the process ensures a decreasing sequence of numerators.

### (c) Why This Proves the Theorem

- Since the numerator decreases at each step, the process must terminate in a finite number of steps.
- The decomposition expresses any positive rational number as a sum of distinct unit fractions.

---

## Question 1

Does the series

$$
\sum_{k=1}^{\infty} \frac{1}{k^{3} \sin^{2}(k)}
$$

converge?

---

## **Question 2**

Does the series

$$
\sum_{k=1}^{\infty} \frac{(-1)^{k} k}{p_{k}}
$$

converge, where $p_{k}$ is the $k$th prime number?

---

## Question 3

Is it true that

$$
\sum_{k=0}^{\infty} \frac{1 + 14k + 76k^{2} + 168k^{3}}{220k} \binom{2k}{k}^{7} = \frac{32}{\pi^{3}} \,?
$$

---

## Question 4

- Is $\sum_{k=0}^{\infty} \frac{(-1)^{k}}{(2k+1)^{2}}$ irrational?
- Is $\sum_{k=1}^{\infty} \frac{1}{k^{3}}$ transcendental?

---

## **Question 5**

One can show that (see note 19)

$$
\sum_{k=1}^{\infty} \left(\frac{1}{k} - \frac{1}{k+1} \right) = 1.
$$

We now reinterpret this algebraic fact geometrically. Notice that the left-hand side represents the sum of the areas of rectangles of dimensions:
- $\frac{1}{1} \times \frac{1}{2}$,
- $\frac{1}{2} \times \frac{1}{3}$,
- $\frac{1}{3} \times \frac{1}{4}$, etc.

On the right-hand side, we have the area of a $1 \times 1$ square. This suggests that it **might be possible** to tile a $1 \times 1$ square using this infinite collection of rectangles. **Prove or disprove** that such a tiling exists. (see note 20)

---

### Footnotes

**Note 19:**

A useful approach to proving the sum formula is to use a discrete version of **partial fractions**:

$$
\frac{1}{k(k+1)} = \frac{A}{k} + \frac{B}{k+1}.
$$

Solving for $A$ and $B$, we get:

$$
1 = (A+B)k + A \implies A = 1, \quad B = -1.
$$

Applying this decomposition, the sum telescopes:

$$
\sum_{k=1}^{n} \frac{1}{k(k+1)} = \sum_{k=1}^{n} \left(\frac{1}{k} - \frac{1}{k+1} \right).
$$

Taking the limit as $n \to \infty$, we obtain:

$$
\lim_{n \to \infty} \left(1 - \frac{1}{n+1} \right) = 1.
$$

**Note 20**:

It is known that this infinite collection of rectangles **can** be packed into a $1.002 \times 1.002$ square. See:
- *Two Packing Problems* by Vojtech Bálint
- *An Algorithm for Packing Squares* by Marc Paulhus.
