---
title: sum_09_sequences_and_series_of_functions_real_analysis_full
uuid: c11710e3-2846-456a-9f0b-327b5440a5ae
aliases:
  - "Summary of Real Analysis: Sequences and Series of Functions"
  - "Full Summary of Real Analysis: Sequences and Series of Functions"
  - "full summary of real analysis: sequences and series of functions"
  - full_summary_of_real_analysis_sequences_and_series_of_functions
  - sum_09_sequences_and_series_of_functions_real_analysis_full
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
  - "[[09_sequences_and_series_of_functions_real_analysis|Real Analysis: Sequences and Series of Functions]]"
about: |-
 Chapter 9 of Cummings’s _Real Analysis_ explores how sequences and series of functions converge, distinguishing between pointwise and uniform convergence. While pointwise convergence evaluates convergence at each individual point, uniform convergence ensures control across the entire domain, preserving key properties like continuity, integrability, and differentiability. The chapter presents conditions under which limits can be interchanged with integrals and derivatives and introduces the Weierstrass M-Test to guarantee uniform convergence of function series. It concludes with an introduction to power series, explaining their radius of convergence and showing that they converge uniformly on compact intervals within that radius, allowing term-by-term operations.
url:
status: develop
type: summary
file_class: pkm_zettel
date_created: 2025-02-14T12:49
date_modified: 2025-10-05T17:48
tags:
---
# Summary of Real Analysis: Sequences and Series of Functions

> [!Summary]
>
> - **Resource**: `dv: this.file.frontmatter.library[0]`
>
> - **Source**:: [[Cummings_2019_Real Analysis_09_Sequences and Series of Functions.pdf|Real Analysis: Sequences and Series of Functions, by Jay Cummings]]
>
> - **Parent**:: [[sum_09_sequences_and_series_of_functions_real_analysis|Summary of Real Analysis: Sequences and Series of Functions]]

---

## Section 9.1: Pointwise Convergence

### Guiding Questions

- What does it mean for a sequence of functions to converge pointwise?
- How does pointwise convergence relate to the behavior of functions at each point?
- Can pointwise limits of continuous functions be discontinuous?
- What are some examples that illustrate the subtleties of pointwise convergence?

### Key Terms

#### Pointwise Convergence (Page 261, Def 9.1.1)

A sequence of functions $\{f_n\}_{n=1}^{\infty}$ defined on a common domain $D$ **converges pointwise** to $f: D \to \mathbb{R}$ if:

$$
\forall x \in D, \quad \lim_{n \to \infty} f_n(x) = f(x)
$$

> [!Note]
> This means convergence is evaluated one point at a time.

#### Example: Sine Power Sequence (Page 261, Ex 9.1.2)

Let $f_n(x) = \sin^n(x)$ on $[0, \pi]$.

- The pointwise limit is:

$$
f(x) = \begin{cases}
0 & x \in (0, \pi) \\
1 & x = \frac{\pi}{2} \\
0 & x = 0, \pi
\end{cases}
$$

This function is not continuous even though each $f_n$ is continuous.

> [!Note]
> Pointwise convergence does not preserve continuity.

---

## Section 9.2: Uniform Convergence

### Guiding Questions

- What is uniform convergence and how does it strengthen pointwise convergence?
- Why does uniform convergence preserve continuity?
- How is uniform convergence defined using the supremum norm?

### Key Terms

#### Uniform Convergence (Page 262, Def 9.2.1)

A sequence of functions $\{f_n\}$ on $D$ **converges uniformly** to $f$ if:

$$
\forall \varepsilon > 0, \ \exists N \in \mathbb{N} \text{ such that } \forall n \geq N, \ \forall x \in D, \ |f_n(x) - f(x)| < \varepsilon
$$

> [!Note]
> The convergence is **uniform** in $x$, meaning $N$ does not depend on $x$.

#### Comparison with Pointwise (Page 263, Ex 9.2.2)

Revisiting $f_n(x) = \sin^n(x)$: the convergence is **not uniform** on $[0, \pi]$ because near $x = \pi/2$ the decay is slow.

---

## Section 9.3: Preserving Continuity

### Guiding Questions

- Under what conditions does the limit of continuous functions remain continuous?
- How does uniform convergence affect continuity?
- Can a uniformly converging sequence of discontinuous functions yield a continuous limit?

### Key Terms

#### Uniform Limit Theorem (Page 264, Thm 9.3.1)

If $\{f_n\}$ are continuous functions on $D$ and $f_n \to f$ **uniformly**, then $f$ is continuous on $D$.

**Proof**:

1\. Let $\varepsilon > 0$. Since $f_n \to f$ uniformly, choose $N$ such that:

$$
\forall x \in D, \quad |f_N(x) - f(x)| < \varepsilon/3
$$

2\. $f_N$ is continuous, so $\exists \delta > 0$ such that $|x - y| < \delta \Rightarrow |f_N(x) - f_N(y)| < \varepsilon/3$

3\. Then for $|x - y| < \delta$:

$$
|f(x) - f(y)| \leq |f(x) - f_N(x)| + |f_N(x) - f_N(y)| + |f_N(y) - f(y)| < \varepsilon
$$

So $f$ is continuous.

---

## Section 9.4: Uniform Convergence and Integration

### Guiding Questions

- When can we interchange limits and integrals?
- Does pointwise convergence guarantee integral convergence?
- What role does uniform convergence play in integration?

### Key Terms

#### Theorem: Limit of Integrals (Page 265, Thm 9.4.1)

Let $f_n \to f$ uniformly on $[a, b]$, and each $f_n$ is integrable. Then:

$$
\lim_{n \to \infty} \int_a^b f_n(x) \, dx = \int_a^b f(x) \, dx
$$

**Proof Sketch**:

1\. Use the inequality:

$$
\left| \int_a^b f_n - \int_a^b f \right| \leq \int_a^b |f_n - f|
$$

2\. Since $f_n \to f$ uniformly, $\sup |f_n - f| < \varepsilon / (b - a)$ for large $n$

3\. So integral of the absolute value is small, hence convergence.

> [!Note]
> Uniform convergence justifies limit–integral interchange.

---

## Section 9.5: Uniform Convergence and Differentiation

### Guiding Questions

- When is it valid to differentiate under the limit?
- What additional conditions are required for interchanging limit and derivative?
- Can uniform convergence alone guarantee differentiability?

### Key Terms

#### Theorem: Differentiation under the Limit (Page 266, Thm 9.5.1)

Suppose:

- $f_n$ differentiable on $[a, b]$
- $f_n \to f$ pointwise
- $f_n'$ converges **uniformly** to $g$

Then $f$ is differentiable and:

$$
f' = g
$$

**Proof Sketch**:

1\. Use the Mean Value Theorem:

$$
\frac{f_n(x + h) - f_n(x)}{h} = f_n'(\xi_n)
$$

for some $\xi_n \in (x, x+h)$.

2\. Take limits, apply uniform convergence of $f_n'$.

> [!Note]
> Pointwise convergence of $f_n$ alone is not enough; derivative convergence must be uniform.

---

## Section 9.6: Sequences vs. Series of Functions

### Guiding Questions

- What's the difference between a sequence and a series of functions?
- How is convergence of a function series defined?
- Can we treat convergence of $\sum f_n$ pointwise or uniformly?

### Key Terms

#### Function Series Convergence (Page 268, Def 9.6.1)

Given a series of functions:

$$
\sum_{n=1}^{\infty} f_n(x)
$$

We define:

- **Pointwise convergence**: the sequence of partial sums $S_n(x) = \sum_{k=1}^n f_k(x)$ converges pointwise
- **Uniform convergence**: $S_n(x)$ converges uniformly to $f(x)$

> [!Note]
> Results from sequences apply to function series via their partial sums.

---

## Section 9.7: Weierstrass M-Test

### Guiding Questions

- How can we test for uniform convergence of a function series?
- What is the role of bounding functions in establishing convergence?
- What are the consequences of the Weierstrass M-Test?

### Key Terms

#### Weierstrass M-Test (Page 269, Thm 9.7.1)

Suppose:

- $|f_n(x)| \leq M_n$ for all $x \in D$
- $\sum M_n$ converges

Then $\sum f_n(x)$ converges **uniformly** and **absolutely** on $D$.

**Proof**:

1\. Define $S_n(x) = \sum_{k=1}^n f_k(x)$ and $S(x) = \sum f_k(x)$

2\. For $n > m$:

$$
|S_n(x) - S_m(x)| = \left| \sum_{k=m+1}^n f_k(x) \right| \leq \sum_{k=m+1}^n M_k
$$

3\. RHS is independent of $x$ and goes to 0 as $m, n \to \infty$ (since $\sum M_k$ converges)

> [!Note]
> Uniform convergence ensures continuity and integrability of the limit.

---

## Section 9.8: Power Series

### Guiding Questions

- What is a power series and when does it converge?
- How is the radius of convergence defined and computed?
- What are the properties of power series within their interval of convergence?

### Key Terms

#### Power Series (Page 271, Def 9.8.1)

A **power series centered at $c$** is:

$$
\sum_{n=0}^{\infty} a_n (x - c)^n
$$

#### Radius of Convergence (Page 272, Def 9.8.2)

Let $R = \sup \{ r \in \mathbb{R}: \sum a_n (x - c)^n \text{ converges for } |x - c| < r \}$

- The series converges absolutely for $|x - c| < R$
- Diverges for $|x - c| > R$

> [!Note]
> Use the ratio or root test to compute $R$.

#### Theorem: Power Series Converge Uniformly on Compact Subintervals (Page 273, Thm 9.8.3)

If $R > 0$, then for every $\delta < R$, the power series converges **uniformly** on $[c - \delta, c + \delta]$.

> [!Note]
> Uniform convergence on compact sets gives term-by-term differentiability and integrability.

---
