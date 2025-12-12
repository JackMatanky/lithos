---
title: sum_06_continuity_real_analysis_full
uuid: caecb34c-e4a2-487c-9d93-39541a34d87e
aliases:
  - "Full Summary of Real Analysis: Continuity"
  - "full summary of real analysis: continuity"
  - full_summary_of_real_analysis_continuity
  - sum_06_continuity_real_analysis_full
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
  - "[[06_continuity_real_analysis|Real Analysis: Continuity]]"
about: |-
 Chapter 6 of Cummings’s _Real Analysis_ introduces the concept of continuity, beginning with the $\varepsilon$–$\delta$ definition and exploring its implications for function behavior. It establishes foundational results such as the algebra of continuous functions, continuity of compositions, and the sequential characterization of continuity. The chapter categorizes different types of discontinuities—removable, jump, and infinite—and defines continuity at the endpoints of closed intervals. It culminates in two powerful theorems,

 - The Intermediate Value Theorem, which guarantees the existence of intermediate values for continuous functions on closed intervals;
 - The Extreme Value Theorem, which ensures that continuous functions on closed intervals attain maximum and minimum values.

 Finally, the chapter distinguishes between continuity and uniform continuity, proving that the latter holds automatically for continuous functions on closed intervals.
url:
status: develop
type: summary
file_class: pkm_zettel
date_created: 2025-02-14T11:55
date_modified: 2025-10-05T17:48
tags:
---
# Full Summary of Real Analysis: Continuity

> [!Summary]
>
> - **Resource**: `dv: this.file.frontmatter.library[0]`
>
> - **Source**:: [[Cummings_2019_Real Analysis_06_Continuity.pdf|Real Analysis: Continuity, by Jay Cummings]]
>
> - **Parent**:: [[sum_06_continuity_real_analysis|Summary of Real Analysis: Continuity]]

---

## Section 6.1: Continuity

### Guiding Questions

- What is the formal definition of continuity at a point for a function?
- How does continuity relate to limits?
- How can we verify whether a function is continuous at a point?
- What are some basic properties of continuous functions?

### Key Terms

#### Continuity at a Point (Page 114, Def 6.1.1)

A function $f: A \to \mathbb{R}$ is **continuous at a point** $a \in A$ if for all $\varepsilon > 0$, there exists a $\delta > 0$ such that for all $x \in A$, if $|x - a| < \delta$, then $|f(x) - f(a)| < \varepsilon$.

> [!Note]
> This is an $\varepsilon$–$\delta$ definition analogous to the limit definition.

#### Discontinuity (Page 114, Def 6.1.2)

A function is **discontinuous at a point** $a \in A$ if it is not continuous at $a$.

#### Continuous Function (Page 114, Def 6.1.3)

A function $f: A \to \mathbb{R}$ is **continuous** if it is continuous at every point in its domain $A$.

#### Basic Continuous Functions (Page 115, Ex 6.1.4)

The functions $f(x) = x^n$ and $f(x) = c$ for any constant $c$ are continuous on $\mathbb{R}$.

---

## Section 6.2: Compositions and Algebra of Continuous Functions

### Guiding Questions

- What operations preserve continuity of functions?
- How does continuity behave under addition, multiplication, and composition?
- Are inverses of continuous functions also continuous?

### Key Terms

#### Algebra of Continuous Functions (Page 116, Thm 6.2.1)

Let $f, g: A \to \mathbb{R}$ be continuous at $a \in A$. Then:

1. $f + g$ is continuous at $a$
2. $f - g$ is continuous at $a$
3. $f \cdot g$ is continuous at $a$
4. If $g(a) \neq 0$, then $f/g$ is continuous at $a$

> [!Note]
> These follow directly from the limit laws.

#### Continuity of Composition (Page 117, Thm 6.2.2)

If $f: A \to \mathbb{R}$ is continuous at $a \in A$, and $g: f(A) \to \mathbb{R}$ is continuous at $f(a)$, then the composition $g \circ f$ is continuous at $a$.

---

## Section 6.3: Sequential Continuity

### Guiding Questions

- How can we express continuity using sequences?
- What are the advantages of using sequences to analyze continuity?
- Is the sequential definition of continuity equivalent to the $\varepsilon$–$\delta$ definition?

### Key Terms

#### Sequential Continuity (Page 118, Thm 6.3.1)

A function $f: A \to \mathbb{R}$ is continuous at $a \in A$ if and only if for every sequence $(x_n) \subseteq A$ with $x_n \to a$, we have $f(x_n) \to f(a)$.

> [!Note]
> This gives a useful alternative to the $\varepsilon$–$\delta$ approach and is often easier to verify.

##### Proof of Theorem 6.3.1: Sequential Characterization of Continuity

**Theorem:** A function $f: A \to \mathbb{R}$ is continuous at $a \in A$ if and only if for all sequences $x_n \to a$, it holds that $f(x_n) \to f(a)$.

**Proof:**

1\. Assume $f$ is continuous at $a$, and let $x_n \to a$.
 - By Def 6.1.1, for all $\varepsilon > 0$, there exists $\delta > 0$ such that $|x - a| < \delta \Rightarrow |f(x) - f(a)| < \varepsilon$.
 - Since $x_n \to a$, there exists $N \in \mathbb{N}$ such that $n \geq N \Rightarrow |x_n - a| < \delta$.
 - Hence $|f(x_n) - f(a)| < \varepsilon$, so $f(x_n) \to f(a)$.

2\. Conversely, assume that for every sequence $x_n \to a$, we have $f(x_n) \to f(a)$.
 - Suppose $f$ is not continuous at $a$.
 - Then there exists $\varepsilon_0 > 0$ such that for all $\delta > 0$, there exists $x \in A$ with $|x - a| < \delta$ and $|f(x) - f(a)| \geq \varepsilon_0$.
 - Construct a sequence $x_n \to a$ with $|f(x_n) - f(a)| \geq \varepsilon_0$.
 - Then $f(x_n) \not\to f(a)$, contradicting the assumption.
 - Thus, $f$ must be continuous at $a$.

---

## Section 6.4: Types of Discontinuities

### Guiding Questions

- What different types of discontinuities exist?
- How can we distinguish removable, jump, and infinite discontinuities?
- Can we fix discontinuities by redefining functions?

### Key Terms

#### Types of Discontinuities (Page 120, Def 6.4.1)

1. **Removable Discontinuity:** Limit exists, but is not equal to function value.
2. **Jump Discontinuity:** Left and right limits exist but are not equal.
3. **Infinite Discontinuity:** One-sided limit diverges to $\pm \infty$.

> [!Note]
> These classifications help in understanding behavior near the discontinuity.

#### Example (Page 121, Ex 6.4.2)

Let:

$$
f(x) =
\begin{cases}
\frac{\sin x}{x}, & x \neq 0 \\
0, & x = 0
\end{cases}
$$

- This has a **removable discontinuity** at $x = 0$.

---

## Section 6.5: Continuity on Closed Intervals

### Guiding Questions

- What does it mean for a function to be continuous on a closed interval?
- How do we define continuity at endpoints?
- Can we extend continuity to closed domains?

### Key Terms

#### Endpoint Continuity (Page 122, Def 6.5.1)

Let $f: [a,b] \to \mathbb{R}$. Then:

- $f$ is continuous at $a$ if $\lim_{x \to a^+} f(x) = f(a)$
- $f$ is continuous at $b$ if $\lim_{x \to b^-} f(x) = f(b)$

---

## Section 6.6: Intermediate Value Theorem

### Guiding Questions

- What is the Intermediate Value Theorem and why is it important?
- What conditions are required for it to apply?
- How does it help in proving existence of roots?

### Key Terms

#### Intermediate Value Theorem (Page 123, Thm 6.6.1)

If $f: [a,b] \to \mathbb{R}$ is continuous and $f(a) < 0 < f(b)$, then there exists $c \in (a,b)$ such that $f(c) = 0$.

> [!Note]
> The IVT ensures the function hits every value between $f(a)$ and $f(b)$.

##### Proof of Theorem 6.6.1: Intermediate Value Theorem

**Theorem:** Let $f: [a,b] \to \mathbb{R}$ be continuous with $f(a) < 0 < f(b)$. Then $\exists c \in (a,b)$ such that $f(c) = 0$.

**Proof:**

1\. Define the set $S = \{ x \in [a,b] \mid f(x) < 0 \}$.
 - $S$ is nonempty (since $f(a) < 0$) and bounded above by $b$.

2\. Let $c = \sup S$.
 - Then $a \leq c \leq b$.

3\. Show $f(c) = 0$.
 - Suppose $f(c) < 0$: then $\exists \varepsilon > 0$ such that $f(x) < 0$ on $(c, c + \varepsilon)$, contradicting $c$ being the supremum.
 - Suppose $f(c) > 0$: then $\exists \varepsilon > 0$ such that $f(x) > 0$ on $(c - \varepsilon, c)$, contradicting $f(x) < 0$ just below $c$.
 - Therefore, $f(c) = 0$.

---

## Section 6.7: Extreme Value Theorem

### Guiding Questions

- What conditions guarantee that a function attains a maximum or minimum?
- How do compactness and continuity relate to boundedness?
- Can the EVT fail if the domain is not closed and bounded?

### Key Terms

#### Extreme Value Theorem (Page 125, Thm 6.7.1)

If $f: [a,b] \to \mathbb{R}$ is continuous, then there exist $c, d \in [a,b]$ such that:

- $f(c) \leq f(x) \leq f(d)$ for all $x \in [a,b]$

> [!Note]
> That is, $f$ achieves its minimum and maximum values.

---

## Section 6.8: Uniform Continuity

### Guiding Questions

- What is the difference between continuity and uniform continuity?
- Why is uniform continuity stronger?
- What functions are uniformly continuous on bounded intervals?

### Key Terms

#### Uniform Continuity (Page 126, Def 6.8.1)

A function $f: A \to \mathbb{R}$ is **uniformly continuous** on $A$ if for all $\varepsilon > 0$, there exists $\delta > 0$ such that for all $x, y \in A$, if $|x - y| < \delta$, then $|f(x) - f(y)| < \varepsilon$.

#### Uniform Continuity on Closed Intervals (Page 126, Thm 6.8.2)

If $f$ is continuous on a closed interval $[a,b]$, then $f$ is uniformly continuous on $[a,b]$.

##### Proof of Theorem 6.8.2: Uniform Continuity on Closed Intervals

**Theorem:** If $f: [a,b] \to \mathbb{R}$ is continuous, then $f$ is uniformly continuous.

**Proof (by contradiction):**

1\. Suppose not: then $\exists \varepsilon_0 > 0$ such that $\forall \delta > 0$, there exist $x, y \in [a,b]$ with $|x - y| < \delta$ and $|f(x) - f(y)| \geq \varepsilon_0$.

2\. Construct sequences $x_n, y_n$ such that $|x_n - y_n| < \frac{1}{n}$ and $|f(x_n) - f(y_n)| \geq \varepsilon_0$.

3\. By compactness of $[a,b]$, $x_n$ has a convergent subsequence $x_{n_k} \to c \in [a,b]$.
 - Then $y_{n_k} \to c$ also (since distance between $x_n$ and $y_n$ goes to zero).

4\. Continuity implies $f(x_{n_k}) \to f(c)$ and $f(y_{n_k}) \to f(c)$, so $|f(x_{n_k}) - f(y_{n_k})| \to 0$, contradiction.
