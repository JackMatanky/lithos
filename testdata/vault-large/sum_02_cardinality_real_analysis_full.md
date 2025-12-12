---
title: sum_02_cardinality_real_analysis_full
uuid: e51d39bd-904d-429f-b331-2ebc1985905b
aliases:
  - "Full Summary of Real Analysis: Cardinality"
  - "full summary of real analysis: cardinality"
  - full_summary_of_real_analysis_cardinality
  - sum_02_cardinality_real_analysis_full
pillar:
  - "[[knowledge_expansion|Knowledge Expansion]]"
category:
  - "[[formal_science|Formal Science]]"
branch:
  - "[[mathematics|Mathematics]]"
field:
  - "[[calculus|Calculus]]"
  - "[[discrete_mathematics|Discrete Mathematics]]"
  - "[[mathematical_logic|Mathematical Logic]]"
subject:
  - "[[set_theory|Set Theory]]"
topic:
subtopic:
library:
  - "[[02_cardinality_real_analysis|Real Analysis: Cardinality]]"
  - "[[cummings_2019_real_analysis|Real Analysis: A Long-form Mathematics Textbook]]"
about: |

url:
status: develop
type: summary
file_class: pkm_zettel
date_created: 2024-12-29T13:12
date_modified: 2025-10-05T17:48
tags:
---
# Full Summary of Real Analysis: Cardinality

> [!Summary]
>
> - **Resource**: `dv: this.file.frontmatter.library[0]`
>
> - **Source**:: [[Cummings_2019_Real Analysis_02_Cardinality.pdf|Real Analysis: Cardinality, by Jay Cummings]]
>
> - **Parent**:: [[sum_02_cardinality_real_analysis|Summary of Real Analysis: Cardinality]]

---

## **2.1 Bijections and Cardinality**

1. What is the bijection principle, and how does it help us compare the sizes of sets?
	- The bijection principle states that two sets are equinumerous if there exist a bijection between them.

### Key Terms

#### The Bijection Principle (Page 43, 2.1)

Two sets have the same size if and only if there is a bijection between them.

## **2.2 Counting Infinities**

### 2.2.1. **Hilbert's Hotel**

1. How does Hilbert's Hotel illustrate the concept of infinite sets and bijections?
2. How can an infinite hotel with no vacancies accommodate new guests?
	- By creating more space.
3. What does this tell us about the "size" of infinite sets?
	- Size is not the same as inclusion. The set of natural numbers, $\mathbb{N}$, is a proper superset of the set of even natural numbers, $2 \mathbb{N}$. However, because a bijection of $f \colon \mathbb{N} \to 2 \mathbb{N}$, with a mapping $n \mapsto 2n$, $|\mathbb{N}| = |2 \mathbb{N}|$. (The function is given by $f(x) = 2x$)

### 2.2.2. Specific Sets

1. Why do some infinite sets have the same size as their proper subsets?
	- Some infinite sets have the same size as their proper sets because size is a question of mapping and not inclusion. Intuitively, if countably infinite sets do not have a lower and/or higher boundary, then the mapping between sets can always continue.
2. How can we establish a bijection between the integers ($\mathbb{Z}$) and the rational numbers ($\mathbb{Q}$)?

### 2.2.3. Unprovable Statements

1. What is Cantor's diagonalization argument, and how does it show that the real numbers ($\mathbb{R}$) are uncountable?
2. How does the continuum hypothesis relate to the sizes of infinities?

### Key Terms

#### Set Cardinality and Relation (Page 45, 2.4)

Let $S$ and $T$ be sets. Then:
- $|S| = |T|$ if and only if there is a **bijection** from $S$ to $T$.
- $|S| \le |T|$ if and only if there is an **injection** from $S$ to $T$.
- $|S| \ge |T|$ if and only if there is a **surjection** from $S$ to $T$.

#### Theorem: Equinumerosity of Integers and Rational Numbers (Page 47, 2.8)

There are the same number of integers as rational numbers:

$$
|\mathbb{Z}| = |\mathbb{Q}|
$$

##### Proof

The integers ($\mathbb{Z}$) and rational numbers ($\mathbb{Q}$) are equinumerous if there exists a bijection $f: \mathbb{Z} \to \mathbb{Q}$. To establish this, we show:

1. $\mathbb{Q}$ is countable by enumerating its elements.
2. Construct an explicit bijection between $\mathbb{Z}$ and $\mathbb{Q}$.

###### Step 1: Proving $\mathbb{Q}$ is Countable

1. **Representation of Rational Numbers:** Any rational number can be expressed as a fraction, $\frac{p}{q}$, where:
	- $p \in \mathbb{Z}$ (numerator),
	- $q \in \mathbb{N}$ (positive denominator).
2. **Pairing $p$ and $q$:** Consider the set of pairs $(p, q) \in \mathbb{Z} \times \mathbb{N}$, in which each rational number is included.
3. **Organizing into a Grid** (see below): Arrange $(p, q)$ pairs in a 2D grid:
	- Rows indexed by $p$ (numerator),
	- Columns indexed by $q$ (denominator).
4. **Diagonal Enumeration:** Traverse the grid diagonally to list each rational number exactly once and remove duplicates (e.g., $\frac{2}{4} = \frac{1}{2}$) to get a unique sequence:
	- $\mathbb{Q} = \{0, 1, -1, \frac{1}{2}, -\frac{1}{2}, \frac{2}{1}, -\frac{2}{1}, \dots\}.$
5. This enumeration shows $\mathbb{Q}$ is countable.

$$
\begin{array}{c|cccc}
p \backslash q & 1 & 2 & 3 & 4 \\
\hline
0 & (0, 1) & (0, 2) & (0, 3) & (0, 4) \\
\hline
1 & (1, 1) & (1, 2) & (1, 3) & (1, 4) \\
\hline
-1 & (-1, 1) & (-1, 2) & (-1, 3) & (-1, 4) \\
\hline
, 2 & (2, 1) & (2, 2) & (2, 3) & (2, 4) \\
\hline
-2 & (-2, 1) & (-2, 2) & (-2, 3) & (-2, 4)
\end{array}
$$

###### Step 2: Constructing the Bijection

1. **Enumerate $\mathbb{Q}$:** Using the diagonal enumeration, list $\mathbb{Q}$ as $q_1, q_2, q_3, \dots$, where $q_i \in \mathbb{Q}$.
2. **Map $\mathbb{Z}$ To $\mathbb{Q}$:** Use a zigzag enumeration of $\mathbb{Z}$ (e.g., $0, 1, -1, 2, -2, \dots$) to assign integers to rational numbers: $f(n) = q_{\text{index of } n}.$

This bijection ensures:
- Every $z \in \mathbb{Z}$ maps uniquely to some $q \in \mathbb{Q}$,
- Every $q \in \mathbb{Q}$ is paired with exactly one $z \in \mathbb{Z}$.

###### Conclusion

The bijection $f: \mathbb{Z} \to \mathbb{Q}$ proves the equinumerosity of $\mathbb{Z}$ and $\mathbb{Q}$. Both sets are countable, and there exists an explicit mapping between their elements. Thus,

$$
|\mathbb{Z}| = |\mathbb{Q}|
$$

#### Theorem: There Are More Real Numbers $\mathbb{R}$ Than Natural Numbers $\mathbb{N}$ (Page 50, 2.9)

Cantor's diagonal proof demonstrates that the set of real numbers ($\mathbb{R}$) is uncountable, meaning its cardinality is strictly greater than the set of natural numbers ($\mathbb{N}$).

We show this by contradiction:
1. Assume $\mathbb{R}$ is countable.
2. Construct a real number not in any enumeration of $\mathbb{R}$, contradicting the assumption.

##### Proof: Cantor's Diagonal Argument

###### Step 1: Assume $\mathbb{R}$ is Countable

1\. Assume, for contradiction, that $\mathbb{R}$ is countable.

2\. If $\mathbb{R}$ were countable, we could list all real numbers in the interval $[0, 1)$ as:

$$
r_1, r_2, r_3, \dots
$$

3\. Each $r_i$ can be written in decimal form:

$$
r_i = 0.a_{i1}a_{i2}a_{i3}\dots
$$

where $a_{ij}$ represents the $j$-th digit of the $i$-th number.

4\. Organizing the list gives:

$$
\begin{array}{cccccc}
r_{1} & \leftrightarrow & 0.a_{11} & a_{12} & a_{13} & a_{14} & \dots \\
r_{2} & \leftrightarrow & 0.a_{21} & a_{22} & a_{23} & a_{24} & \dots \\
r_{3} & \leftrightarrow & 0.a_{31} & a_{32} & a_{33} & a_{34} & \dots \\
r_{4} & \leftrightarrow & 0.a_{41} & a_{42} & a_{43} & a_{44} & \dots \\
\vdots & \vdots & \vdots & \vdots & \vdots & \vdots & \vdots \end{array}
$$

###### Step 2: Construct a Real Number Not in the List

5\. Using this enumeration, construct a new real number $r$ such that:

$$
r \notin \{r_1, r_2, r_3, \dots\}.
$$

6\. Define $r = 0.b_1b_2b_3 \dots$, where:

$$
b_i =
\begin{cases}
1 & \text{if } a_{ii} \neq 1, \\
2 & \text{if } a_{ii} = 1.
\end{cases}
$$

- Here, $b_i$ is chosen specifically to differ from the $i$-th digit of $r_i$, ensuring that:
 - The 1st digit $b_1 \neq a_{11}$, so $r \neq r_1$,
 - The 2nd digit $b_2 \neq a_{22}$, so $r \neq r_2$,
 - The 3rd digit $b_3 \neq a_{33}$, so $r \neq r_3$,
 - And so on for all $i$.

7\. Thus, $r$ is constructed to differ from every $r_i$ in at least one digit, guaranteeing that $r \notin \{r_1, r_2, \dots\}$.

###### Conclusion

The assumption that $\mathbb{R}$ is countable leads to a contradiction. Therefore, $\mathbb{R}$ is uncountable, and its cardinality is strictly greater than that of $\mathbb{N}$.

$$
|\mathbb{R}| > |\mathbb{N}|
$$

#### Countable and Uncountable Infinities (Page 52, 2.10)

If $S$ is an infinite set, then $S$ is ***countable*** if $|S| = |\mathbb{N}|$; otherwise $S$ is ***uncountable***.

#### Theorem: Sizes of Infinity (Page 53, 2.11)

There are different sizes of infinity with countable infinity being the smallest. Moreover, $\mathbb{N}$, $\mathbb{Z}$, and $\mathbb{Q}$ are countable while $\mathbb{R}$ is uncountable.

#### The (Unprovable) Continuum Hypothesis (Page 54, 2.12)

There is no set whose cardinality is strictly between that of the naturals and the reals.

$$
|\mathbb{N}| \not< |S| \not< |\mathbb{R}|
$$

## **2.3 How Many Infinities Are There?**

1. What is the power set theorem, and how does it show there are infinitely many infinities?
2. How does the corollary to the power set theorem demonstrate the existence of distinct infinite cardinalities?
3. Why is it impossible to create a "set of all infinities"?

### Key Terms

#### Theorem: The Size of a Power Set is Greater Than the Original Set (Page 56, 2.13)

For any set $S$, the cardinality of its power set $\mathcal{P}(S)$ is strictly greater than the cardinality of $S$:

$$
|\mathcal{P}(S)| > |S|.
$$

##### Proof

###### Step 1: Assume there Exists a Surjection

Assume, for contradiction, $|S| \geq |\mathcal{P}|$, which is to say that there exists a surjection $f: S \to \mathcal{P}(S)$. This means:
- Every subset of $S$ is in the image of $f$, i.e., for every $D \in \mathcal{P}(S)$, there exists some $x \in S$ such that $f(x) = D$.

###### Step 2: Define the Set $D$ in $\mathcal{P}(S)$

Using $f$, construct the following subset of $S$:

$$
D = \{x \in S \mid x \notin f(x)\}.
$$

- $D$ is the set of all elements in $S$ that are **not members** of the subset they are mapped to by $f$.

###### Step 3: Analyze $D$

Since $D \in \mathcal{P}(S)$, and $f$ is assumed to be surjective, there must exist some $z \in S$ such that:

$$
f(z) = D.
$$

We now analyze whether $z \in D$ or $z \notin D$.

1. If $z \in D$:
   - By the definition of $D$, $z \notin f(z)$.
   - But $f(z) = D$, so $z \notin D$, which is a contradiction.

2. If $z \notin D$:
   - By the definition of $D$, $z \in f(z)$.
   - But $f(z) = D$, so $z \in D$, which is a contradiction.

###### Step 4: Conclude that $f$ is not Surjective

In both cases, assuming $f(z) = D$ leads to a logical contradiction. Therefore, no such $z \in S$ exists such that $f(z) = D$. This implies that $D \notin \text{Im}(f)$, contradicting the assumption that $f$ is surjective.

###### Conclusion

Since no surjection $f: S \to \mathcal{P}(S)$ can exist, the cardinality of $\mathcal{P}(S)$ must be strictly greater than the cardinality of $S$:

$$
|\mathcal{P}(S)| > |S|.
$$

#### Corollary: Existence of Infinite Infinities (Page 57, 2.14)

Following the theorem that $|S| < |\mathcal{P}(S)|$, it follows that for an infinite set $I$,

$$
|I| < |\mathcal{P}(I)| < |\mathcal{P}(\mathcal{P}(I))| < |\mathcal{P}(\mathcal{P}(\mathcal{P}(I)))| < \dots
$$
