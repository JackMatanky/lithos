---
title: sum_04_series_real_analysis_full
uuid: 113a7ff4-4245-43da-a719-5cb6cedc73ed
aliases:
  - "Full Summary of Real Analysis: Series"
  - "full summary of real analysis: series"
  - full_summary_of_real_analysis_series
  - sum_04_series_real_analysis_full
pillar:
  - "[[knowledge_expansion|Knowledge Expansion]]"
category:
  - "[[formal_science|Formal Science]]"
branch:
  - "[[mathematics|Mathematics]]"
field:
  - "[[calculus|Calculus]]"
  - "[[real_analysis|Real Analysis]]"
subject:
topic:
subtopic:
library:
  - "[[04_series_real_analysis|Real Analysis: Series]]"
  - "[[cummings_2019_real_analysis|Real Analysis: A Long-form Mathematics Textbook]]"
about: |
url:
status: develop
type: summary
file_class: pkm_zettel
date_created: 2024-12-29T13:12
date_modified: 2025-04-24T18:17
tags:
---
# Full Summary of Real Analysis: Series

> [!Summary]
>
> - **Resource**: `dv: this.file.frontmatter.library[0]`
>
> - **Source**:: [[Cummings_2019_Real Analysis_04_Series.pdf|Real Analysis: Series, by Jay Cummings]]
>
> - **Parent**:: [[sum_04_series_real_analysis|Summary of Real Analysis: Series]]

---

## 4.1 Sequences of Partial Sums

### Questions

1\. What is an infinite series, and how is it related to the sequence of partial sums?
2\. How do we define the sequence of partial sums $\langle s_{n} \rangle$ for a series $\sum_{k=1}^\infty a_k$?
3\. What conditions determine whether a series converges or diverges?

### Key Terms

#### Definition of a Series (Page 118, 4.1)

A **series** is a formal sum of the terms of a sequence $\langle x_{k} \rangle$:

$$
\sum_{k = 1}^{\infty} x_{k}
$$

##### Terms of a Series

The numbers $x_{k}$ are the terms of the series.

##### Sequence of Partial Sums

A series is defined via its **sequence of partial sums**:

$$
s_{n} = \left( \sum_{k = 1}^{n} x_{k} \right)_{n = 1}^{\infty}
$$

where $s_{n}$ represents the sum of the first $n$ terms.

$$
\begin{align}
s_{1} &= x_{1} \\
s_{2} &= x_{1} + x_{2} \\
s_{3} &= x_{1} + x_{2} + x_{3} \\
s_{n} &= x_{1} + x_{2} + \cdots + x_{n}
\end{align}
$$

##### Convergence of a Series

- A series **converges** to $L$ if:

$$
\lim_{n \to \infty} s_{n} = L.
$$

- If the sequence of partial sums $s_{n}$ **does not converge**, the series **diverges**.
- If $s_{n} \to \infty$ or $s_{n} \to -\infty$, the series **diverges to infinity**.

#### Series Limit Laws (Page 119, 4.3)

##### Addition Rule

Given the convergent series:

$$
\sum_{k = 1}^{\infty}a_{k} = A \quad \text{and}\quad \sum_{k = 1}^{\infty} b_{k} = B
$$

$$
\sum_{k = 1}^{\infty} (a_k + b_k) = A + B
$$

###### Proof: Addition Rule ([Math Online Wiki](http://mathonline.wikidot.com/properties-of-convergent-series))

1\. Let $x_{n}$ and $y_{n}$ be sequences of partial sums:

$$
x_{n} = \sum_{k=1}^{n} a_k, \quad \text{and} \quad y_{n} = \sum_{k=1}^{n} b_k
$$

2\. By definition of convergent series, if $\sum_{k = 1}^{\infty} a_{n}$ and $\sum_{k = 1}^{\infty} b_{n}$ are convergent, $x_{n}$ and $y_{n}$ also converge:

$$
\lim_{n \to \infty} x_{n} = A \quad \text{and} \quad \lim_{n \to \infty} y_{n} = B
$$

3\. By the **sum** property of summation:

$$
\sum_{k=1}^{n} (a_k + b_k) = \sum_{k=1}^{n} a_{k} + \sum_{k=1}^{n} b_{k} = x_{n} + y_{n}
$$

4\. By the **sum law** of convergent real sequences, since $\langle x_{n} \rangle$ and $\langle y_{n} \rangle$ are convergent sequences:

$$
\lim_{n \to \infty} \sum_{k = 1}^{n} (a_{k} + b_{k}) = \lim_{n \to \infty} (x_{n} + y_{n}) = A + B
$$

$$
\therefore ~ \boxed{\sum_{k = 1}^{\infty} (a_k + b_k) = A + B}
$$

###### Proof: Addition Rule (*Real Analysis*, Page 120)

**Step 1: Define Partial Sums**

Define the sequences of partial sums:

$$
s_{n} = \sum_{k=1}^{n} a_k, \quad \text{and} \quad t_{n} = \sum_{k=1}^{n} b_k.
$$

By assumption, both series are convergent, so their partial sums satisfy:

$$
\lim_{n \to \infty} s_{n} = \alpha, \quad \text{and} \quad \lim_{n \to \infty} t_{n} = \beta.
$$

**Step 2: Apply Sequence Limit Laws**

From the **limit laws for sequences**, we know:
- Sum Rule: $\lim \limits_{ n \to \infty } (s_{n} + t_{n}) = \alpha + \beta$
- Difference Rule: $\lim \limits_{ n \to \infty } (s_{n} - t_{n}) = \alpha - \beta$
- Scalar Multiplication Rule: $\lim \limits_{ n \to \infty } (c \cdot s_{n}) = c \cdot \alpha$ for any constant $c$

**Step 3: Define the Partial Sums of the Combined Series**

Define the sequence of partial sums for $\sum (a_k + b_k)$:

$$
v_{n} = \sum_{k=1}^{n} (a_k + b_k).
$$

Since we can rearrange finite sums:

$$
\begin{align}
s_{n} + t_{n} &= (a_{1} + a_{2} + \dots + a_{n}) + (b_{1} + b_{2} + \dots + b_{n}) \\
&= (a_{1} + b_{1}) + (a_{2} + b_{2}) + \dots + (a_{n} + b_{n}) \\
&= v_{n}.
\end{align}
$$

**Step 4: Conclusion**

Now, collecting everything:

$$
\begin{align}
\sum_{k=1}^{\infty} (a_k + b_k) &= \lim_{n \to \infty} \sum_{k=1}^{n} (a_k + b_k) \tag{Definition of series} \\
&= \lim_{n \to \infty} v_{n} \tag{Definition of $v_{n}$} \\
&= \lim_{n \to \infty} (s_{n} + t_{n}) \tag{Summation of partial sums} \\
&= \alpha + \beta \tag{Limit sum rule}
\end{align}
$$

Thus, we conclude that $\sum (a_k + b_k)$ converges to $\alpha + \beta$.
$\square$

##### Difference Rule

Given the convergent series:

$$
\sum_{k = 1}^{\infty}a_{k} = A \quad \text{and}\quad \sum_{k = 1}^{\infty} b_{k} = B
$$

$$
\sum_{k = 1}^{\infty} (a_k - b_k) = A - B
$$

###### Proof: Difference Rule

1\. Let $x_{n}$ and $y_{n}$ be sequences of partial sums:

$$
x_{n} = \sum_{k=1}^{n} a_k, \quad \text{and} \quad y_{n} = \sum_{k=1}^{n} b_k
$$

2\. By definition of convergent series, if $\sum_{k = 1}^{\infty} a_{n}$ and $\sum_{k = 1}^{\infty} b_{n}$ are convergent, $\langle x_{n} \rangle$ and $\langle y_{n} \rangle$ also converge:

$$
\lim_{n \to \infty} x_{n} = A \quad \text{and} \quad \lim_{n \to \infty} y_{n} = B
$$

3\. By the **difference** property of summation:

$$
\sum_{k=1}^{n} (a_k - b_k) = \sum_{k=1}^{n} a_{k} - \sum_{k=1}^{n} b_{k} = x_{n} - y_{n}
$$

4\. By the **difference law** of convergent real sequences, since $\langle x_{n} \rangle$ and $\langle y_{n} \rangle$ are convergent sequences:

$$
\lim_{n \to \infty} \sum_{k = 1}^{n} (a_{k} - b_{k}) = \lim_{n \to \infty} (x_{n} - y_{n}) = A - B
$$

$$
\therefore ~ \boxed{\sum_{k = 1}^{\infty} (a_k - b_k) = A - B}
$$

##### Scalar Multiplication Rule

Given a real number $,c \in \mathbb{R},$ and a convergent series

$$
\sum_{k = 1}^{\infty}a_{k} = A
$$

$$
c \cdot \sum_{k = 1}^{\infty}a_{k} = c \cdot A
$$

###### Proof: Scalar Multiplication Rule

1\. Let $x_{n}$ be sequence of partial sums and $c$ be a real number:

$$
x_{n} = \sum_{k=1}^{n} a_k \quad \text{and}\quad c \in \mathbb{R}
$$

2\. By definition of convergent series, if $\sum_{k = 1}^{\infty} a_{n}$ is convergent, $\langle x_{n} \rangle$ also converges:

$$
\lim_{n \to \infty} x_{n} = A
$$

3\. By the **scalar multiplication** property of summation:

$$
\sum_{k=1}^{n} (c \cdot a_k) = c \cdot \sum_{k=1}^{n} a_{k} = c \cdot x_{n}
$$

4\. By the **scalar multiplication** of convergent real sequences, since $\langle x_{n} \rangle$ is a convergent sequence:

$$
\lim_{n \to \infty} \sum_{k = 1}^{n} (c \cdot a_{k}) = \lim_{n \to \infty} ( c \cdot x_{n}) = c \cdot A
$$

$$
\therefore ~ \boxed{\sum_{k = 1}^{\infty} (c \cdot a_k) = c \cdot A}
$$

---

## 4.2 Series Convergence Tests

### Basic Convergence Tests

1\. What is the $k$-th term test for divergence? Why does $a_k \to 0$ imply convergence of the series $\sum a_k$?
2\. How does the geometric series test work? What conditions must be met for convergence?
3\. State and explain the comparison test. How does it relate two series $\sum a_k$ and $\sum b_k$?

### Examples

1\. Provide an example of a series that converges due to the comparison test and explain why.
2\. Show that the harmonic series $\sum \frac{1}{k}$ diverges.

### Key Terms

#### $k$-th Term (Divergence) Test (Page 121, 4.5)

- Given a real sequence, $\langle x_{k} \rangle \in \mathbb{R}$, if $x_{k} \not \to 0$, $\sum_{k = 1}^{\infty} x_{k}$ diverges.
- Given a convergent series, $\sum_{k = 1}^{\infty} x_{k}$, then $\lim \limits_{ k \to \infty } x_{k} = 0$

##### Proof (*Real Analysis*)

###### Step 1: Define the Sequence of Partial Sums

Let $\langle s_{n} \rangle$ be the sequence of partial sums, defined by:

$$
s_{n} = \sum_{k=1}^{n} x_k.
$$

###### Step 2: Convergence of the Partial Sums

By the **definition of a convergent series**, if $\sum_{k=1}^{\infty} x_k$ **converges**, then the sequence of partial sums $\langle s_{n} \rangle$ is **convergent**.

###### Step 3: Apply the Cauchy Criterion

Since $\langle s_{n} \rangle$ is a convergent sequence, it satisfies the **Cauchy Criterion**, meaning that for any $\varepsilon > 0$, there exists an integer $N$ such that for all $m, n > N$:

$$
|s_{n} - s_{m}| < \varepsilon.
$$

###### Step 4: Expressing the Cauchy Condition in Terms of the Series

Without loss of generality, let us assume $n \geq m$.

Expanding the definition of $\langle s_{n} \rangle$, we get:

$$
\left| \sum_{k=1}^{n} x_k - \sum_{k=1}^{m} x_k \right| < \varepsilon.
$$

For all $m, n > N$, this simplifies to:

$$
\left| \sum_{k=m+1}^{n} x_k \right| < \varepsilon.
$$

###### Step 5: Special Case for a Single Term

Let $m = n - 1$. Then:

$$
\begin{align}
\left| x_k \right| &< \varepsilon.
\end{align}
$$

###### Step 6: Conclusion

By the definition of **convergence**, we conclude that:

$$
\lim_{k \to \infty} x_k = 0,
$$

since for every $\varepsilon > 0$, there exists an integer $N$ such that for all $k \geq N$, we have $|x_k| < \varepsilon$.

$$
\therefore ~ \boxed{\sum_{k=1}^{\infty} x_k \text{ converges} \implies \lim_{k \to \infty} x_k = 0.}
$$

---

##### Proof ([Math Online Wiki](http://mathonline.wikidot.com/sequence-of-terms-divergence-criterion-for-infinite-series))

###### Step 1: Define the Partial Sums

Let $\sum_{k=1}^{\infty} x_k$ be a convergent series, and define its sequence of partial sums:

$$
s_k = \sum_{n=1}^{k} x_{n}.
$$

The sequence $\{s_k\}$ represents the cumulative sum of the first $k$ terms of the series.

###### Step 2: Convergence of the Partial Sums

Since the series is **convergent**, by definition, the sequence of partial sums $\{ s_k \}$ converges to a real number $L$:

$$
\lim_{k \to \infty} s_k = L.
$$

This means that as we sum more and more terms, the total sum approaches a finite limit $L$.

###### Step 3: Shift Rule for Limits

A fundamental property of limits states that if $\lim \limits_{ k \to \infty }s_k = L$, then shifting the index by one does not change the limit:

$$
\lim_{k \to \infty} s_k = L \iff \lim_{k \to \infty} s_{k-1} = L.
$$

This follows from the fact that a sequence approaching $L$ remains arbitrarily close to $L$ if we shift it by a finite number of indices.

###### Step 4: Recursive Definition of Partial Sums

From the **recursive definition** of partial sums, we express $s_k$ in terms of the previous partial sum:

$$
s_k = s_{k-1} + x_k.
$$

This equation simply states that the $k$-th partial sum is obtained by adding the $k$-th term $x_k$ to the $(k-1)$-th partial sum.

###### Step 5: Expressing $x_k$ Using Differences

By rearranging the equation from Step 4, we express $x_k$ in terms of the difference between successive partial sums:

$$
x_k = s_k - s_{k-1}.
$$

This is a key step because it allows us to analyze the behavior of $x_k$ using the known properties of $s_k$.

###### Step 6: Apply the Limit Operation

Taking limits on both sides as $k \to \infty$:

$$
\lim_{k \to \infty} x_k = \lim_{k \to \infty} (s_k - s_{k-1}).
$$

###### Step 7: Apply the Difference Rule for Limits

For any two convergent sequences $\{ a_k \}$ and $\{ b_k \}$, the difference rule states that:

$$
\lim_{k \to \infty} (a_k - b_k) = \lim_{k \to \infty} a_k - \lim_{k \to \infty} b_k.
$$

Applying this to our case:

$$
\lim_{k \to \infty} x_k = \lim_{k \to \infty} s_k - \lim_{k \to \infty} s_{k-1}.
$$

###### Step 8: Substituting the Known Limits

Since we already established that:

$$
\lim_{k \to \infty} s_k = L \quad \text{and} \quad \lim_{k \to \infty} s_{k-1} = L,
$$

the equation simplifies to:

$$
\lim_{k \to \infty} x_k = L - L = 0.
$$

###### Step 9: Conclusion

Thus, we have shown that for any convergent series $\sum_{k=1}^{\infty} x_k$, its terms must satisfy:

$$
\lim_{k \to \infty} x_k = 0.
$$

$$
\therefore ~ \boxed{\sum_{k=1}^{\infty} x_k \text{ converges} \implies \lim_{k \to \infty} x_k = 0.}
$$

#### Geometric Series (Page 122, 4.6)

- A geometric series is of the form:

$$
\sum_{k=0}^{\infty} ar^{k} = a + ar^{1} + ar^2 + ar^3 + \dots
$$

where $a$ is the first term and $r$ is the common ratio.

#### Lemma: $\langle r^{n} \rangle \text{ Converges} \iff R \in (-1, 1]$ (Page 122, 4.8)

The sequence $\langle r^{n} \rangle$ converges to 0 if $r \in (-1, 1)$, converges to 1 if $r = 1$, and diverges otherwise.

##### Proof

###### Case 1: $-1 < X < 1$

1\. Suppose $-1 < x < 1$.

2\. By the definition of **absolute value**, observe that $\forall n \in \mathbb{N}$,

$$
-1 < x < 1 \implies |x^{n}| < 1.
$$

3\. Hence, by the definition of **bounded sequences**, $\langle x^{n} \rangle$ is **bounded**.

4\. Consider the sequence $\{ |x^{n}|: n \in \mathbb{N} \}$, which consists of the absolute values of the terms.

5\. Observe that for all $n \in \mathbb{N}$:

$$
|x^{n+1}| - |x^{n}| = |x|^{n} (|x| - 1).
$$

6\. For any $x \in \mathbb{R}$, such that $|x| < 1$,

$$
\begin{align}
|x| - 1 < 0 < |x|^{n} &\implies |x|^{n} (|x| - 1) < 0 \\
&\implies |x^{n+1}| - |x^{n}| < 0 \\
&\implies |x^{n+1}| < |x^{n}|
\end{align}
$$

7\. Thus, $\langle |x^{n}| \rangle$ is a monotone **non-increasing** sequence.

8\. By the **Monotone Convergence Theorem**, since $\langle |x^{n}| \rangle$ is bounded below and monotone **non-increasing** it converges to its infimum:

$$
\lim_{n \to \infty} |x^{n}| = \inf(\{ |x^{n}| : n \in \mathbb{N} \})
$$

9\. Because the **absolute value** terms are non-negative and decreasing:

$$
\inf(\{ |x^{n}| : n \in \mathbb{N} \}) = 0
$$

10\. By the **non-negativity** of absolute values:

$$
-|x^{n}| \leq x^{n} \leq |x^{n}|.
$$

11\. Since $|x^{n}| \to 0$, applying the **Squeeze Theorem**, we obtain:

$$
\lim_{n \to \infty} x^{n} = 0.
$$

$$
\therefore ~ \boxed{ -1 < x < 1 \implies \lim_{ n \to \infty } x^{n} = 0}
$$

###### Case 2: $r = 1$

1\. Suppose $r = 1$. The sequence is given by:

$$
r^{n} = 1^{n} = 1, \quad \forall n \in \mathbb{N}.
$$

2\. The sequence is **constant** at 1.

3\. By the **definition of convergence**, a sequence $x_{n}$ converges to $L$ if:

$$
\forall \varepsilon > 0, \exists N \in \mathbb{N}, \forall n \geq N, |x_{n} - L| < \varepsilon.
$$

4\. Choosing $L = 1$ and $N = 1$, we check:

$$
|r^{n} - 1| = |1 - 1| = 0 < \varepsilon.
$$

5\. Since the condition holds for all $\varepsilon > 0$, the sequence converges to 1.

$$
\therefore ~ \boxed{ r = 1 \implies \lim_{ n \to \infty } r^{n} = 1}
$$

###### Case 3: $r = -1$

1\. Suppose $r = -1$. Then the sequence is:

$$
r^{n} = (-1)^{n}.
$$

2\. The sequence alternates between $1$ and $-1$, meaning:

$$
(-1)^1 = -1, \quad (-1)^2 = 1, \quad (-1)^3 = -1, \quad (-1)^4 = 1, \dots
$$

3\. **Subsequence Divergence Argument:**

- Define two subsequences:
 - Even-indexed terms: $x_{2k} = 1$.
 - Odd-indexed terms: $x_{2k+1} = -1$.
- Since these subsequences approach different values, the sequence **does not converge** to any single limit.

$$
\therefore ~ \boxed{r = -1 \implies \langle r^{n} \rangle \text{ has no limit }}
$$

###### Case 4: $r > 1$

1\. Suppose $r > 1$.

2\. The sequence is given by:

$$
r^{n} = r \cdot r \cdot r \cdots r \quad \text{(n times)}.
$$

3\. Since $r > 1$, each term grows **exponentially**:

$$
r^{n} \to \infty \text{ as } n \to \infty.
$$

4\. A sequence is **convergent** only if it is **bounded**.

5\. Since $r^{n}$ is **unbounded**, the sequence **diverges to $+\infty$**.

$$
\therefore ~ \boxed{ r > 1 \implies r^{n} \to \infty}
$$

###### Case 5: $r < -1$

1\. Suppose $r < -1$.

2\. The sequence is given by:

$$
r^{n} = (-|r|)^{n}.
$$

3\. The magnitude of $r^{n}$ grows **exponentially**, and the sign **alternates**:

$$
(-2)^1 = -2, \quad (-2)^2 = 4, \quad (-2)^3 = -8, \quad (-2)^4 = 16, \dots
$$

4\. The sequence exhibits **both oscillation and unbounded growth**.

5\. Since the values **oscillate** while **increasing in magnitude**, $\langle r^{n} \rangle$ **has no limit**, but:

$$
|r^{n}| \to \infty.
$$

$$
\therefore ~ \boxed{r < -1 \implies \langle r^{n} \rangle \text{ has no limit }}
$$

###### Conclusion

From the previous cases:
- **Convergence occurs for** $r \in (-1,1]$.
- **Divergence occurs for** $r \notin (-1,1]$.

Thus, we conclude:

$$
\langle r^{n} \rangle \text{ converges} \iff r \in (-1,1].
$$

$$
\therefore \boxed{\langle r^{n} \rangle \text{ converges} \iff r \in (-1,1]}
$$

#### Geometric Series Test (Page 123, 4.9)

Given $x, r \in \mathbb{R}$ such that $x \neq 0, r \neq 0$, then

$$
\sum_{k = 0}^{\infty} x \cdot r^{k} = \begin{cases}
\frac{x}{1 - r}, \quad &\text{if } |r| < 1 \\
\text{diverges}, \quad &\text{if } |r| \geq 1 \end{cases}
$$

##### Proof (*Real Analysis*)

The case where $|r| > 1$ follows from **Lemma 4.8** and the **$k$-th term test**:

If $r = 1$, then the series is:

$$
x + x + x + \dots
$$

which **clearly diverges**.

If $r = -1$, then the series alternates:

$$
x - x + x - x + x - \dots
$$

which has partial sums $(x, 0, x, 0, x, 0, \dots)$, and **does not converge**.

Thus, we focus on the case $|r| < 1$.

###### Step 1: Express the Partial Sum Formula

1\. For $|r| < 1$, factor the sequence of partial sums:

$$
\begin{align}
&(1 - r)(1 + r + r^2 + r^3 + \dots + r^{n})  \\
&\qquad = 1 + r + r^2 + r^3 + \dots + r^{n} \\
&\qquad \quad - r - r^2 - r^3 - \dots - r^{n+1} \\
&\qquad = 1 + 0 + 0 + \dots + 0 - r^{n+1} \\
&\qquad = 1 - r^{n+1}.
\end{align}
$$

2\. Dividing both sides by $1 - r$, we obtain:

$$
1 + r + r^2 + r^3 + \dots + r^{n} = \frac{1 - r^{n+1}}{1 - r}.
$$

###### Step 2: Compute the Partial Sum of a Geometric Series

3\. Using the previous result, we write the partial sum of the geometric series:

$$
\begin{align}
s_{n} &= x + xr + xr^2 + xr^3 + \dots + xr^{n} \\
&= x(1 + r + r^2 + r^3 + \dots + r^{n}) \\
&= x \frac{1 - r^{n+1}}{1 - r}.
\end{align}
$$

###### Step 3: Compute the Limit as $n \to \infty$

4\. By definition, the sum of the infinite geometric series is:

$$
\begin{align}
\sum_{k=0}^{\infty} x r^{k} &= \lim_{n \to \infty} s_{n} \\
&= \lim_{n \to \infty} \frac{x(1 - r^{n+1})}{1 - r}.
\end{align}
$$

5\. Since $|r| < 1$, we know that:

$$
\lim_{n \to \infty} r^{n+1} = 0.
$$

6\. Thus,

$$
\begin{align}
\sum_{k=0}^{\infty} x r^{k} &= \frac{x(1 - 0)}{1 - r} \\
&= \frac{x}{1 - r}.
\end{align}
$$

###### Conclusion

For $|r| < 1$, the geometric series **converges** and has sum:

$$
\sum_{k=0}^{\infty} x r^{k} = \frac{x}{1 - r}.
$$

Otherwise, for $|r| \geq 1$, the series **diverges**.

##### Proof ([Math Online Wiki](http://mathonline.wikidot.com/geometric-series-of-real-numbers))

###### **Step 1: Define the Sequence of Partial Sums**

1\. Define the sequence of partial sums $\langle s_{n} \rangle$ as:

$$
\begin{align}
s_{n} &= \sum_{k = 0}^{n} x \cdot r^{k} \\
s_{n} &= x + x \cdot r + x \cdot r^2 + \dots + x \cdot r^{n}
\end{align}
$$

###### **Step 2: Apply the Telescoping Property**

2\. Multiply both sides by $(1 - r)$ and distribute across the sum:

$$
\begin{align}
(1 - r) s_{n} &= (1 - r) \sum_{k = 0}^{n} x r^{k} \\
&= \sum_{k=0}^{n} (1 - r) x r^{k} \\
&= \sum_{k=0}^{n} (x r^{k} - x r^{k + 1}) \\
\end{align}
$$

4\. By the **telescoping** property of summation, all of the series's middle terms cancel out, leaving only the first and last terms:

$$
\begin{align}
(1 - r) s_{n} &= (x - x r^{n + 1}) \\
&= x(1 - r^{n + 1})
\end{align}
$$

5\. Solving for $s_{n}$, for $r \neq 1$:

$$
\begin{align}
(1 - r) s_{n} &= x(1 - r^{n + 1}) \\
s_{n} &= \frac{x(1 - r^{n+1})}{1 - r}
\end{align}
$$

###### **Step 3: Take the Limit as $n \to \infty$**

6\. If $|r| < 1$, then $\lim \limits_{ n \to \infty } r^{n+1} = 0$, then:

$$
\begin{align}
\lim_{n \to \infty} s_{n} &= \frac{x(1 - 0)}{1 - r}  \\
&= \frac{x}{1 - r}
\end{align}
$$

7\. If $|r| \geq 1$, then $r^{n+1} \not \to 0$, so the sum **diverges**.

###### **Conclusion**

Thus, we conclude:

- **If** $|r| < 1$, **the series converges** to $\sum_{k=0}^{\infty} x r^{k} = \frac{x}{1 - r}.$
- **If** $|r| \geq 1$, **the series diverges**.

$$
\therefore \boxed{\sum_{k=0}^{\infty} x r^{k} = \frac{x}{1 - r}, \quad \text{if } |r| < 1.}
$$

#### Non-Negative Series Test (Page 124, 4.11)

Given the real series, $\sum_{k=1}^{\infty} x_k$, such that for all $k \in \mathbb{N}$, $x_{k} \geq 0$, then the series either **converges** or **diverges to $\infty$**.

##### Proof

1\. Since $k \in \mathbb{N}$, $x_{k} \geq 0$ the **sequence of partial sums** is **monotone non-decreasing**:

$$
s_n = \sum_{k=1}^{n} x_k, \quad s_{1} \leq s_{2} \leq s_3 \leq \dots
$$

2\. By the **Monotone Convergence Theorem**, a monotone non-decreasing sequence is either
- **bounded above** and convergent to the supremum.
- **unbounded** and divergent to $\infty$.

$$
\therefore \sum_{k=1}^{\infty} x_k \text{ either converges or diverges to } \infty.
$$

###### Proof for Convergence

1\. Suppose $\sum_{k=1}^{\infty} x_k$ is a real series, such that $\forall k \in \mathbb{N},$ each term satisfies $x \geq 0,$ and assume the sequence of partial sums, $\langle s_{n} \rangle,$ is **bounded**.

2\. Since $x_k \geq 0$ for all $k$, the sequence of partial sums $\langle s_{n} \rangle$ is **monotone non-decreasing**, meaning:

$$
s_n \leq s_{n+1}, \quad \forall n \in \mathbb{N}.
$$

3\. Thus, by the **Monotone Convergence Theorem**, since $\langle s_{n} \rangle$ is **bounded** and **monotone non-decreasing**, the sequence converges to its supremum.

$$
\therefore ~ \boxed{\sum_{k=1}^{\infty} x_{k} = \lim_{ n \to \infty } s_{n} = \sup (\{s_n : n \in \mathbb{N} \})}
$$

###### Proof for Divergence

1\. Suppose $\sum_{k=1}^{\infty} x_k$ is a real series, such that $\forall k \in \mathbb{N},$ each term satisfies $x \geq 0,$ and assume the sequence of partial sums, $\langle s_{n} \rangle,$ is **unbounded**.

2\. Since $x_{k} \geq 0$ for all $k$, the sequence of partial sums $\langle s_{n} \rangle$ is **monotone non-decreasing**, meaning:

$$
s_n \leq s_{n+1}, \quad \forall n \in \mathbb{N}.
$$

3\. Thus, by the **Monotone Convergence Theorem**, since $\langle s_{n} \rangle$ is **unbounded** and **monotone non-decreasing**, the sequence diverges to infinity.

$$
\therefore ~ \boxed{\sum_{k=1}^{\infty} x_{k} = \lim_{ n \to \infty } s_{n} = \infty}
$$

#### Comparison Test (Page 125, 4.12)

Assume $0 \leq a_{k} \leq b_{k}$ for all $k \in \mathbb{N}$,

- If $\sum_{k=1}^{\infty}b_{k}$ converges, then $\sum_{k=1}^{\infty}a_{k}$ converges.
- If $\sum_{k=1}^{\infty}a_{k}$ diverges, then $\sum_{k=1}^{\infty}b_{k}$ diverges.

> [!Note]
>
> The converse of these statements is **not** necessarily true.

> [!Note] Additional Notes
>
> If $0 \leq a_{k} \leq b_{k}$ for **all but finitely many terms**, the theorem still applies.
>
> **Note 4.13:** Changing **finitely many terms** of a sequence or series **does not** affect whether or not the sequence or series **converges**.

##### Proof

Let $\langle a_{n} \rangle$ be the **sequence of partial sums** of $\sum_{k=1}^{\infty} a_{k}$, and let $\langle b_{n} \rangle$ be the **sequence of partial sums** of $\sum_{k=1}^{\infty} b_{k}$.

Since $a_{k} \leq b_{k}$ for all $k$, we have:

$$
\begin{align}
& a_{n} = a_{1} + a_{2} + \cdots + a_{n}  \\
& \quad \leq b_{n} = b_{1} + b_{2} + \cdots b_{n}
\end{align}
$$

for all $n$. That is,

$$
a_{n} \leq b_{n} \quad \forall n \in \mathbb{N}
$$

---

###### Proof of (1): If $\sum_{k=1}^{\infty} b_{k}$ Converges, then $\sum_{k=1}^{\infty} a_{k}$ Converges

> [!Note]
>
> By **Proposition 4.11** ([[sum_04_series_real_analysis_full#Non-Negative Series Test (Page 124, 4.11)|Non-Negative Series Test]]), to prove that $\langle a_{n} \rangle$ **converges**, we must show that $\langle a_{n} \rangle$ is **bounded above**.

1\. Suppose $\sum_{k=1}^{\infty} a_{k}$ and $\sum_{k=1}^{\infty} b_{k}$ are real series, such that $\forall k \in \mathbb{N},$ each term satisfies $0 \leq a_{k} \leq b_{k}$ and assume $\sum_{k=1}^{\infty}b_{k}$ converges.

2\. Define $\langle a_{n} \rangle$ and $\langle b_{n} \rangle$ as the series' sequences of partial sums:

$$
\begin{align}
a_{n} &= \sum_{k=1}^{n}  a_{k} = a_{1} + a_{2} + \cdots a_{n} \\
b_{n} &= \sum_{k=1}^{n}  b_{k} = b_{1} + b_{2} + \cdots + b_{n}
\end{align}
$$

3\. Since $0 \leq a_{k} \leq b_{k},$ for all $k \in \mathbb{N},$ both sequence of partial sums, $\langle a_{n} \rangle$ and $\langle b_{n} \rangle,$ are **monotone non-decreasing**, such that $\forall n \in \mathbb{N}:$

$$
a_{n} \leq a_{n + 1}\quad \text{and} \quad b_{n} \leq b_{n + 1}
$$

4\. Since $a_{k} \leq b_{k},$ for all $k \in \mathbb{N},$ then, $\forall n \in \mathbb{N}:$

$$
\begin{align}
& a_{n} = a_{1} + a_{2} + \cdots + a_{n}  \\
& \quad \leq b_{n} = b_{1} + b_{2} + \cdots b_{n} \\
& \qquad \implies a_{n} \leq b_{n}
\end{align}
$$

5\. By definition of **convergent series**, $\sum_{k=1}^{\infty} b_{k}$ converges if and only if its sequence of partial sums, $\langle b_{n} \rangle$ converges.

6\. By definition of **convergent sequences as bounded**, $\exists C > 0,$ $\forall n \in \mathbb{N},$ such that:

$$
|b_{n}| \leq C
$$

7\. Since $a_{n} \leq b_{n}$, then $\forall n \in \mathbb{N}:$

$$
a_{n} \leq b_{n} \leq C
$$

8\. By the **Monotone Convergence Theorem** since $\langle a_{n} \rangle$ **monotone non-decreasing** and **bounded above**, $\langle a_{n} \rangle$ **converges** to its supremum.

$$
\lim_{ n \to \infty } a_{n} = \sup(\{a_{n} : n \in \mathbb{N} \})
$$

9\. Thus, by definition of **convergent series**, since $\langle a_{n} \rangle$ converges, $\sum_{k=1}^{\infty} a_{k}$ converges.

$$
\therefore ~ \boxed{\sum_{k=1}^{\infty} b_{k} ~ \text{Converges} \implies \sum_{k=1}^{\infty} a_{k} ~ \text{Converges}}
$$

###### Proof of (2): If $\sum_{k=1}^{\infty} a_{k}$ Diverges, then $\sum_{k=1}^{\infty} b_{n}$ Diverges

1\. Suppose $\sum x_k$ **diverges**. Then, by **Proposition 4.11** ([[sum_04_series_real_analysis_full#Non-Negative Series Test (Page 124, 4.11)|Non-Negative Series Test]]), $\langle a_{n} \rangle$ **diverges to $\infty$**.

2\. By the **definition of divergence to $\infty$**, for any $M > 0$, there exists an $N \in \mathbb{N},$ for all $n \in \mathbb{N},$ such that,

$$
n \geq N \implies M < a_{n}
$$

3\. Since $a_{n} \leq b_{n},$ it follows that:

$$
M < a_{n} \leq b_{n} \quad \forall n \geq N
$$

4\. Therefore, by **Definition 3.15** ([[sum_03_sequences_real_analysis_full#Divergent Sequence (Page 77, 3.15)|Divergent Sequence]]), $\langle b_{n} \rangle$ **also diverges to $\infty$**.

---

## 4.3 Harmonic Series and the Series $p$-Test

### Harmonic Series

1\. Why does the harmonic series diverge even though its terms approach zero?

### $p$-Test

1\. What is the $p$-test for convergence of a series? When does $\sum \frac{1}{k^{p}}$ converge?

### Key Terms

#### The Harmonic Series (Page 126)

- The harmonic series is:

$$
\sum_{k=1}^{\infty} \frac{1}{k} = 1 + \frac{1}{2} + \frac{1}{3} + \frac{1}{4} + \dots
$$

#### Harmonic Series Test (Page 126, 4.15)

Given a harmonic series, $\sum_{k=1}^{\infty}\left(\frac{1}{k} \right)$, the series diverges.

##### Proof (*Real Analysis*, Page 127)

1\. Suppose $\sum_{k=1}^{\infty} \frac{1}{k}$ is a real, harmonic series.

###### Step 1: Consider the Partial Sums

2\. Observe that the expanded series can be split into **partial sums**:

$$
\begin{align}
\sum_{k=1}^{\infty} \frac{1}{k} &= 1 + \frac{1}{2} + \frac{1}{3} + \frac{1}{4} + \frac{1}{5} + \frac{1}{6} + \frac{1}{7} + \frac{1}{8} + \dots \\
&= 1 + \left(\frac{1}{2}\right) + \left(\frac{1}{3} + \frac{1}{4}\right) + \left(\frac{1}{5} + \frac{1}{6} + \frac{1}{7} + \frac{1}{8}\right) + \dots
\end{align}
$$

###### Step 2: Construct a Lower Bound

2\. Each group can be **lower bounded** as follows:

$$
\begin{align}
\sum_{k=1}^{\infty} \frac{1}{k} &\geq 1 + \left(\frac{1}{2}\right) + \left(\frac{1}{4} + \frac{1}{4}\right) + \left(\frac{1}{8} + \frac{1}{8} + \frac{1}{8} + \frac{1}{8}\right) + \dots \\
&= 1 + \left(\frac{1}{2}\right) + \left(\frac{1}{2}\right) + \left(\frac{1}{2}\right) + \dots
\end{align}
$$

3\. Thus, if $s_{n}$ is the $n^{\text{th}}$ partial sum of the harmonic series, then $\langle s_{n} \rangle$ is monotonically non-decreasing and, by the above, we obtain the inequality:

$$
s_{2^n} \geq 1 + n \cdot \frac{1}{2}.
$$

###### Step 3: Show Divergence by Comparison

4\. The expression $1 + n \cdot \frac{1}{2}$ **diverges to $\infty$** as $n \to \infty$.

5\. By the **Comparison Test** (Proposition 4.12), since the **subsequence** $\langle s_{2^{n}} \rangle$ **diverges to $\infty$**, the entire sequence $\langle s_{n} \rangle$ is **unbounded**.

6\. By the ([[sum_03_sequences_real_analysis_full#Monotone Convergence Theorem (Page 90, 3.27)|Monotone Convergence Theorem]]), since $\langle s_{n} \rangle$ is **monotonically increasing** and **unbounded**, it must **diverge to $\infty$**.

$$
\therefore \sum_{k=1}^{\infty} \frac{1}{k} \text{ Diverges}.
$$

##### Proof ([ProofWiki - The Harmonic Series is Divergent](https://proofwiki.org/wiki/Harmonic_Series_is_Divergent))

###### Step 1: Define the Harmonic Series

1.1. Consider the harmonic series:

$$
\sum_{k=1}^{\infty} \frac{1}{k} = 1 + \frac{1}{2} + \frac{1}{3} + \frac{1}{4} + \frac{1}{5} + \frac{1}{6} + \frac{1}{7} + \dots
$$

1.2. Partition the series into dyadic blocks of partial sums, $\langle s_{k} \rangle,$ such that for any $k \geq 1,$ $s_{k}$ is double the cardinality of the previous block, $s_{k - 1}:$

$$
\sum_{k = 1}^{\infty} \frac{1}{k} =
\underbrace{ 1 }_{ s_{0} } + \underbrace{ \left(\frac{1}{2}\right) }_{ s_{1} } + \underbrace{ \left(\frac{1}{3} + \frac{1}{4}\right) }_{ s_{2} } + \underbrace{ \left(\frac{1}{5} + \frac{1}{6} + \frac{1}{7} + \frac{1}{8}\right) }_{ s_{3} } + \cdots
$$

1.3. Hence, each group $s_k$ consists of terms from $\frac{1}{2^k}$ to $\frac{1}{2^{k+1} - 1}$.

$$
\begin{align}
s_{k} &= \sum_{i = 2^{k}}^{2^{k + 1} - 1} \frac{1}{i} \\
&= \frac{1}{2^{k}} + \frac{1}{2^{k} + 1} + \cdots + \frac{1}{2^{k + 1} - 1}
\end{align}
$$

###### Step 2: Lower Bound for Partial Sums

2.1. By the **Ordering of Reciprocals**, each term in $s_k$ satisfies:

$$
i < 2^{k+1} \implies \frac{1}{i} > \frac{1}{2^{k+1}}
$$

2.2. By the **cardinality of closed intervals**, the number of terms in each $s_k$ is:

$$
(2^{k+1} - 1) - 2^{k} + 1 = 2^{k+1} - 2^k = 2^k
$$

2.4. Therefore, the sum of block $s_{k}$ satisfies:

$$
s_k > 2^{k} \cdot \frac{1}{2^{k+1}} = \frac{1}{2} \tag{1}
$$

###### Step 3: Establishing Divergence

3.1. Let $H_{2^{n}}$ denote the $2^{n}$-th harmonic sum of the harmonic series:

$$
H_{2^n} = \sum_{k=1}^{2^n} \frac{1}{k} > \sum_{k=1}^{2^n -1} \frac{1}{k}
$$

3.2. Since $H_{2^n}$ includes all the blocks from $s_{0}$ through $s_{n}:$

$$
H_{2^n} = \sum_{k=0}^{n} s_k
$$

3.3. Using the lower bound from $(1),$ with $s_{0} = 1$ and $s_{k} > \frac{1}{2}$ for $k \geq 1:$

$$
H_{2^{n}} > 1 + \sum_{k=1}^{n} \frac{1}{2} = 1 + \frac{n}{2}
$$

3.4 By the **$k^{\text{th}}$ Term Test for Real Series**, since $1 + \frac{n}{2}$ diverges to infinity as $n \to \infty$, the harmonic series diverges.

$$
\lim_{n \to \infty} H_{2^n} = \infty
$$

###### Step 4: Conclusion by the Comparison Test

4.1. By the **Comparison Test** (Proposition 4.12), since the **subsequence** $\langle H_{2^{n}} \rangle$ **diverges to infinity**, the sequence, $\langle H_{n} \rangle$ is **unbounded**.

4.2. By the **Monotone Convergence Theorem**, since $\langle H_{n} \rangle$ is **monotonically increasing** and **unbounded**, it must **diverge to infinity**.

$$
\therefore \sum_{k=1}^{\infty} \frac{1}{k} ~ \text{ Diverges}
$$

##### Proof: *Real-Analysis* Proof Reworked

1\. Suppose $\sum_{k=1}^{\infty} \frac{1}{k}$ is a real, harmonic series:

$$
\sum_{k=1}^{\infty} \frac{1}{k} = 1 + \frac{1}{2} + \frac{1}{3} + \frac{1}{4} + \frac{1}{5} + \frac{1}{6} + \frac{1}{7} + \frac{1}{8} + \dots
$$

2\. Starting from the second term, partition the series into dyadic blocks of partial sums, $\langle s_{j} \rangle,$ such that for any block $s_{j},$ its cardinality is double that of the previous block, $s_{j - 1}:$

$$
\sum_{k=1}^{\infty} \frac{1}{k} = 1 +
\underbrace{ \left(\frac{1}{2} \right) }_{ s_{1} } +
\underbrace{ \left(\frac{1}{3} + \frac{1}{4}\right) }_{ s_{2} } +
\underbrace{ \left(\frac{1}{5} + \frac{1}{6} + \frac{1}{7} + \frac{1}{8} \right) }_{ s_{3} } +
\cdots
$$

3\. For each block $s_{j},$ construct a lower bound, $\langle t_{j} \rangle,$ by taking the smallest term of $s_{j}$ and repeating it for the block's length of $2^{j}:$

$$
\begin{align}
\sum_{k = 1}^{\infty} \frac{1}{k} &\geq 1 +
\underbrace{ \left(\frac{1}{3} \right) }_{ t_{1} } +
\underbrace{ \left(\frac{1}{4} + \frac{1}{4}\right) }_{ t_{2} } +
\underbrace{ \left(\frac{1}{8} + \frac{1}{8} + \frac{1}{8} + \frac{1}{8} \right) }_{ t_{3} } +
\cdots \\
&= 1 +
\underbrace{ \left( \frac{1}{2} \right) }_{ t_{1} } +
\underbrace{ \left( \frac{1}{2} \right) }_{ t_{2} } +
\underbrace{ \left( \frac{1}{2} \right) }_{ t_{3} } +
\cdots
\end{align}
$$

4\. Define the $n^{\text{th}}$ partial sum of the harmonic series as:

$$
h_n = \sum_{k = 1}^{n} \frac{1}{k}
$$

5\. For each $n \in \mathbb{N}$, the following identity holds:

$$
h_{n+1} = h_n + \frac{1}{n+1}
$$

6\. Hence, the sequence $\langle h_{n} \rangle$ is **monotone non-decreasing**, since $\frac{1}{n + 1} > 0$ implies $h_{n+1} > h_{n}.$

7\. For each $n \in \mathbb{N}$, define the $2n^{th}$ partial sum such that it includes the initial term $1$ and the complete dyadic blocks $s_{1}, s_{2}, \dots, s_{n}:$

$$
h_{2^n} = \sum_{k = 1}^{2^n} \frac{1}{k} = 1 + \sum_{j = 1}^{n} s_{j}
$$

8\. Since $s_j \geq \frac{1}{2}$ for all $j \geq 1$, it follows that:

$$
h_{2^n} \geq 1 + \sum_{j = 1}^{n} \frac{1}{2} = 1 + \frac{n}{2}
$$

9\. Since $\lim\limits_{n \to \infty} \left(1 + \frac{n}{2} \right) = \infty,$ it follows that the subsequence $\langle h_{2^n} \rangle$ is **unbounded above**.

8\. Because $\langle h_n \rangle$ is a **monotonically increasing** sequence and it contains a subsequence that is unbounded above, the entire sequence $\langle h_n \rangle$ must also be **unbounded**.

9\. By the **Monotone Convergence Theorem**, a monotone increasing sequence that is unbounded diverges. Therefore:

$$
\boxed{\sum_{k = 1}^{\infty} \frac{1}{k} = \infty}
$$

##### Proof ([Math Online Wiki - The Harmonic Series](http://mathonline.wikidot.com/the-harmonic-series))

**Proof (Integral Test)**: Consider the function $f(x) = \frac{1}{x}$. The integral

$$
\int_{1}^{\infty} \frac{dx}{x} = \lim_{t \to \infty} \ln t
$$

**diverges**, so by the **Integral Test**, the harmonic series also **diverges**.

---

#### The $p$-Series Test (Page 128, 4.16)

Given the series:

$$
\sum_{k=1}^{\infty} \frac{1}{k^{p}},
$$

- The series **converges** if and only if $p > 1$.
- The series **diverges** if $p \leq 1$.

##### Proof (*Real Analysis*, 128-9)

###### Case 1: $p \leq 1$ Implies Divergence

1\. Suppose $\sum_{k=1}^{\infty} \frac{1}{k^{p}}$ is a real, series, such that $p \leq 1.$

2\. By the **ordering of reciprocals**, if $p \leq 1$, then $\forall k \in \mathbb{N}:$

$$
k > k^{p} \implies \frac{1}{k} \leq \frac{1}{k^{p}}
$$

3\. Observe that $\sum_{k=1}^{\infty} \frac{1}{k}$ is a harmonic series.

4\. By the **harmonic series and comparison test**, since the harmonic series $\sum_{k=1}^{\infty} \frac{1}{k}$ **diverges**, it follows that:

$$
\sum_{k=1}^{\infty} \left( \frac{1}{k^{p}} \right) ~ \text{Diverges}
$$

###### Case 2: $p > 1$ Implies Convergence (Non-Negativity)

> [!Definition] Dyadic Block
>
> A dyadic block is a set of consecutive integers of the form:
>
> $$
> \{ k \in \mathbb{N} : 2^j \leq k < 2^{j+1} \}
> $$
>
> for some fixed $j \in \mathbb{N} \cup \{ 0 \}$. Each dyadic block contains exactly $2^j$ terms.

1\. Suppose $\sum_{k=1}^{\infty} \frac{1}{k^{p}}$ is a real series, with $p > 1$. Then:

$$
\sum_{k=1}^{\infty} \frac{1}{k^{p}} = 1 + \frac{1}{2^{p}} + \frac{1}{3^{p}} + \frac{1}{4^{p}} + \cdots
$$

2\. Group the terms into $s_j$ blocks where each block doubles the size of the previous block:

$$
\sum_{k=1}^{\infty} \frac{1}{k^{p}} =
\underbrace{ \frac{1}{1^{p}} }_{ s_{0} } +
\underbrace{ \left( \frac{1}{2^{p}} + \frac{1}{3^{p}} \right) }_{ s_{1} } +
\underbrace{ \left( \frac{1}{4^{p}} + \frac{1}{5^{p}} + \frac{1}{6^{p}} + \frac{1}{7^{p}} \right) }_{ s_{2} } + \cdots
$$

3\. For each $s_j$ block, construct an overestimate block $t_j$ by taking the largest (i.e., first) term of $s_j$ and repeating it for the full length of the block:

$$
\begin{align}
\sum_{k=1}^{\infty} \frac{1}{k^{p}} &<
\underbrace{ \frac{1}{1^{p}} }_{ t_{0} } +
\underbrace{ \left( \frac{1}{2^{p}} + \frac{1}{2^{p}} \right) }_{ t_{1} } +
\underbrace{ \left( \frac{1}{4^{p}} + \frac{1}{4^{p}} + \frac{1}{4^{p}} + \frac{1}{4^{p}} \right) }_{ t_{2} } + \cdots \\
&= \underbrace{ 1 }_{ t_{0} } +
\underbrace{ \left( \frac{2}{2^{p}} \right) }_{ t_{1} } +
\underbrace{ \left( \frac{4}{4^{p}} \right) }_{ t_{2} } + \cdots
\end{align}
$$

4\. Formally, each block $t_j$ consists of $2^j$ copies of $\frac{1}{(2^j)^{p}}$, such that:

$$
t_j = 2^j \cdot \frac{1}{(2^j)^{p}} = 2^{j(1 - p)}
$$

5\. Therefore, the entire series is bounded above:

$$
\sum_{k=1}^{\infty} \frac{1}{k^{p}} < \sum_{j=0}^{\infty} 2^{j(1 - p)}
$$

6\. Define $\langle s_{n} \rangle$ as the series's sequence of partial sums and observe $\langle s_{n} \rangle$ is **monotone non-decreasing**, since $\frac{1}{k^{p}} \geq 0$ for all $k:$

$$
s_{n} \leq s_{n+1}, \quad \forall n \in \mathbb{N}
$$

7\. By the **Monotone Convergence Theorem**, since $\langle s_{n} \rangle$ is **bounded** and **monotone non-decreasing** it converges to a finite limit:

$$
\lim_{n \to \infty } s_{n} = L < \infty
$$

8\. Thus, by definition of **series convergence**, since $\langle s_{n} \rangle$ converges:

$$
\sum_{k=1}^{\infty} \frac{1}{k^{p}} \quad \text{Converges}
$$

$$
\therefore ~ \boxed{ \sum_{k=1}^{\infty} \frac{1}{k^{p}} \quad \text{Converges} \iff  p > 1 }
$$

###### Case 2: $p > 1$ Implies Convergence (Geometric Series and Comparison Test)

1\. Suppose $\sum_{k=1}^{\infty} \frac{1}{k^{p}}$ is a real series, with $p > 1$. Then:

$$
\sum_{k=1}^{\infty} \frac{1}{k^{p}} = 1 + \frac{1}{2^{p}} + \frac{1}{3^{p}} + \frac{1}{4^{p}} + \cdots
$$

2\. Group the terms into $s_j$ blocks where each block doubles the size of the previous block:

$$
\sum_{k=1}^{\infty} \frac{1}{k^{p}} =
\underbrace{ \frac{1}{1^{p}} }_{ s_{0} } +
\underbrace{ \left( \frac{1}{2^{p}} + \frac{1}{3^{p}} \right) }_{ s_{1} } +
\underbrace{ \left( \frac{1}{4^{p}} + \frac{1}{5^{p}} + \frac{1}{6^{p}} + \frac{1}{7^{p}} \right) }_{ s_{2} } + \cdots
$$

3\. For each $s_j$ block, construct an overestimate block $t_j$ by taking the largest (i.e., first) term of $s_j$ and repeating it for the full length of the block:

$$
\begin{align}
\sum_{k=1}^{\infty} \frac{1}{k^{p}} &<
\underbrace{ \frac{1}{1^{p}} }_{ t_{0} } +
\underbrace{ \left( \frac{1}{2^{p}} + \frac{1}{2^{p}} \right) }_{ t_{1} } +
\underbrace{ \left( \frac{1}{4^{p}} + \frac{1}{4^{p}} + \frac{1}{4^{p}} + \frac{1}{4^{p}} \right) }_{ t_{2} } + \cdots \\
&= \underbrace{ 1 }_{ t_{0} } +
\underbrace{ \left( \frac{2}{2^{p}} \right) }_{ t_{1} } +
\underbrace{ \left( \frac{4}{4^{p}} \right) }_{ t_{2} } + \cdots
\end{align}
$$

4\. Formally, each block $t_j$ consists of $2^j$ copies of $\frac{1}{(2^j)^{p}}$, such that:

$$
t_j = 2^j \cdot \frac{1}{(2^j)^{p}} = 2^{j(1 - p)}
$$

5\. Therefore, the entire series is bounded above:

$$
\sum_{k=1}^{\infty} \frac{1}{k^{p}} < \sum_{j=0}^{\infty} 2^{j(1 - p)}
$$

6\. Define $r = 2^{1 - p}$. Since $p > 1$, we have $1 - p < 0$, so:

$$
r = 2^{1 - p} = \frac{1}{2^{p - 1}} < 1
$$

7\. By the **geometric series test**, since $|r| \leq 1,$ the series $\sum_{j=0}^{\infty} 2^{j(1 - p)}$ converges:

$$
\sum_{j=0}^{\infty} 2^{j(1 - p)} = \sum_{j=0}^{\infty} r^j = \frac{1}{1 - r}
$$

8\. Thus, by the **comparison test**, $\sum_{k=1}^{\infty} \frac{1}{k^{p}}$ converges.

$$
\boxed{ \sum_{k=1}^{\infty} \frac{1}{k^{p}} \quad  \text{Converges} \iff p > 1 }
$$

$$
\therefore ~ \sum_{k=1}^{\infty} \left( \frac{1}{k^{p}} \right) =
\begin{cases}
\text{Converges}, & p > 1, \\
\text{Diverges}, & p \leq 1.
\end{cases}
$$

##### Proof ([Math Online Wiki](http://mathonline.wikidot.com/the-p-series-test))

##### Basel Problem

$$
\sum_{k=1}^{\infty} \left( \frac{1}{k^{2}} \right) = \frac{\pi^{2}}{6}
$$

See: [3Blue1Brown - Basel Problem](https://www.youtube.com/watch?v=d-o3eB9sfls)

See: [Basel Problem - Wikipedia](https://en.wikipedia.org/wiki/Basel_problem)

---

## 4.4 Absolute Convergence

### Definition

1\. What does it mean for a series to converge absolutely? How does this differ from conditional convergence?

### Absolute and Conditional Convergence

#### Examples

1\. Give an example of a series that converges absolutely and explain why.
2\. Give an example of a series that converges conditionally and explain why.

### Alternating Series Test

1\. What conditions must a series satisfy to use the alternating series test?
2\. Why is monotonicity of the sequence $\{a_k\}$ necessary in the alternating series test?
- Monotonicity is required in the **Alternating Series Test** to prevent situations where the series appears to have terms decreasing to zero but still **diverges**.
- The key issue is that if a sequence is alternating and converges to zero but is **not monotonically decreasing**, then:
	- The **positive terms** may diverge to $\infty$ while the negative terms converge, leading to overall divergence.
	- The **negative terms** may diverge to $-\infty$ while the positive terms converge, also leading to divergence.
- Consider the alternating series: $\sum_{k=1}^{\infty} (-1)^{k+1} \left(\frac{1}{k} - \frac{1}{k^2} \right)$, here:
	- The **positive terms** $\frac{1}{k}$ form a **divergent** harmonic series.
	- The **negative terms** $\frac{1}{k^2}$ form a **convergent** p-series.
- Since the negative terms are **not large enough** to counteract the divergence of the positive terms, the overall sum **diverges**.
- **Monotonicity ensures that the negative terms are large enough to "slow down" the divergence of the positive terms**, preventing cases where one side dominates and leads to divergence. This condition is necessary to guarantee convergence in the **Alternating Series Test**.

### Key Terms

#### Alternating Series Test (Page 130, 4.17)

Given a real, monotonic non-increasing sequence $\langle x_{k} \rangle_{k=1}^{\infty} \subseteq \mathbb{R}$ of non-negative terms, with $x_{k} \geq 0$ for all $k \in \mathbb{N}$, and such that $\lim\limits_{n \to \infty} x_{k} = 0$, then

$$
\sum_{k=1}^{\infty} (-1)^{k + 1} x_{k} ~ \text{Converges}
$$

##### Proof

1\. Suppose $\langle x_{k} \rangle$ is a real, monotonic non-increasing sequence convergent to 0:

$$
\lim_{k \to \infty } x_{k} = 0
$$

2\. Define the sequence of partial sums $\langle s_{n} \rangle$ for an alternating series:

$$
s_{n} = \sum_{k=1}^{n} (-1)^{k+1} x_{k}
$$

3\. Consider the **subsequence** $\langle s_{2n} \rangle$ formed by grouping alternating terms:

$$
s_{2n} = (x_{1} - x_{2}) + (x_{3} - x_{4}) + \dots + (x_{2n-1} - x_{2n}).
$$

4\. Since $\langle x_{k} \rangle$ is **monotonically non-increasing**, for all $k \in \mathbb{N}:$

$$
x_{2k - 1} \geq x_{2k} \implies x_{2k - 1} - x_{2k} \geq 0
$$

5\. Hence, each term in the sum is non-negative, which implies that $\langle s_{2n} \rangle$ is **monotonically non-decreasing**.

6\. Next, we show that $\langle s_{2n} \rangle$ is **bounded above**. By regrouping the terms:

$$
x_{1} \geq x_{1} - (x_{2} - x_{3}) - (x_{3} - x_{4}) - \dots - (x_{2n-2} - x_{2n-1}) - x_{2n}.
$$

7\. Since all terms on the right-hand side are non-negative, for all $n \in \mathbb{N}:$

$$
s_{2n} \leq x_{1}
$$

8\. Thus, by the **Monotonic Convergence Theorem**, since $\langle s_{2n} \rangle$ is **monotonically non-decreasing** and **bounded above** by $x_{1},$ $\langle s_{2n} \rangle$ converges to some limit $L \in \mathbb{R}:$

$$
\lim_{n \to \infty} s_{2n} = L
$$

10\. Now, consider the odd-indexed **subsequence** $\langle s_{2n+1} \rangle:$

$$
s_{2n+1} = s_{2n} + x_{2n+1}.
$$

11\. Since $x_{2n+1}$ is **positive and decreasing**, adding $x_{2n+1}$ to $s_{2n}$ results in a **monotonically decreasing** sequence:

$$
s_{2n+1} \geq s_{2n+3}, \quad \forall n.
$$

12\. Furthermore, since $\langle s_{2n+1} \rangle$ is always **greater than $\langle s_{2n} \rangle$ but decreasing**, it is **bounded below** by:

$$
s_{2n+1} \geq x_{1} - x_{2}.
$$

13\. Since $\langle s_{2n+1} \rangle$ is **monotonically decreasing** and **bounded below**, it **converges** to some limit $U$.

14\. Because:

$$
L - U = \lim_{n \to \infty} (s_{2n} - s_{2n+1}) = \lim_{n \to \infty} x_{2n+1} = 0,
$$

15\. we conclude that:

$$
L = U.
$$

16\. Since both subsequences $\langle s_{2n} \rangle$ and $\langle s_{2n+1} \rangle$ **converge to the same limit $L$**, it follows that $\langle s_{n} \rangle$ **converges to $L$** as well.

17\. Therefore, the **sequence of partial sums** $\langle s_{n} \rangle$ **converges**, which implies that the alternating series **converges**.

$$
\therefore ~ \boxed{ \sum_{k=1}^{\infty} (-1)^{k+1} x_k \text{ Converges} }
$$

##### Proof ([Math Online Wiki](http://mathonline.wikidot.com/the-alternating-series-test-for-alternating-series-of-real-n))

1\. Suppose $\langle x_n \rangle_{n=1}^{\infty}$ is a real, monotonically non-increasing sequence convergent to $0:$

$$
\lim_{n \to \infty } x_{n} = 0
$$

2\. By the **$k^{\text{th}}$ term series convergence test**, since $x_{n}$ is non-increasing and $x_{n} \to 0:$

$$
x_{n} \geq 0, \quad  \forall n \in \mathbb{N}
$$

3\. Define the corresponding sequence of partial sums, $s_{n}:$

$$
\begin{align}
s_{n} &= \sum_{k = 1}^{n} (-1)^{k+1} x_{k} \\
&= x_{1} - x_{2} + x_{3} - x_{4} + \cdots + (-1)^{n + 1} x_{n}
\end{align}
$$

4\. Let $s_{2n}$ and $s_{2n - 1}$ denote the evenand odd-indexed subsequences of $s_{n},$ respectively:

$$
\begin{align*}
s_{2n - 1} &= x_{1} - x_{2} + \cdots - x_{2n-2} + x_{2n - 1} \\
s_{2n} &= x_{1} - x_{2} + \cdots + x_{2n-1} - x_{2n}
\end{align*}
$$

5\. Rewriting the subsequences in relation to differently indexed subsequences:

$$
s_{2n + 1} = s_{2n - 1} - x_{2n} + x_{2n + 1} \tag{1}
$$

6\. By the **sign-reversal property of inequalities**, since $\langle x_{n} \rangle$ is monotonically non-increasing:

$$
x_{2n} \geq x_{2n + 1} \implies -x_{2n} + x_{2n + 1} \leq 0 \tag{2}
$$

7\. Hence, by $(1),$ $(2),$ and the non-negativity of $x_{n},$ the odd-indexed subsequence, $\langle s_{2n - 1} \rangle,$ is monotonically **non-increasing** and **bounded below** by $s_{1}:$

$$
s_{2n + 1} < s_{2n - 1} < \dots < s_{3} < s_{1}
$$

8\. Hence, by the **Monotone Convergence Theorem**, since $\langle s_{2n - 1} \rangle$ is monotonically **non-increasing** and **bounded below**, $\langle s_{2n - 1} \rangle$ converges.

9\. Since $x_{2n} = s_{2n} - s_{2n - 1}$ and $\lim\limits_{n \to \infty} x_{n} = 0,$ by the **sum rule of convergent sequences**:

$$
\begin{align}
& \lim_{n \to \infty } s_{2n} = \lim_{n \to \infty } s_{2n - 1} + 0 \\
& \quad  \implies \lim_{n \to \infty } s_{2n} = \lim_{n \to \infty } s_{2n - 1}
\end{align}
$$

10\. By the **equality of sequence and subsequence limits**, since $s_{2n}$ and $s_{2n - 1}$ converge to the same limit, $s_{n}$ is also convergent.

11\. Thus, by definition of **series convergence**, since the sequence of partial sums, $s_{n},$ converges, the series also converges.

$$
\therefore ~\boxed{\sum_{n=1}^\infty (-1)^{n+1} x_{n} ~ \text{Converges}}
$$

##### Proof ([ProofWiki](https://proofwiki.org/wiki/Alternating_Series_Test))

###### Lemma ([ProofWiki](https://proofwiki.org/wiki/Alternating_Series_Test/Lemma))

#### Absolute Convergence (Page 132, 4.18)

Given the series $\sum_{k=1}^{\infty}x_{k}$, if $\sum_{k=1}^{\infty}|x_{k}|$ converges, then $\sum_{k=1}^{\infty}x_{k}$ converges absolutely.

$$
\sum_{k=1}^{\infty}|x_{k}| ~ \text{Converges} \implies \sum_{k=1}^{\infty}x_{k} ~ \text{Converges Absolutely}
$$

##### Proof ([Absolute and Conditional Convergence - Mathonline Wiki](http://mathonline.wikidot.com/absolute-and-conditional-convergence))

1\. Suppose that $\sum_{n=1}^{\infty} |a_n|$ **converges**.

2\. Define the sequence $\langle b_{n} \rangle$ such that for all $n \in \mathbb{N}$:

$$
b_n = a_n + |a_n|
$$

3\. Since $-|a_n| \leq a_n \leq |a_n|$, adding $|a_n|$ to both sides gives:

$$
0 \leq a_n + |a_n| \leq 2 |a_n|
$$

4\. This provides the bound:

$$
0 \leq b_n \leq 2 |a_n|
$$

5\. By the **Comparison Test**, since $\sum_{n=1}^{\infty} |a_n|$ **converges**, it follows that the series $\sum_{n=1}^{\infty} b_n$ **also converges**.

6\. Observing that:

$$
\sum_{n=1}^{\infty} a_n = \sum_{n=1}^{\infty} b_n - \sum_{n=1}^{\infty} |a_n|
$$

and since both $\sum_{n=1}^{\infty} b_n$ and $\sum_{n=1}^{\infty} |a_n|$ **converge**, it follows that $\sum_{n=1}^{\infty} a_n$ **must also converge**.

$$
\sum_{k=1}^{\infty}|a_{k}| ~ \text{Converges} \implies \sum_{k=1}^{\infty}a_{k} ~ \text{Converges Absolutely}
$$

#### Absolute and Condition Convergence (Page 132, 4.18 and 4.20)

Consider the series $\sum_{k=1}^{\infty}x_{k}$,

- If $\sum_{k=1}^{\infty}|x_{k}|$ converges, then $\sum_{k=1}^{\infty}x_{k}$ is **absolutely convergent**.
- If $\sum_{k=1}^{\infty}x_{k}$ converges, but $\sum_{k=1}^{\infty}|x_{k}|$ diverges, then $\sum_{k=1}^{\infty}x_{k}$ is **conditionally convergent**.

---

## 4.5 Rearrangements

### Rearrangement of Terms

1\. What is a rearrangement of a series, and how can it change the value of the sum?
2\. State the rearrangement theorem. Under what conditions can the sum of a series be rearranged to equal any value?

### Key Terms

#### Rearrangement of Terms in a Series (Page 134, 4.21)

Given a series $\sum_{k=1}^{\infty}a_{k}$, a rearrangement of the series is the series $\sum_{k=1}^{\infty}b_{k}$ for which there is a bijection $f: \mathbb{N} \to \mathbb{N}$, such that $b_{f(k)} = a_{k}$.

#### Rearrangement Theorem (Page 135, 4.23)

Given a conditionally convergent series, $\sum_{k=1}^{\infty} x_{k}$, for any $L$, such that $L \in \mathbb{R}$ or $L = \pm \infty$, there exists some rearrangement of $\sum_{k=1}^{\infty} x_{k}$ that converges to $L$.

See also: [Rearrangement of Terms in Convergent Series - Mathonline](http://mathonline.wikidot.com/rearrangement-of-terms-in-convergent-series), [Convergence of Rearranged Series of Real Numbers - Mathonline](http://mathonline.wikidot.com/convergence-of-rearranged-series-of-real-numbers)

---

## General Exercises

### Exploration

1\. Prove the ratio test for a series $\sum a_k$. What does it determine about convergence?
2\. Prove the root test for a series $\sum a_k$. How does it compare to the ratio test?
3\. Show that a series that converges absolutely must converge, but the converse is not always true. Provide an example.

### 4.4 Absolute and Conditional Convergence

#### Definition of Absolute Convergence

- A series $\sum a_k$ **converges absolutely** if:

$$
\sum |a_k| \text{ converges}.
$$

- **Theorem 4.18 (Absolute Convergence Implies Convergence)**:
  - If $\sum |a_k|$ converges, then $\sum a_k$ also converges.
  - **Absolute convergence guarantees stability under rearrangements.**

#### Conditional Convergence

- A series **converges conditionally** if:
  - $\sum a_k$ converges, but
  - $\sum |a_k|$ **diverges**.
- **Example: The Alternating Harmonic Series**

$$
\sum_{k=1}^{\infty} \frac{(-1)^{k+1}}{k}
$$

**converges**, but the harmonic series itself diverges.

---

### 4.5 Rearrangements of Series

#### Definition of a Rearrangement

- A **rearrangement** of $\sum a_k$ is a new series:

$$
\sum a_{\sigma(k)}
$$

where $\sigma$ is a permutation of $\mathbb{N}$.

#### Theorem 4.23 (Riemann's Rearrangement Theorem)

- If a series **converges conditionally**, then it can be **rearranged to sum to any real number** or **diverge to $\pm \infty$**.
- **Absolute convergence prevents this issue**.

---

### 4.6 Applications and Historical Insights

- **Zeno's Paradoxes**:
  - The sum of an **infinite number of decreasing time intervals** can be finite.
  - Example:

$$
t + \frac{t}{2} + \frac{t}{4} + \frac{t}{8} + \dots = 2t
$$

resolves the paradox of motion.

- **Euler's Basel Problem**:
  - Euler proved:

$$
\sum_{k=1}^{\infty} \frac{1}{k^2} = \frac{\pi^2}{6}.
$$

---

### Key Takeaways

- A **series is defined by the limit of its partial sums**.
- **The Divergence Test**: If $\lim a_k \neq 0$, the series **diverges**.
- **Geometric Series**: Converges if $|r| < 1$, diverges otherwise.
- **Harmonic Series** **diverges**, while the **$p$-series converges for $p > 1$**.
- **Absolute convergence implies convergence**, but **conditional convergence allows rearrangement to any sum**.
- **Riemann's Rearrangement Theorem**: Conditionally convergent series can be reordered to sum to any real number.
- **Euler's Basel Problem** connects series with $\pi$, showing deep relationships in analysis.

---

### Extra Proofs Related to Series

#### Shift Theorem for Series

Given a real sequence $\langle x_{k} \rangle_{k=1}^{\infty} \subseteq \mathbb{R}$ and a natural number $N \in \mathbb{N},$ the series $\sum_{k=1}^{\infty} x_{k}$ converges if and only if the tail series, $\sum_{k=N}^{\infty} x_{k}$ converges.

$$
\sum_{k=1}^{\infty} x_{k} ~ \text{Converges} \iff \sum_{k=N}^{\infty} x_{k} ~ \text{Converges}
$$

---

##### Proof

###### (⇒) Suppose $\sum_{k=1}^{\infty} x_{k}$ Converges

1\. Suppose $\sum_{k=1}^{\infty} x_{k}$ is a real, convergent series with a limit, $L \in \mathbb{R},$ and define the series's sequence of partial sums:

$$
s_{n} = \sum_{k=1}^{n} x_{k}
$$

2\. By definition of **series convergence**, since $\sum_{k=1}^{\infty} x_{k}$ converges, $s_{n}$ also converges, where $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies |s_{n} - L| < \varepsilon
$$

3\. By the **recursive definition of summation**, the series can be expressed as the sum of the first $N - 1$ terms plus the sum of the remaining terms, such that for any $n \geq N:$

$$
\sum_{k=1}^{n} x_{k} = \sum_{k=1}^{N-1} x_{k} + \sum_{k=N}^{n} x_{k}
$$

4\. Define $C$ and $t_{n}$ as the sequences of partial sums for the constant, finite sum and the tail partial sum, respectively:

$$
C = \sum_{k=1}^{N-1} x_{k} \quad \text{and} \quad t_{n} = \sum_{k=N}^{n} x_{k}
$$

5\. Rewriting the partial sums:

$$
s_{n} = C + t_{n}
$$

6\. Notice that since $C$ is a constant sequence, it converges to itself:

$$
\lim_{n \to \infty } C = C
$$

7\. Hence, by the **difference rule of convergent sequences**, since $s_{n} \to L$ and $C \to C:$

$$
t_{n} = s_{n} - C \to L - C
$$

8\. Thus, by definition of **series convergence**, since the sequence of partial sums, $t_{n},$ converges

$$
\sum_{k=N}^{\infty} x_{k} \quad \text{Converges}
$$

$$
\therefore ~ \boxed{ \sum_{k=1}^{\infty} x_{k} ~ \text{Converges} \implies \sum_{k=N}^{\infty} x_{k} ~ \text{Converges} }
$$

---

###### (⇐) Suppose $\sum_{k=N}^{\infty} x_{k}$ Converges

1\. Suppose $\sum_{k=N}^{\infty} x_{k}$ is a convergent series with limit $L \in \mathbb{R}$, and define the sequence of tail partial sums:

$$
t_{n} = \sum_{k=N}^{n} x_{k}
$$

2\. By definition of **series convergence**, since $\sum_{k=N}^{\infty} x_{k}$ converges, $t_{n}$ also converges, where $\forall \varepsilon > 0,$ $\exists N' \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N' \implies |t_{n} - L| < \varepsilon
$$

3\. By the **recursive definition of summation**, define the full series's sequence of partial sum as the sum of tail partial sum and the sum of the first $N - 1$ terms, such that for any $n \geq N:$

$$
s_{n} = \sum_{k=1}^{n} x_{k} = \sum_{k=1}^{N-1} x_{k} + \sum_{k=N}^{n} x_{k}
$$

4\. Define $C$ as the sequence of partial sums for the constant, finite sum:

$$
C = \sum_{k=1}^{N-1} x_{k}
$$

5\. Hence for all $n \geq N:$

$$
s_{n} = C + t_{n}
$$

6\. Notice that since $C$ is a constant sequence, it converges to itself:

$$
\lim_{n \to \infty } C = C
$$

7\. Hence, by the **sum rule of convergent sequences**, since $t_{n} \to L$ and $C \to C:$

$$
s_{n} = C + t_{n} \to C + L
$$

8\. Thus, by definition of **series convergence**, since the sequence of partial sums, $s_{n},$ converges

$$
\sum_{k=1}^{\infty} x_{k} \quad \text{Converges}
$$

$$
\therefore ~ \boxed{ \sum_{k=N}^{\infty} x_{k} ~ \text{Converges} \implies \sum_{k=1}^{\infty} x_{k} ~ \text{Converges} }
$$
