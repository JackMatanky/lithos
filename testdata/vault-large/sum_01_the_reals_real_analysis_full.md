---
title: sum_01_the_reals_real_analysis_full
uuid: ea431f9a-2ca0-46b7-abcc-b2e02a7e3f0a
aliases:
  - "Full Summary of Real Analysis: The Reals"
  - "full summary of real analysis: the reals"
  - full_summary_of_real_analysis_the_reals
  - sum_01_the_reals_real_analysis_full
pillar:
  - "[[knowledge_expansion|Knowledge Expansion]]"
category:
  - "[[formal_science|Formal Science]]"
branch:
  - "[[mathematics|Mathematics]]"
field:
  - "[[calculus|Calculus]]"
subject:
topic:
subtopic:
library:
  - "[[01_the_reals_real_analysis|Real Analysis: The Reals]]"
  - "[[cummings_2019_real_analysis|Real Analysis: A Long-form Mathematics Textbook]]"
about: |

url:
status: review
type: summary
file_class: pkm_zettel
date_created: 2024-12-24T13:56
date_modified: 2025-10-05T17:48
tags:
---
# Full Summary of Real Analysis: The Reals

> [!Summary]
>
> - **Resource**: `dv: this.file.frontmatter.library[0]`
>
> - **Source**:: [[Cummings_2019_Real Analysis_01_The Reals.pdf|Real Analysis: The Reals, by Jay Cummings]]
>
> - **Parent**:: [[sum_01_the_reals_real_analysis|Summary of Real Analysis: The Reals]]

---

> [!Quote] Textbook Goal
>
> By studying the infinite, develop a ground-up understanding of the real numbers and functions on the reals. Also, improve one's mathematical maturity; that is, understand mathematical statements and arguments, construct proofs and find counterexamples, and appreciate the intrinsic beauty in the mathematics

## 1.1. Zeno's Paradoxes

Zeno's paradoxes support the position that change is an illusion.
- First Paradox: Achilles and the Tortoise
	- TL;DR: A thought experiment showing that Achilles will never overtake the tortoise due to the infinite sequence of intervals he must traverse.
	- Achilles and a tortoise are in a race and the tortoise is given a head start. To get from point A to point B, each runner must go half the distance between the points. Hence, every time either of the runners advance they have to advance half the distance. Looking just at Achilles, if he has to get from the start to where the tortoise started, he will have to pass an infinite amount of halfway points while the tortoise is progressing. Thereby, he will never catch the tortoise.
	  ![[philosophy_zeno_paradox_runner.webp|300]]
- Second Paradox: The Archer's Arrow
	- Similar to the above paradox, an archer's arrow in motion will never reach its target since it must pass an infinite sequence of halfway points.
	  ![[philosophy_zeno_paradox_arrow.webp|300]]

> [!Important] Key Insight:
>
> These paradoxes highlight the complexities of dealing with infinite sequences and were foundational in the development of limits and real analysis.

## 1.2. Basic [[set_theory|Set Theory]] Definitions

Revisits essential set theory concepts, including:
- Sets, subsets, unions, intersections, and complements.
- Definitions of Cartesian products and power sets.
- Cardinality of sets.

The "box analogy" illustrates these concepts intuitively:
- **Union:** Combine contents of two boxes.
- **Intersection:** Items common to both boxes.
- **Difference:** Items in one box but not the other.

> [!math_problem]
>
> (Page 7) Using the box analogy of sets,
> 1. if $A$ and $B$ are two boxes, describe:
> 	- (1) $A \setminus B$, (2) $P(A)$, (3) $|A|$
> 1. If $A_{1}, A_{2}, \dots, A_{n}$ are all boxes describe:
> 	- sum notation union and sum notation intersection

- the chapter also included basic definitions of functions

- [ ] update Anki function questions

### Definition

#### Set Properties (Page 4, 1.1)

- **Set:** Unordered collection of distinct objects.
	- Set-builder notation: $S = \{\text{elements} \mid \text{conditions used to generate the elements} \}$
- If x is an element of a set S, we write $x \in S$. This is read as "x in S."
- **Empty Set:** The set containing no elements, denoted $\emptyset$.
- **Subset:** If $x \in B$ for every $x \in A$, then $A$ is a subset of $B$; denoted $A \subseteq B$.
- **Set Operations:**
	- **Intersection:** $A ∩ B = \{x \mid x \in A \quad \text{and}\quad x \in B\}$
	- **Union:** $A \cup B = \{x \mid x \in A \quad \text{or}\quad x \notin B\}$
	- **Difference:** $A \setminus B = \{x \mid x \in A \quad \text{and}\quad x \in B\}$
- $A$ and $B$ are disjoint if $A \cap B = \emptyset$
- If $A \subseteq U$ for a universal set $U$ (typically $U = \mathbb{R}$), then the complement of $A$ in $U$ is $A^{c} = U \setminus A$
- **Cartesian product:** $A \times B = \{(a,b) \mid a \in A \quad \text{and}\quad b \in B\}$
- **Power Set:** $P(A) = \{X \mid X \subseteq A\}$.
- If $A_{1}, A_{2}, A_{3}, \dots, A_{n}$ are all sets, then their
	- **Union:** $\bigcup_{i=1}^{n}A_{i} = A_{1} \cup A_{2} \cup \cdots \cup A_{n}$
	- **Intersection:** $\bigcap_{i=1}^{n}A_{i} = A_{1} \cap A_{2} \cap \cdots \cap A_{n}$
- **Cardinality:** The number of elements in the set; denoted $∣A∣$.

#### Function Definition (Page 7, 1.2)

Given a pair of sets $A$ and $B$, suppose that each element $x \in A$ is associated, in some way, to one element of $B$, which we denote $f(x)$. Then $f$ is said to be a [[function_mathematics|function]] from $A$ to $B$. This is sometimes denoted "$f: A \to B$".
- $A$ is called the domain of $f$.
- $B$ is called the codomain $f$.
- The set $\{f(x) \mid x \in A\}$ is called the range of $f$.

A function $f: A \to B$ is
- **injective (or one-to-one)** if $f(a) = f(b)$ implies that $a = b$.
- **surjective (or onto)** if, for every $b \in B$, there exists some $a \in A$ such that $f(a) = b$.
- **bijective** if it is both injective and surjective

## 1.3. What is a Number?

Explores the construction of the number system:
1. **Natural Numbers ($\mathbb{N}$):** Counting numbers.
2. **Integers ($\mathbb{Z}$):** Extends $\mathbb{N}$ with negatives.
3. **Rational Numbers ($\mathbb{Q}$):**
   - Closed under basic arithmetic operations.
   - Have "holes"—not all numbers can be expressed as a ratio.

### Highlights

- Proof of $\sqrt{2}$ being irrational.
- Rational numbers form an **ordered field** but are incomplete.
- review of number systems
- listing of properties and problems of [[rational_numbers|rational numbers]]
	- proof $\sqrt[]{ 2 }$ is not rational.
- the rationals are an ordered field
- definition of a field
	- the natural numbers, $\mathbb{N}$, do not form a field because the number zero is not included; therefore, they cannot satisfy the Identity Law that a field $\mathbb{F}$ includes the elements 0, 1, where $\forall x \in \mathbb{F} (x + 0 = x \quad \text{and}\quad x \cdot 1 = x)$
	- integers do not form a field because they do not satisfy the inverse law that $\forall x \in \mathbb{F} \exists (-x) \in \mathbb{F}[(x + (-x) = 0 \land \exists x^{-1} \in \mathbb{F}(x \neq 0 \to x \cdot x^{-1} = 1)]$ because they do not include fractions

### Key Terms

#### Field (Page 11, 1.5)

A ***field*** is a nonempty set $\mathbb{F}$, along with two binary operations, addition (+) and multiplication (∙), satisfying the following axioms:
- **Commutative Law:** If $a, b \in \mathbb{F}$, then $a + b = b + a$ and $a \cdot b = b \cdot a$.
- **Distributive Law:**. If $a, b, c \in \mathbb{F}$, then $a \cdot (b + c) = a \cdot b + a \cdot c$.
- **Associative Law:** If $a, b, c \in \mathbb{F}$, then $(a + b) + c = a + (b + c)$ and $(a \cdot b) \cdot c = a \cdot (b \cdot c)$.
- **Identity Law:** There are special elements $0,1 \in \mathbb{F}$, where $a + 0 = a$ and $a \cdot 1 = a$ for all $a \in \mathbb{F}$.
- **Inverse Law:** For each $a \in \mathbb{F}$, there is an element $(— a) \in \mathbb{F}$ such that $a + (—a) = 0$. If $a ≠ 0$, then there is also an element $a^{-1} \in \mathbb{F}$ such that $a \cdot a^{-1} = 1$.
- **Field Axioms (Page 11, 1.5):**
  1. **Commutative Law:** $a + b = b + a$ and $a \cdot b = b \cdot a$.
  2. **Distributive Law:** $a \cdot (b + c) = a \cdot b + a \cdot c$.
  3. **Associative Law:** $(a + b) + c = a + (b + c)$ and $(a \cdot b) \cdot c = a \cdot (b \cdot c)$.
  4. **Identity Law:** $a + 0 = a$ and $a \cdot 1 = a$ for all $a$.
  5. **Inverse Law:** $a + (-a) = 0$ and $a \cdot a^{-1} = 1$ for $a \neq 0$.

> [!math_problem]
>
> (Page 11) Prove that the rationals form a field

- [ ] create Anki cards about fields and number sets

## 1.4. Ordered Fields

An ordered field satisfies field axioms and an additional **order axiom**:
- Defines a subset $P$ of positive elements, enabling inequalities.

### Properties

- Absolute values and distance functions are derived from ordered fields.
- Proofs include the **triangle inequality** and its reverse.

### Key Terms

#### Order Axiom (Page 12, 1.7)

There exists a non-empty subset $P \subseteq \mathbb{F}$ such that:
1. $a, b \in P \to a + b \in P \quad \text{and}\quad a \cdot b \in P$.
2. For $a \neq 0$, either $a \in P$ or $-a \in P$ (but not both).

#### Absolute Value (Page 14, 1.10)

If $\mathbb{F}$ is an ordered field (like $\mathbb{R}$), the absolute value function $| \cdot |: \mathbb{F} \to \mathbb{F}$ is defined as

$$
|x| = \begin{cases}
&x, & \text{ for } x \ge 0& \\
&-x, & \text{ for } x < 0&
\end{cases}
$$

#### Distance Function (Page 18, 1.16)

Let $\mathbb{F}$ be an ordered field. Then define the *distance* function as

$$
d(x, y) = |x - y|
$$

- [ ] create Anki cards about ordered fields, order axiom, and distance function

## 1.5. The Completeness Axiom

Defines completeness in terms of suprema and infima:
- **Supremum (Least Upper Bound):** Smallest upper bound of a set.
- **Infimum (Greatest Lower Bound):** Largest lower bound of a set.

Bounded Sets
- A set $A \subseteq \mathbb{R}$ is bounded above if $\exists b \in \mathbb{R}$ such that $x \leq b$ for all $x \in A$.
- It is bounded below if $\exists b \in \mathbb{R}$ such that $x \geq b$ for all $x \in A$.

> [!Important]
>
> The reals ($\mathbb{R}$) are constructed by "completing" the rationals ($\mathbb{Q}$):
>
> - Completeness fills in the "holes" in $\mathbb{Q}$ to form $\mathbb{R}$, such that there are no gaps in the real number line.
> - Completeness allows the real numbers to form a **complete ordered field**, meaning that sequences and series in $\mathbb{R}$ can converge in ways they cannot in $\mathbb{Q}$.

### Key Terms

- **Supremum ($\sup$):** The least upper bound of a set.
- **Infimum ($\inf$):** The greatest lower bound of a set.

#### Bounded Set (Page 20, 1.17)

Let $S$ be an ordered field (like $\mathbb{R}$) and $A \subseteq S$ be nonempty.
1. The set $A$ is *bounded above* if there exists some $b \in S$ such that $x \leq b$ for all $x \in A$; in this case, $b$ is called an *upper bound* of $A$.
2. The *least upper bound* of $A$ — if it exists — is some $b_{0} \in S$ such that
	- (1) $b_{0}$ is an upper bound of $A$, and
	- (2) if $b$ is any other upper bound of A, then $b_{0} \leq b$.
	Such a $b_{0}$ is also called the *supremum* of $A$ and is denoted $\mathrm{sup}(A)$.
3. Likewise, the set $A$ is *bounded below* if there exists some $b \in S$ such that $x \geq b$ for all $x \in A$; in this case, $b$ is called a $lower bound$ of $A$.
4. Again, like above, the *greatest lower bound* of $A$ — if it exists — is some $b_{0} \in S$ such that
	- (1) $b_{0}$ is a lower bound of $A$, and
	- (2) if $b$ is any other lower bound of $A$, then $b_{0} \geq b$.
	Such a $b_{0}$ is also called the *infimum* of $A$ and is denoted $\mathrm{inf}(A)$.
5. If a set is both bounded above and bounded below, then it is simply called ***bounded***.

#### Completeness

##### Statement (Page 21, 1.19)

- Let $S$ be an ordered field (like $\mathbb{R}$).
- Then $S$ has the *least upper bound property* if given any nonempty $A \in S$ where $A$ is bounded above, $A$ has a least upper bound in $S$.
	- In other words, $\sup(A) \in S$ for every such $A$.
- Such a set $S$ is also called **complete**.

##### Axiom (Page 24, 1.21)

If $A \subseteq \mathbb{R}$ is non-empty and bounded above, then there exists a real number $\sup(A) \in \mathbb{R}$ such that:
1. $\sup(A)$ is an upper bound of $A$:
   $\forall x \in A, x \leq \sup(A)$.
2. $\sup(A)$ is the least upper bound:
   $\forall b < \sup(A), b$ is not an upper bound of $A$.

- [ ] create Anki cards from examples in 1.18

## 1.6. Working with $\mathrm{Sup}$s and $\mathrm{Inf}$s

- Supremum and infimum are **unique** if they exist.
- **Analytical definitions**:
  - $\sup(A) = a$ if $a$ is an upper bound and $a - \epsilon$ is not.
  - $\inf(A) = b$ if $b$ is a lower bound and $b + \epsilon$ is not.

### Key Terms

#### Uniqueness of Suprema and Infima (Page 24, 1.22)

If the supremum or infimum of $A \subseteq \mathbb{R}$ exists, then it is unique.

#### **First Proof**

1\. Assume for a contradiction that $\alpha$ and $\beta$ are distinct least upper bounds of $A$, such that $\alpha \neq \beta$.

2\. By definition, a least upper bound is less than or equal to an upper bound, hence:
- If $\alpha$ is a least upper bound and $\beta$ is an upper bound, then $\alpha \leq \beta$.
- If $\beta$ is a least upper bound and $\alpha$ is an upper bound, then $\beta \leq \alpha$.

3\. If $\alpha \leq \beta$ and $\beta \leq \alpha$, this implies $\alpha = \beta$. However, this contradicts our assumption that $\alpha \neq \beta$.

$\therefore$ Suprema are unique.

#### **Second Proof**

Assume for a contradiction that $\alpha$ and $\beta$ are distinct least upper bounds of $A$. In particular, both are upper bounds of $A$, while $\alpha \neq \beta$.

Since $\mathbb{R}$ is an ordered field, either $\alpha < \beta$ or $\beta < \alpha$ (technically, this is a consequence of the "positive subset" that we defined). Without loss of generality, assume that $\alpha < \beta$.

But this contradicts $\beta$ being the least upper bound:
- $\alpha$ is an upper bound of $A$, yet $\beta \not \leq \alpha$, as is required by the definition of the least upper bound.

$\square$

#### Analytic Suprema and Infima (Page 26, 1.24)

1. Let $A \subseteq \mathbb{R}$
2. $\mathrm{sup}(A) = \alpha$ if and only if
	1. $\alpha$ is an upper bound of $A$
	2. Given any $\epsilon > 0$, $\alpha - \epsilon$ is *not* an upper bound of $A$.
		- That is, there is some $x \in A$ for which $x > \alpha - \epsilon$.
3. $\mathrm{inf}(A) = \beta$ if and only if
	1. $\beta$ is a lower bound of $A$
	2. Given any $\epsilon > 0$, $\beta + \epsilon$ is *not* a lower bound of $A$.
		- That is, there is some $x \in A$ for which $x < \beta + \epsilon$.

## 1.7. The Archimedean Principle

The Archimedean Principle ensures that there are no "infinitely large" or "infinitely small" elements in the real numbers, making $\mathbb{R}$ consistent with our intuitive understanding of size.

- **Implications:**
	- This principle formalizes that the set of natural numbers, $\mathbb{N}$, is unbounded in the real numbers.
	- It guarantees that there are no "infinitely small" positive real numbers, as for any small $\epsilon > 0$, a smaller fraction $\frac{1}{n}$ can always be found.

The rational numbers $\mathbb{Q}$ are dense in the real numbers $\mathbb{R}$

- **Implications:**
	1. Rational numbers can approximate any real number to any degree of accuracy.
	2. This property follows from the Archimedean Principle.

### Key Terms

#### The Archimedean Principle (Page 28, Lemma 1.26)

For $a, b \in \mathbb{R}$ with $a > 0$, there exists $n \in \mathbb{N}$ such that $na > b$.
- **Corollary:** For any $\epsilon > 0$, there exists $n \in \mathbb{N}$ such that $\frac{1}{n} < \epsilon$.

##### Proof

**Part I: $n > x$ for $x \in \mathbb{R}$**

1\. **Reformulate the Statement:**
- By dividing $b$ by $a > 0$, we need to show that there exists some $n \in \mathbb{N}$ such that $n > \frac{b}{a}$.
- Let $x = \frac{b}{a}$, which is just some real number. Thus, equivalently, we need to prove that for any real number $x$, there exists $n \in \mathbb{N}$ such that $n > x$.

2\. **Assume for Contradiction:**
- Suppose, for the sake of contradiction, that there is no $n \in \mathbb{N}$ such that $n > x$.
- This means $x$ is an **upper bound** on the set of natural numbers $\mathbb{N}$.

3\. **Supremum of $\mathbb{N}$:**
- Since $\mathbb{N}$ is a subset of $\mathbb{R}$ that is bounded above (by $x$), the **completeness property** of $\mathbb{R}$ implies that the supremum $\alpha = \sup(\mathbb{N})$ exists.

4\. **Contradiction with Supremum Property:**
- By the definition of the supremum, $\alpha$ is the **least upper bound** of $\mathbb{N}$.
- Therefore, $\alpha - 1$ cannot be an upper bound for $\mathbb{N}$.
- This means there exists some integer $m \in \mathbb{N}$ such that:

$$
m > \alpha - 1.
$$

5\. **Adding 1 to Both Sides:**

$$
m + 1 > \alpha.
$$

- But $m + 1 \in \mathbb{N}$, and $\alpha$ is supposed to be an upper bound for $\mathbb{N}$. This is a contradiction.

6\. **Conclusion:**
- The assumption that no $n > x$ exists is false.
- Thus, for any $x \in \mathbb{R}$, there exists $n \in \mathbb{N}$ such that $n > x$.

**Part II: $na > b$ for $a, b > 0$**

1\. **Reformulation:**
- Let $x = \frac{b}{a}$. From the first part of the Archimedean principle, there exists $n \in \mathbb{N}$ such that:

$$
n > \frac{b}{a}.
$$

2\. **Multiply by $a$:**
- Multiplying both sides by $a > 0$ gives:

$$
na > b.
$$

3\. **Conclusion:**
- For any $a, b > 0$, there exists $n \in \mathbb{N}$ such that $na > b$.

**Final Conclusion**

Both parts of the Archimedean principle are proven, showing that:
1. For any real $x \in \mathbb{R}$, $n > x$ for some $n \in \mathbb{N}$.
2. For any $a, b > 0$, $na > b$ for some $n \in \mathbb{N}$.

#### The Density Property (Page 30, 1.29)

Suppose $A$ and $B$ are ordered fields (like $\mathbb{R}$). If for any $x, y \in B$ there exists $a \in A$ such that $x < a < y$, then $A$ is ***dense*** in $B$.

#### Lemma: Density of Integers (Page 30, 1.30)

Let $x, y \in \mathbb{R}$. If $y - x > 1$, then there exists $z \in \mathbb{Z}$ such that $x < z < y$.

For any $a, b \in \mathbb{R}$ with $a < b$, there exists an integer $m \in \mathbb{Z}$ such that $a < m < b$.

##### Proof

1. **Apply the Archimedean Principle:**
   - By the Archimedean property, there exists an integer $N$ such that $N > -a$ and $N > b$.
   - This means the real line is covered by integers.

2. **Find $m$:**
   - There exists an integer $m$ such that $m \in \mathbb{Z}$ and $a < m \leq b$.
   - The integers $\mathbb{Z}$ form a discrete set.

3. **Conclusion:**
   - Since $a < m < b$, this proves the lemma.

#### $\mathbb{Q}$ Is Dense in $\mathbb{R}$ (Page 31, 1.31)

The rational numbers are dense in the real numbers.

Formally, for any $x, y \in \mathbb{R}$ with $x < y$, there exists a rational number $q \in \mathbb{Q}$ such that $x < q < y$.

##### Proof

1\. **Case $x < 0 < y$:**
- If $x < 0 < y$, then $0 \in \mathbb{Q}$ satisfies $x < 0 < y$.
- Thus, the claim is trivially true in this case.

2\. **Reduction to $x > 0$ and $y > 0$:**
- If $x < 0$ or $y < 0$, the argument extends naturally by symmetry.
- Without loss of generality, assume $x > 0$ and $y > 0$.

3\. **Apply the Archimedean Principle:**
- Since $y - x > 0$, the Archimedean property guarantees that there exists some $n \in \mathbb{N}$ such that:

$$
n(y - x) > 1.
$$

- This implies:

$$
ny - nx > 1.
$$

4\. **Use Lemma 1.30 (Density of Integers):**
- By Lemma 1.30, there exists an integer $m$ such that:

$$
nx < m < ny.
$$

5\. **Construct the Rational Number:**
- Define $q = \frac{m}{n}$. Then:

$$
x < \frac{m}{n} < y.
$$

6\. **Conclusion:**
- We have constructed a rational number $q = \frac{m}{n} \in \mathbb{Q}$ such that $x < q < y$, proving the density of $\mathbb{Q}$ in $\mathbb{R}$.

---

## Anki Cards

```plain


{{c1::<b><i>Real numbers</i></b>}}
<br><br>
ⓘ: are unique because they satisfy the completeness axiom;

{{c1::<b><i>Ordered field</i></b>}}
<br><br>
ⓘ: includes addition, multiplication, and a notion of order;

{{c1::<b><i>Supremum</i></b>}}
<br><br>
ⓘ: is the least upper bound of a set;

{{c1::<b><i>Archimedean property</i></b>}}
<br><br>
ⓘ: ensures that for any two real numbers, there exists an integer that can bound them;

{{c1::<b><i>Density of rationals</i></b>}}
<br><br>
ⓘ: implies there are infinitely many rational numbers between any two real numbers;

{{c1::<b><i>Reals</i></b>}}
<br><br>
ⓘ: are constructed by filling the gaps in the rational numbers;

{{c1::<b><i>Completeness axiom</i></b>}}
<br><br>
ⓘ: guarantees that every bounded set has a least upper bound;

{{c1::<b><i>Uniqueness of supremum</i></b>}}
<br><br>
ⓘ: ensures that if there were two suprema, it would contradict the property of least upper bound;

{{c1::<b><i>Absolute value</i></b>}}
<br><br>
ⓘ: measures the distance of a number from zero;

{{c1::<b><i>Triangle inequality</i></b>}}
<br><br>
ⓘ: states that for any real numbers x and y, |x + y| ≤ |x| + |y|;

{{c1::<b><i>Real analysis</i></b>}}
<br><br>
ⓘ: studies the properties of real numbers, including their limits and functions;
```
