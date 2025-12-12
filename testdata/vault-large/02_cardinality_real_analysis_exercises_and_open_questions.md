---
title: 02_cardinality_real_analysis_exercises_and_open_questions
uuid: dc9efe3a-e169-43ed-b674-5f33d575a563
aliases:
  - "Real Analysis: Cardinality, Exercises and Open Questions"
  - "Cardinality: Exercises and Open Questions"
  - "2. Cardinality: Exercises and Open Questions"
  - cardinality exercises and open questions
  - cardinality_exercises_and_open_questions
  - real_analysis_cardinality_exercises_and_open_questions
  - 02_cardinality_real_analysis_exercises_and_open_questions
main_title: Cardinality
subtitle: Exercises and Open Questions
author:
  - "[[cummings_jay|Jay Cummings]]"
editor:
translator:
year_published: 2019
publisher:
page_start: 59
page_end: 63
doi:
url: https://longformmath.com/analysis-home
library:
  - "[[02_cardinality_real_analysis|2. Cardinality]]"
  - "[[cummings_2019_real_analysis|Real Analysis: A Long-form Mathematics Textbook]]"
cssclasses:
status: done
type: book_chapter
file_class: lib_book_chapter
date_created: 2024-12-22T19:42
date_modified: 2025-10-05T17:48
tags:
---
# 2. Cardinality: Exercises and Open Questions

> [!book_chapter] Book Chapter Details
>
> - **Author**: `dv: this.file.frontmatter.author`
> - **Chapter**: `dv: this.file.frontmatter.aliases[0]`
> - **Book**: `dv: this.file.frontmatter.library[0]`
> - **Publisher**: `dv: this.file.frontmatter.publisher`
> - **Date Published**: `dv: this.file.frontmatter.year_published`
> - **Pages**: `dv: this.file.frontmatter.page_start + " - " + this.file.frontmatter.page_end`
>
> - **Completed**:: [[2025-01-07]]

---

<!-- Insert chapter content here -->

![[Cummings_2019_Real Analysis_02_Cardinality.pdf]]

---

## Exercise 2.1

1. **List all the elements of** $\mathcal{P}(\{a, b, c\})$.
2. **Determine a formula for the number of elements in the power set of an $n$-element set.**

---

## Exercise 2.2

**Prove that** $\{e^n: n \in \mathbb{N}\} = |\mathbb{N}|$.

---

## Exercise 2.3

The following pairs of sets have the same size, and so there exists a bijection between them. Write down an explicit bijection in each case. You do not need to prove your answers.

(a) $(0, \infty)$ and $(1, \infty)$
(b) $(0, \infty)$ and $(-\infty, 3)$
(c) $(0, \infty)$ and $(0, 1)$
(d) $\mathbb{R}$ and $(0, \infty)$
(e) $\mathbb{R}$ and $(0, 1)$
(f) $\mathbb{Z}$ and $\{\dots, \frac{1}{8}, \frac{1}{4}, \frac{1}{2}, 1, 2, 4, 8, \dots\}$
(g) $\{0, 1\} \times \mathbb{N}$ and $\mathbb{N}$
(h) $[0, 1]$ and $(0, 1)$

---

## Exercise 2.4

This problem shows that "equinumerosity is an equivalence relation." (This justifies the notation $|A| = |B|$.) Let $A, B,$ and $C$ be sets. For this problem only, we'll write $A \sim B$ to mean that $A$ and $B$ are equinumerous, meaning that there is a bijection $A \to B$.

### 2.4.1

Show that $A \sim A$.

#### Solution

### 2.4.2

Show that if $A \sim B$, then $B \sim A$.

#### Solution

### 2.4.2

Show that if $A \sim B$ and $B \sim C$, then $A \sim C$.

#### Solution

---

## Exercise 2.5

(a) Prove that if $A$ and $B$ are countable sets, then $A \cup B$ is also a countable set.

(b) Prove that if $A_n$ is a countable set for each $n \in \mathbb{N}$, then the set $\bigcup_{n=1}^\infty A_n$ is also countable.

---

## Exercise 2.6

Show that $|\mathbb{N}| = |\mathbb{Z}|$ by finding an explicit bijection from $\mathbb{N}$ to $\mathbb{Z}$. You do not need to prove your bijection works.

---

## Exercise 2.7

Let $A, B \subseteq \mathbb{R}$. We define:

$$
A \cdot B = \{a \cdot b : a \in A \text{ and } b \in B\}.
$$

(a) Give an example of sets $A_1$ and $B_1$ where $|A_1 \cdot B_1| < \max(|A_1|, |B_1|)$.
(b) Give an example of sets $A_2$ and $B_2$ where $|A_2 \cdot B_2| = \max(|A_2|, |B_2|)$.
(c) Give an example of sets $A_3$ and $B_3$ where $|A_3 \cdot B_3| = \max(|A_3|, |B_3|)$.

For which of the above does there exist an example where one or both of the sets are infinite?

---

## Exercise 2.8

(a) Describe a way to partition the set $\mathbb{N}$ into 6 subsets, each containing infinitely many elements.

(b) Describe a way to partition the set $\mathbb{N}$ into infinitely many subsets, each containing infinitely many elements.

---

## Exercise 2.9

Is $|\mathbb{Z} \times \mathbb{N}|$ countable or uncountable?

---

## Exercise 2.10

Let $S$ be the set of sequences $(a_n)$, where, for each $n$, $a_n \in \{0, 1\}$.
Is $S$ countable or uncountable?

---

## Exercise 2.11

Suppose that $X$ is a nonempty set. Prove that the following three assertions are equivalent:

(a) $X$ is finite or countably infinite.
(b) There is a one-to-one function $f: X \to \mathbb{N}$.
(c) There is an onto function $g: \mathbb{N} \to X$.

---

## Exercise 2.12

### 2.12.1

Give an example of a collection of countably many disjoint open intervals, or prove that this does not exist.

#### Solution

> [!Note] Explanation
>
> We can build a countable collection of disjoint open intervals by carefully placing small intervals around rational numbers in the interval $(0, 1)$.
>
> Since the rational numbers in $(0, 1)$ are countable, we can enumerate them and assign to each a small open interval centered at that rational number — but small enough that the intervals don't overlap. For example, we can choose intervals of length $2^{-n}$ for the $n$th rational number.

##### Proof

1\. Let $\{q_n\}_{n \in \mathbb{N}}$ be an enumeration of the rational numbers in the open interval $(0, 1)$.

2\. For each $n \in \mathbb{N}$, define an open interval:

$$
I_n = \left(q_n - 2^{-n}, \, q_n + 2^{-n} \right).
$$

3\. Define inductively a sequence of collections $\{\mathcal{A}_n\}_{n \in \mathbb{N}}$ of **disjoint open intervals**, where each $\mathcal{A}_n \subseteq \{I_{0}, I_{1}, \dots, I_{n}\}$:

$$
\mathcal{A}_n =
\begin{cases}
\{I_0\}, & \text{if } n = 0, \\\\
\mathcal{A}_{n-1} \cup \left\{ I_n \,\middle|\, I_n \cap \bigcup \mathcal{A}_{n-1} = \emptyset \right\}, & \text{if } n \geq 1.
\end{cases}
$$

4\. Define the final collection:

$$
\mathcal{A} = \bigcup_{n=0}^{\infty} \mathcal{A}_n.
$$

5\. By construction, the intervals in $\mathcal{A}$ are pairwise disjoint, since each $I_{n}$ is included only if it does not intersect any interval already in $\mathcal{A}_{n-1}.$

6\. Thus, $\mathcal{A}$ is a **countable collection of disjoint open intervals**, such that each interval is centered at a rational number, $q_n \in (0,1),$ has a length $2^{-n+1},$ for some $n \in \mathbb{N},$ and is of the form:

$$
I_n = \left(q_n - 2^{-n},\, q_n + 2^{-n}\right).
$$

### 2.12.2

Give an example of a collection of uncountably many disjoint open intervals, or prove that this does not exist.

#### Solution

Such a collection **does not exist**.

1\. Let $\mathcal{A}$ be a collection of **pairwise disjoint open intervals** in $\mathbb{R}$.

2\. Each open interval in $\mathcal{A}$ contains at least one rational number (by the **density of $\mathbb{Q}$ in $\mathbb{R}$**).

3\. Since the intervals in $\mathcal{A}$ are disjoint, no two intervals can share a rational number.

4\. Thus, we can define an injective function:

$$
f: \mathcal{A} \to \mathbb{Q}, \quad f((a, b)) = q \in (a, b) \cap \mathbb{Q}.
$$

5\. This function is well-defined because each open interval contains at least one rational, and is injective because the intervals are disjoint.

6\. Since $\mathbb{Q}$ is **countable**, and there exists an injection from $\mathcal{A}$ to $\mathbb{Q}$, it follows that $\mathcal{A}$ is **countable**.

7\. Thus, **there does not exist** an uncountable collection of disjoint open intervals in $\mathbb{R}$.

---

## Exercise 2.13

Show that there are uncountably many irrational numbers.

---

## Exercise 2.14

Prove that $\mathbb{N} \times \mathbb{N}$ is countably infinite by showing that the function $f: \mathbb{N} \times \mathbb{N} \to \mathbb{N}$ defined by $f(m, n) = 2^m \cdot (2n - 1)$ is a bijection.

---

## Exercise 2.15

Let $\mathcal{F}$ be the collection of all functions $f: \mathbb{R} \to \mathbb{R}$. Prove that $\mathcal{F}$ is uncountable.

---

## Exercise 2.16

Show that the smallest infinity is $|\mathbb{N}|$. That is, show that if $A \subseteq \mathbb{N}$, then either $A$ is finite or $|A| = |\mathbb{N}|$.

---

## Exercise 2.17

Prove that the set of all finite subsets of $\mathbb{N}$ is countable.

---

## Exercise 2.18

Is the subset of rational numbers:

$$
\left\{\frac{m}{n} : m, n \in \mathbb{Z} \text{ and } 1 \leq n \leq 10 \right\}
$$

dense in $\mathbb{R}$?

---

## Exercise 2.19

Let $A$ be the set of polynomials with rational coefficients. Prove that $|A|$ is countable.

---

## Exercise 2.20

Show that $|\mathcal{P}(\mathbb{N})| = |\mathbb{R}|$ by finding an explicit bijection from $\mathcal{P}(\mathbb{N})$ to $\mathbb{R}$. You do not need to prove your bijection works.

---

## Exercise 2.21

Prove that the set of points on the unit circle in $\mathbb{R}^2$ (that is, $\{(x, y): x^2 + y^2 = 1\}$) is uncountable.

---

## Exercise 2.22

A real number $x$ is said to be **algebraic** (over the rationals) if it satisfies some polynomial equation (of positive degree):

$$
a_nx^n + a_{n-1}x^{n-1} + a_{n-2}x^{n-2} + \dots + a_1x + a_0 = 0
$$

where each $a_i \in \mathbb{Q}$. If a real number is not algebraic, then it is **transcendental**.

(a) Prove that there are countably many algebraic numbers.
(You may use the fundamental theorem of algebra, which says that a polynomial with degree $n$ has at most $n$ real roots.)

(b) Prove that there are uncountably many transcendental numbers.

---

## Open Questions

For subsets $X$ and $Y$ of $\mathbb{N}$, we say that $X$ **splits** $Y$ if both $Y \cap X$ and $Y \setminus X$ are infinite. A family $F$ of subsets of $\mathbb{N}$ is **unsplittable** if no single subset of $\mathbb{N}$ splits every set in $F$. Moreover, $F$ is **$\sigma$-unsplittable** if even countably many sets don't suffice to split every member of $F$. (As an exercise, show that no countable family can be unsplittable.)

---

### Question 1

**Must the least size of an unsplittable family equal the least size of a $\sigma$-unsplittable family?**

> [!Note]
>
> Notice that every unsplittable family is an infinite subset of the power set $\mathcal{P}(\mathbb{N})$, and under the continuum hypothesis, all such subsets are said to have size $|\mathbb{R}|$. Therefore:
>
> - A **positive** answer to Question 1 would directly prove the result under the continuum hypothesis.
> - A **negative** answer would indicate a result in a universe where the continuum hypothesis fails.
>
> Question 1 is one of many questions about what you might call **almost-countable cardinals**, which are typically called **cardinal characteristics of the continuum**.

---

### Axiom of Choice and Related Questions

The next two open questions involve the **Axiom of Choice**, which asserts:

> For every family $F$ of nonempty sets, there is a function $f$ assigning to each $x \in X$ a member of $x$ (that is, $f(x) \in x$).

At the beginning of the 20th century, the Axiom of Choice inspired some controversy. Today, it is mostly accepted, although its use still raises logical and philosophical questions.

> [!Definition]
>
> - For sets $X$ and $Y$, write $|X| \leq |Y|$ if there is a one-to-one function from $X$ to $Y$.
> - Write $|X| \leq^* |Y|$ if there is an onto function from $Y$ to $X$.
>
> In the presence of the Axiom of Choice, the orderings $\leq$ and $\leq^*$ are equivalent.

---

#### Question 2

**Suppose that for all sets $X$ and $Y$, the implication $|X| \leq^* |Y| \implies |Y| \leq |X|$ holds. Must the Axiom of Choice be true?**

---

#### Question 3

**Assuming the Axiom of Choice, there is no infinite sequence of sets strictly decreasing in cardinality:**

$$
|X_1| > |X_2| > |X_3| > \cdots
$$

(This is not obvious!) Now here's the open question:
**Does the non-existence of such a sequence imply the Axiom of Choice?**
