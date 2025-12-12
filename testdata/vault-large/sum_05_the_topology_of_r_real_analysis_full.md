---
title: sum_05_the_topology_of_r_real_analysis_full
uuid: d2a735a5-f44e-4d37-92b2-37e10af1f07c
aliases:
  - "Full Summary of Real Analysis: The Topology of $\\\\mathbb{R}$"
  - "Full Summary of Real Analysis: The Topology of R"
  - "full summary of real analysis: the topology of r"
  - full_summary_of_real_analysis_the_topology_of_r
  - sum_05_the_topology_of_r_real_analysis_full
pillar:
  - "[[knowledge_expansion|Knowledge Expansion]]"
category:
  - "[[formal_science|Formal Science]]"
branch:
  - "[[mathematics|Mathematics]]"
field:
  - "[[calculus|Calculus]]"
  - "[[real_analysis|Real Analysis]]"
  - "[[topology|Topology]]"
subject:
topic:
subtopic:
library:
  - "[[05_the_topology_of_r_real_analysis|Real Analysis: The Topology of R]]"
  - "[[cummings_2019_real_analysis|Real Analysis: A Long-form Mathematics Textbook]]"
about: |-
 This chapter introduces fundamental topological concepts in real analysis, including open sets, closed sets, and compact sets. These ideas lay the foundation for the study of continuous functions and demonstrate the interconnectedness of different branches of mathematics.
 This chapter establishes the fundamental ideas of topology that are essential in real analysis, particularly in understanding continuous functions and convergence.
url:
status: develop
type: summary
file_class: pkm_zettel
date_created: 2025-02-04T23:00
date_modified: 2025-10-05T17:48
tags:
---
# Full Summary of Real Analysis: The Topology of $\mathbb{R}$

> [!Summary]
>
> - **Resource**: `dv: this.file.frontmatter.library[0]`
>
> - **Source**:: [[Cummings_2019_Real Analysis_05_The Topology of R.pdf|Real Analysis: The Topology of R, by Jay Cummings]]
>
> - **Parent**:: [[sum_05_the_topology_of_r_real_analysis|Summary of Real Analysis: The Topology of R]]

---

> [!Quote]
>
> Nevertheless, this brief introduction will aim to do twro things. The first is practical: It lays the foundation for our studv of continuous functions by introducing special classes of sets which, tvhen wre ask for a function's behavior on these sets, will determine whether that function is continuous. The second is more philosophical. Although math is traditionally taught by first partitioning it into discrete2, non overlapping areas like real analysis, abstract algebra, linear algebra, combinatorics and topology, mathematics is in reality much more interconnected than this and one's mathematical education should include examples thereof.

## 5.1 Open Sets

### Guiding Questions

- What does it mean for a set to be open?
- How can open sets be constructed?
- What are some key properties of open sets?

### Key Terms

#### Definition of Open Sets in $\mathbb{R}$ (Page 149, 5.1)

See also: [Definition: Open Set in Real Analysis - ProofWiki](https://proofwiki.org/wiki/Definition:Open_Set/Real_Analysis)

##### Primary Definition

- A real set $U \subseteq \mathbb{R},$ is open if and and only if, for every $x \in U$, there exists some $\delta \in \mathbb{R}_{> 0},$ for any $y \in \mathbb{R},$ such that if $y$ satisfies $|x - y| < \delta,$ then $y \in U.$

$$
\begin{align}
& \forall x \in U \, \Bigl[ \, \exists \delta \in \mathbb{R}_{>0} \, \bigl[ \, \forall y \in \mathbb{R} \\
& \quad \left( \, |x - y| < \delta \implies y \in U \, \right) \bigr] \Bigr]
\end{align}
$$

> [!Note]
>
> This is the **core logical definition**, and everything else can be derived from it.

##### Interval Definition

- A real set, $U \subseteq \mathbb{R},$ is open if and and only if, for every $x \in U$, there exists some $\delta \in \mathbb{R}_{> 0},$ such that the open interval $(x - \delta, x + \delta)$ is a subset of $U.$

$$
\begin{align}
& \forall x \in U \,\Bigl[\, \exists \delta \in \mathbb{R}_{>0} \\
& \quad \bigl( (x - \delta,\, x + \delta) \subseteq U \bigr) \Bigr]
\end{align}
$$

##### Neighborhood Definitions

> [!Definition] $\delta$-Neighborhood Notation
>
> The $\delta$-neighborhood of a point $x \in \mathbb{R}$, denoted $V_{\delta}(x)$, is the set of all real numbers $y$ such that the distance between $y$ and $x$ is less than $\delta.$
>
> $$
> V_{\delta}(x) := \{ y \in \mathbb{R} : |x - y| < \delta \}
> $$

A set $U \subseteq \mathbb{R}$ is open if and and only if, for all $x \in U,$ there exists some $\delta$-neighborhood of $x,$ denoted $V_{\delta}(x),$ such that $V_{\delta}(x) \subseteq U.$

$$
\forall x \in U \, \bigl[\, \exists \delta \in \mathbb{R}_{>0} \, \left( V_{\delta}(x) \subseteq U \,\right)\, \bigr]
$$

##### Definition of Open Sets in $\mathbb{R}^n$

###### Euclidean Space Definition

Let $U \subseteq \mathbb{R}^n$, with $n \geq 1$. Then $U$ is open if and only if:

- **Open Ball Formulation**:

$$
\forall x \in U,\ \exists R \in \mathbb{R}_{>0},\ B(x, R) \subset U
$$

Where:

- $B(x, R):= \{ y \in \mathbb{R}^n: \|y - x\| < R \}$ is the open Euclidean ball centered at $x$ of radius $R$.

##### Examples of Open Sets (Page 150, 5.2)

- The set $\mathbb{R}$ is open.
- The empty set $\emptyset$ is open.

###### Proof: $\emptyset$ is an Open Set

1\. Assume, for **contradiction**, the empty set, $\emptyset$ is not open.

2\. By the contrapositive definition of an **open set**, $\exists x \in U,$ $\forall \delta > 0,$ such that:

$$
V_{\delta}(x) \not \subseteq U
$$

3\. However, this contradicts the definition of the empty set as containing no elements.

$$
\therefore ~ \boxed{ \emptyset ~\text{is an Open Set}}
$$

###### Proof: $(a, b)$ is an Open Set

1\. Let $(a, b)$ is an open interval:

$$
(a, b) = \{ x \in \mathbb{R} \mid a < x < b \}
$$

2\. Given any arbitrary point, $x \in (a, b),$ define:

$$
\delta := \min\{ x - a, b - x \}
$$

3\. By definition of **minimum**, since $x - a$ and $b - x$ are strictly positive, $\delta > 0$:

$$
\begin{align}
& x - a > 0 \quad  \text{and} \quad  b - x > 0  \\
& \quad  \implies \delta > 0
\end{align}
$$

4\. Consider the $\delta$-neighborhood of $x,$ $V_{\delta}(x),$ and let $y \in V_{\delta}(x):$

$$
V_\delta(x) := \{ y \in \mathbb{R} : |x - y| < \delta \}
$$

5\. By definition of the $\delta$-neighborhood of $x,$ for any $y \in V_{\delta}(x):$

$$
\begin{align}
& |x - y| < \delta \implies \\
& \quad x + \delta > y > x - \delta \tag{1}
\end{align}
$$

6\. By definition of $\delta:$

$$
\begin{align}
\delta \leq x - a &\implies x - \delta \geq a \tag{2}\\
\delta \leq b - x &\implies x + \delta \leq b \tag{3}
\end{align}
$$

7\. Combining $(2)$ and $(3)$ with the result of $(1)$ and simplifying:

$$
\begin{gather}
a \leq x - \delta < y < x + \delta \leq b \\
a < y < b
\end{gather}
$$

8\. Hence, by definition of open intervals:

$$
y \in V_{\delta}(x) \implies y \in (a, b)
$$

9\. Thus, by definition of open sets, $(a, b)$ is open since $\forall x \in (a, b),$ $\exists V_{\delta}(x)$ such that:

$$
V_{\delta}(x) \subseteq (a, b)
$$

$$
\therefore ~ \boxed{ (a, b) ~ \text{is an Open Set}}
$$

###### Proof: $(a, \infty)$ is an Open Set

1\. Let $(a, \infty)$ is an open, right unbounded interval:

$$
(a, \infty) = \{ x \in \mathbb{R} \mid x > a \}
$$

2\. Given any arbitrary point, $x \in (a, \infty),$ define:

$$
\delta := x - a
$$

3\. Since $x > a$ implies $x - a > 0,$ $\delta > 0.$

4\. Consider the $\delta$-neighborhood of $x:$

$$
\begin{align}
V_\delta(x) &:= (x - \delta, x + \delta) \\
&= (x - (x - a), x + (x - a)) \\
&= (a, x + \delta)
\end{align}
$$

5\. Since $x + \delta > x > a$, it follows that:

$$
V_{\delta}(x) \subseteq (a, \infty)
$$

6\. Thus, by definition of **open sets**, $(a, \infty)$ is open since $\forall x \in (a, \infty),$ $\exists \delta \in \mathbb{R}_{> 0},$ such that:

$$
V_{\delta}(x) \subseteq (a, \infty)
$$

$$
\therefore ~ \boxed{ (a, \infty) ~ \text{is an Open Set}}
$$

###### Proof: $(-\infty, b)$ is an Open Set

1\. Let $(-\infty, b)$ be the open, left-unbounded interval:

$$
(-\infty, b) = \{ x \in \mathbb{R} \mid x < b \}
$$

2\. Given any arbitrary point $x \in (-\infty, b)$, define:

$$
\delta := b - x
$$

3\. Since $x < b$, it follows that $b - x > 0$, so $\delta > 0$.

4\. Consider the $\delta$-neighborhood of $x$:

$$
\begin{align}
V_\delta(x) &:= (x - \delta, x + \delta) \\
&= (x - (b - x), x + (b - x)) \\
&= (2x - b,\ b)
\end{align}
$$

5\. Since $2x - b < x < b$, it follows that:

$$
V_\delta(x) \subseteq (-\infty, b)
$$

6\. Thus, by definition of **open sets**, $(-\infty, b)$ is open since for every $x \in (-\infty, b)$, there exists $\delta > 0$ such that:

$$
V_{\delta}(x) \subseteq (-\infty, b)
$$

$$
\therefore ~ \boxed{ (-\infty, b) ~ \text{is an Open Set}}
$$

- Non-example: The closed interval $[3,7]$ is not open since it includes its endpoints.

##### Equivalence of Pointwise and Set-Theoretic Statements

To say that a set $A \subseteq \mathbb{R}$ is **open** means:

> [!Definition]
>
> For every point $x \in A$, there exists $\delta > 0$ such that:
>
> $$
> V_\delta(x) \subseteq A.
> $$

This is the pointwise definition of openness.

So when we prove:

- that $\bigcup_\alpha U_\alpha$ is open, or
- that $\bigcap_{k=1}^n U_k$ is open,

we are showing that **every point** in these sets has a $\delta$-neighborhood contained entirely within the set. That directly satisfies the definition of openness.

#### Open Sets via Arbitrary Unions and Finite Intersections (Page 151, 5.3)

> [!Info] See also
>
> [The Union and Intersection of Collections of Open Sets - Mathonline](http://mathonline.wikidot.com/the-union-and-intersection-of-collections-of-open-sets)

1. If $\{ U_{\alpha} \}$ is any collection of open sets, then $\bigcup_{\alpha} U_{\alpha}$ is also an open set.
	- The union of any collection of open sets is open.
2. If $\{ U_{1}, U_{2}, \ldots, U_{n} \}$ is any finite collection of open sets, then $\bigcap_{k = 1}^{n}U_{k}$ is also an open set.
	- The finite intersection of open sets is open.

##### Proof: The Union of Open Sets is Open

1\. Suppose $\{ U_{\alpha} \}$ is any collection of real, open sets.

2\. Let $x \in \bigcup_{\alpha} U_{\alpha}.$

3\. By definition of **set union**, there exists a set $\alpha_{0},$ such that $x \in U_{\alpha_{0}}.$

4\. By **subset transitivity**, since $U_{\alpha_{0}} \subseteq \bigcup_{\alpha} U_{\alpha}$ is open, $\exists \delta > 0,$ such that the $\delta$-neighborhood of $x$ is contained in $U_{\alpha_{0}}:$

$$
V_\delta(x) \subseteq U_{\alpha_{0}}
$$

5\. By definition of **union** and **subset transitivity**:

$$
V_\delta(x) \subseteq U_{\alpha_{0}} \implies V_{\delta}(x) \subseteq \bigcup_{\alpha} U_{\alpha}
$$

6\. Thus, by definition of **open sets**, $\bigcup_\alpha U_\alpha$ is open, since $\forall x \in \bigcup_{\alpha} U_{\alpha},$ $\exists \delta > 0$ such that:

$$
V_\delta(x) \subseteq \bigcup_\alpha U_\alpha
$$

$$
\boxed{ \therefore ~ \forall x \left[ x \in \bigcup_{\alpha} U_{\alpha} \implies \exists \delta > 0  \left( V_{\delta}(x) \subseteq \bigcup_{\alpha} U_{\alpha} \right) \right]}
$$

##### Proof: Finite Intersections of Open Sets Are Open

1\. Suppose $\{ U_{1}, U_{2}, \ldots, U_{n} \}$ is any finite collection of real, open sets.

2\. Let $x \in \bigcap_{k = 1}^{n} U_{k}$

3\. By definition of **set intersection**, $x \in U_{k},$ $\forall k \in \{1, 2, \ldots, n\}.$

4\. By definition of **open sets**, for each $\forall k \in \{1, 2, \ldots, n\},$ since $x \in U_{k}$ and $U_{k}$ is open, $\exists \delta_{k} > 0$ such that the $\delta_{k}$-neighborhood of $x$ is contained in $U_{k}:$

$$
V_{\delta_{k}}(x) \subseteq U_{k}
$$

5\. Define $\delta = \min \{ \delta_{1}, \delta_{2}, \ldots, \delta_{n} \}.$

6\. By **definition of minimum** and **neighborhood inclusion**, $\forall k \in \{1, 2, \ldots, n\}:$

$$
V_{\delta}(x) \subseteq V_{\delta_{k}}(x) \subseteq U_{k}
$$

7\. Thus, by the definition of **open sets**, $\bigcap_{k = 1}^{n} U_{k}$ is open, since $\forall x \in \bigcap_{k = 1}^{n} U_{k},$ $\exists \delta > 0$ such that:

$$
V_{\delta}(x) \subseteq \bigcap_{k = 1}^{n} U_{k}
$$

$$
\boxed{ \therefore ~ \forall x \left[ x \in \bigcap_{k = 1}^{n} U_{k} \implies \exists \delta > 0  \left( V_{\delta}(x) \subseteq \bigcap_{k = 1}^{n} U_{k} \right) \right]}
$$

1. If \\(\\\\{ U\_{\alpha} \\\\}\\) is any collection of open sets, then \\(\bigcup\_{\alpha} U\_{\alpha}\\) is also an open set.
	- The union of any collection of open sets is open.
2. If \\(\\\\{ U\_{1}, U\_{2}, \ldots, U\_{n} \\\\}\\) is any finite collection of open sets, then \\(\bigcap\_{k = 1}\^{n}U\_{k}\\) is also an open set.
	- The finite intersection of open sets is open.

##### Proof: The Union of Open Sets is Open

1. Suppose \\(\\\\{ U\_{\alpha} \\\\}\\) is any collection of real, open sets.
2. Let \\(x \in \bigcup\_{\alpha} U\_{\alpha}.\\)
3. By definition of **set union**, there exists a set \\(\alpha\_{0},\\) such that \\(x \in U\_{\alpha\_{0}}.\\)
4. By **subset transitivity**, since \\(U\_{\alpha\_{0}} \subseteq \bigcup\_{\alpha} U\_{\alpha}\\) is open, \\(\exists \delta > 0,\\) such that the \\(\delta\\)-neighborhood of \\(x\\) is contained in \\(U\_{\alpha\_{0}}:\\)

\\[
V\_\delta(x) \subseteq U\_{\alpha\_{0}}
\\]

1. By definition of **union** and **subset transitivity**:

\\[
V\_\delta(x) \subseteq U\_{\alpha\_{0}} \implies V\_{\delta}(x) \subseteq \bigcup\_{\alpha} U\_{\alpha}
\\]

1. Thus, by definition of **open sets**, \\(\bigcup\_\alpha U\_\alpha\\) is open, since \\(\forall x \in \bigcup\_{\alpha} U\_{\alpha},\\) \\(\exists \delta > 0\\) such that:

\\[
V\_\delta(x) \subseteq \bigcup\_\alpha U\_\alpha
\\]

\\[
\boxed{ \therefore ~ \forall x \left[ x \in \bigcup\_{\alpha} U\_{\alpha} \implies \exists \delta > 0 \left(V\_{\delta}(x) \subseteq \bigcup\_{\alpha} U\_{\alpha} \right) \right]}
\\]

##### Proof: Finite Intersections of Open Sets Are Open

1. Suppose \\(\\\\{ U\_{1}, U\_{2}, \ldots, U\_{n} \\\\}\\) is any finite collection of real, open sets.
2. Let \\(x \in \bigcap\_{k = 1}\^{n} U\_{k}\\)
3. By definition of **set intersection**, \\(x \in U\_{k},\\) \\(\forall k \in \\\\{1, 2, \ldots, n\\\\}.\\)
4. By definition of **open sets**, for each \\(\forall k \in \\\\{1, 2, \ldots, n\\\\},\\) since \\(x \in U\_{k}\\) and \\(U\_{k}\\) is open, \\(\exists \delta\_{k} > 0\\) such that the \\(\delta\_{k}\\)-neighborhood of \\(x\\) is contained in \\(U\_{k}:\\)

\\[
V\_{\delta\_{k}}(x) \subseteq U\_{k}
\\]

1. Define \\(\delta = \min \\\\{ \delta\_{1}, \delta\_{2}, \ldots, \delta\_{n} \\\\}.\\)
2. By **definition of minimum** and **neighborhood inclusion**, \\(\forall k \in \\\\{1, 2, \ldots, n\\\\}:\\)

\\[
V\_{\delta}(x) \subseteq V\_{\delta\_{k}}(x) \subseteq U\_{k}
\\]

1. Thus, by the definition of **open sets**, \\(\bigcap\_{k = 1}\^{n} U\_{k}\\) is open, since \\(\forall x \in \bigcap\_{k = 1}\^{n} U\_{k},\\) \\(\exists \delta > 0\\) such that:

\\[
V\_{\delta}(x) \subseteq \bigcap\_{k = 1}\^{n} U\_{k}
\\]

\\[
\boxed{ \therefore ~ \forall x \left[ x \in \bigcap\_{k = 1}\^{n} U\_{k} \implies \exists \delta > 0 \left(V\_{\delta}(x) \subseteq \bigcap\_{k = 1}\^{n} U\_{k} \right) \right]}
\\]

> [!Note] Arbitrary Unions
>
> - If $\{ U_{\alpha} \}$ contains **finitely many sets**, then $\bigcup_{\alpha}U_{\alpha}$ equals $\bigcup_{k = 1}^{n} U_{k}$
> - If $\{ U_{\alpha} \}$ contains **countably many sets**, then $\bigcup_{\alpha}U_{\alpha}$ equals $\bigcup_{k = 1}^{\infty} U_{k}$
> - If $\{ U_{\alpha} \}$ contains **$|\mathbb{R}|$ many sets**, then $\bigcup_{\alpha}U_{\alpha}$ equals $\bigcup_{x \in \mathbb{R}} U_{x}$
> - If $\{ U_{\alpha} \}$ contains **$|\mathcal{P}(\mathbb{R})|$ many sets**, then $\bigcup_{\alpha}U_{\alpha}$ equals $\bigcup_{x \in \mathcal{P}(\mathbb{R})} U_{x}$

##### Examples

- Since all open intervals $(a, b)$ are open, all unions and finite intersections of open intervals are open.
	- Note: finite intersections are always of the original form: either a single open interval (a, δ), or the empty set.

#### Countable Disjoint Union of Open Intervals Theorem (Page 153, 5.5)

Every open set $A \subseteq \mathbb{R}$ is a countable union of disjoint open intervals. That is,

$$
A = \bigcup_{k=1}^\infty (a_k, b_k)
$$

for some disjoint open intervals $(a_k, b_k)$.

---

##### Lemma: Maximal Overlapping Intervals Are Equal

Given a real, open set, $A \subseteq \mathbb{R},$ for any pair of elements, $x, y,$ with maximal open intervals, $I_{x}, I_{y} \subseteq A:$

$$
I_{x} = I_{y}\quad \text{or} \quad I_{x} \cap I_{y} = \emptyset
$$

---

###### Proof

1\. Suppose $A \subseteq \mathbb{R}$ is an open set containing elements $x, y.$

2\. Assume the maximal intervals for $x, y,$ $I_{x}, I_{y}$ are not disjoint:

$$
I_{x} \cap I_{y} \ne \emptyset
$$

3\. By definition of **subsets** and **set intersection**, given any $z \in I_{x} \cap I_{y},$

$$
z \in I_{x} \quad \text{and} \quad z \in I_{y} \quad \text{and} \quad z \in A
$$

4\. Define the open set $J = I_{x} \cup I_{y}.$

5\. Since $I_{x}$ and $I_{y}$ are open intervals contained in $A$ and $I_{x} \cap I_{y} \ne \emptyset$, their union, $J,$ is also an open interval in $A.$

6\. Since $x \in I_{x} \subseteq J \subseteq A$, and $J$ is an open interval properly containing $I_{x}$ (because $I_{x} \ne I_{y}$ but $I_{x} \cap I_{y} \ne \emptyset$), this contradicts the maximality of $I_{x}$.

7\. Hence, our assumption that $I_{x} \ne I_{y}$ must be false. Therefore, $I_{x} = I_{y}$.

##### Proof

1\. Suppose $A \subseteq \mathbb{R}$ is an open set.

2\. By definition of **open sets**, since $A$ is open, for every $x \in A$, there exists some $\delta > 0$ such that the interval $(x - \delta, x + \delta) \subseteq A$.

3\. Define $I_{x}$ as the **largest open interval** containing $x,$ such that $I_{x} = (\alpha, \beta),$ where:

$$
\alpha = \inf \{ a : (a, x) \subseteq A \}, \quad
\beta = \sup \{ b : (x, b) \subseteq A \}
$$

4\. By the lemma on **overlapping maximal interval**, for any $x, y \in A$, either:

$$
I_{x} = I_{y}\quad \text{or} \quad I_{x} \cap I_{y} = \emptyset
$$

5\. Since each $x \in A$ lies in some $I_{x} \subseteq A:$

$$
A = \bigcup_{x \in A} I_{x}
$$

6\. By the **density of $\mathbb{Q}$ in $\mathbb{R}$**, every interval $I_{x}$ contains at least one rational number.

7\. By the **countability of $\mathbb{Q},$** since the intervals are disjoint and each contains a distinct rational, the collection $\{I_{x}\}$ is **countable**.

8\. Therefore, $A$ is a countable union of disjoint open intervals.

---

## 5.2 Closed Sets

### Guiding Questions

- How is a closed set defined?
- How do closed sets relate to open sets?
- What role do limit points play in closed sets?

### Definition Of Closed Sets

- A set $A \subseteq \mathbb{R}$ is closed if its complement $\mathbb{R} \setminus A$ is open.
- Equivalent Characterization: A set is closed if it contains all its limit points.

### Examples Of Closed Sets

- The empty set and $\mathbb{R}$ are closed.
- Any closed interval $[a, b]$ is closed.
- A finite set of points is closed.

### Limit Points and Closed Sets

- Definition: A point $x$ is a limit point of $A$ if every $\epsilon$-neighborhood of $x$ contains at least one point of $A$ different from $x$.
- Theorem: A set is closed if and only if it contains all of its limit points.

### Properties Of Closed Sets

- Arbitrary intersections of closed sets are closed.
- Finite unions of closed sets are closed.

---

## 5.3 Open Covers and Compactness

### Guiding Questions

- What does it mean for a set to be covered by open sets?
- How does the concept of compactness relate to open covers?
- Why is compactness an important property?

### Open Covers

- An open cover of a set $A$ is a collection of open sets $\{U_{\alpha}\}$ such that $A \subseteq \bigcup_{\alpha} U_{\alpha}$.
- A finite subcover is a finite subcollection of $\{U_{\alpha}\}$ that still covers $A$.

### Definition Of Compactness

- A set $A$ is compact if every open cover of $A$ has a finite subcover.

### Heine-Borel Theorem

- A subset of $\mathbb{R}$ is compact if and only if it is closed and bounded.
- This means compact sets behave in a well-controlled manner, avoiding the "pathologies" of unbounded or non-closed sets.

### Expanded Heine-Borel Theorem

- A set $A$ is compact if and only if every sequence in $A$ has a subsequence that converges to a point in $A$.

---

## Extracted Proofs (Structured Step-by-Step)

---

### Proof Of Theorem 5.10 (Closed Sets Contain All Their Limit Points)

#### Statement

A set $A$ is closed if and only if it contains all its limit points.

#### Proof Of Forward Direction

1. Assume $A$ is closed but does not contain all its limit points.
2. Let $x$ be a limit point of $A$ that is not in $A$.
3. Then $x$ belongs to $A^c$, which is open, so there exists $\delta$-neighborhood $(x - \delta, x + \delta)$ contained in $A^c$.
4. This contradicts $x$ being a limit point (since there must be points of $A$ arbitrarily close to $x$).
5. Thus, $A$ must contain all its limit points.

#### Proof Of Reverse Direction (Contrapositive)

1. Suppose $A$ is not closed, meaning $A^c$ is not open.
2. Then, there exists $x \in A^c$ such that every $\delta$-neighborhood contains points of $A$.
3. This implies $x$ is a limit point of $A$ but not in $A$, contradicting the given condition.
4. Thus, $A$ is closed.

---

### Proof Of the Heine-Borel Theorem

#### Statement

A subset $S \subseteq \mathbb{R}$ is compact if and only if it is closed and bounded.

#### Proof Of Forward Direction (Compact $\implies$ Closed and Bounded)

1. Boundedness: Suppose $S$ were unbounded, then no finite subcover could exist. Since $S$ is compact, this contradicts compactness.
2. Closedness: Suppose $S$ is not closed, meaning it has a limit point $x \notin S$. Then no finite subcover can cover $x$, contradicting compactness.

#### Proof Of Reverse Direction (Closed and Bounded $\implies$ Compact)

1. Assume $S$ is closed and bounded.
2. Suppose $S$ has an open cover $\mathcal{U}$.
3. Using the completeness of $\mathbb{R}$, we construct a finite subcover step by step.
4. Since a contradiction arises if no finite subcover exists, $S$ must be compact.

---
