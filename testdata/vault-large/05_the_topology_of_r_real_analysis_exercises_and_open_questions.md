---
title: 05_the_topology_of_r_real_analysis_exercises_and_open_questions
uuid: 35231988-cba5-4ddf-bfa7-18b0ff0be27d
aliases:
  - "Real Analysis: The Topology of R, Exercises and Open Questions"
  - "The Topology of R: Exercises and Open Questions"
  - "5. The Topology of R: Exercises and Open Questions"
  - the topology of r exercises and open questions
  - the_topology_of_r_exercises_and_open_questions
  - real_analysis_the_topology_of_r_exercises_and_open_questions
  - 05_the_topology_of_r_real_analysis_exercises_and_open_questions
main_title: The Topology of R
subtitle: Exercises and Open Questions
author:
  - "[[cummings_jay|Jay Cummings]]"
editor:
translator:
year_published: 2019
publisher:
page_start: 163
page_end: 169
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
# 5. The Topology of R: Exercises and Open Questions

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

![[Cummings_2019_Real Analysis_05_The Topology of R.pdf]]

---

## Exercise 5.1

For each of the following, determine whether the set is open, whether it is closed, and whether it is compact. (It might be more than one, or none of these.) You do not need to prove your answers.

**(a) $\mathbb{Z}$**

- **Open?** No. Consider the point $0 \in \mathbb{Z}$. For any $\delta > 0$, the open interval $(0 - \delta, 0 + \delta)$ will contain numbers that are not integers (e.g., if $\delta = 0.5$, the interval is $(-0.5, 0.5)$, which contains $0.25 \notin \mathbb{Z}$). Therefore, there is no $\delta$-neighborhood of $0$ entirely contained in $\mathbb{Z}$.
- **Closed?** Yes. A set is closed if its complement is open. The complement of $\mathbb{Z}$ in $\mathbb{R}$ is $\mathbb{R} \setminus \mathbb{Z} = \bigcup_{n \in \mathbb{Z}} (n, n+1)$. Each interval $(n, n+1)$ is open, and the union of any collection of open sets is open by Proposition 5.3(i). Thus, $\mathbb{R} \setminus \mathbb{Z}$ is open, and so $\mathbb{Z}$ is closed.
- **Compact?** No. A compact set in $\mathbb{R}$ must be closed and bounded by the Heine-Borel theorem (Theorem 5.19). While $\mathbb{Z}$ is closed, it is not bounded (it extends infinitely in both positive and negative directions).

**(b) ${1, \frac{1}{2}, \frac{1}{3}, \frac{1}{4}, …} \cup {0}$**

- **Open?** No. Consider the point $0$ in the set. Any $\delta$-neighborhood of $0$, $(-\delta, \delta)$, will contain positive numbers. For sufficiently small $\delta$, this neighborhood will not contain any of the terms in the sequence ${1/n}$ (other than possibly $n=1$ if $\delta > 1$, but we need the entire neighborhood to be in the set). For example, if we take $\delta = 0.1$, then the interval $(-0.1, 0.1)$ only contains $0$ from our set, not an entire open interval around $0$ within the set.
- **Closed?** Yes. Let $A = {1, \frac{1}{2}, \frac{1}{3}, …} \cup {0}$. We need to consider its limit points. The sequence ${1/n}$ converges to $0$, and $0$ is in the set. Any other point in the set $\frac{1}{k}$ (for some $k \in \mathbb{N}$) has a neighborhood that contains no other points of the set (except itself), so these are not limit points of the set. Thus, the only limit point is $0$, and since $0$ is in $A$, the set $A$ contains all of its limit points, and therefore $A$ is closed by Theorem 5.10.
- **Compact?** Yes. By the Heine-Borel theorem (Theorem 5.19), a subset of $\mathbb{R}$ is compact if and only if it is closed and bounded. We've established that the set is closed. It is also bounded, as all its elements lie in the interval $()$. Therefore, the set is compact.

**(c) $\mathbb{R}$**

- **Open?** Yes. For any $x \in \mathbb{R}$, we can choose any $\delta > 0$ (for example, $\delta = 17$ as mentioned in Example 5.2(i)), and the $\delta$-neighborhood $(x - \delta, x + \delta)$ will always be a subset of $\mathbb{R}$.
- **Closed?** Yes. The complement of $\mathbb{R}$ is the empty set $\emptyset$, which is open (Example 5.2(ii)). Therefore, $\mathbb{R}$ is closed.
- **Compact?** No. $\mathbb{R}$ is not bounded, as it extends infinitely. By the Heine-Borel theorem (Theorem 5.19), a compact set in $\mathbb{R}$ must be bounded.

**(d) $(0, 1) \cup$**

- **Open?** No. Consider the point $3$ in the set. Any $\delta$-neighborhood of $3$, $(3 - \delta, 3 + \delta)$, for any $\delta > 0$, will contain points less than $3$ (e.g., $3 - \delta/2$). If $\delta/2 < 3$, then $3 - \delta/2$ might be in $(0, 1)$ if $\delta > 4$, or outside the set entirely if $0 < \delta \le 6$. More importantly, if $\delta > 0$, then $(3 - \delta, 3 + \delta)$ will contain points in $(3 - \delta, 3)$, which are not in the set $(0, 1) \cup$ if $\delta$ is small enough (e.g., $\delta < 1$). Thus, no $\delta$-neighborhood of $3$ is entirely contained in the set.
- **Closed?** No. The complement of the set is $(-\infty, 0] \cup (1, 3) \cup (4, \infty)$. This complement is not open because, for instance, $0$ is in the complement, but any $\delta$-neighborhood of $0$, $(-\delta, \delta)$, contains positive numbers that are in $(0, 1)$ and thus not in the complement. Therefore, the original set is not closed. Alternatively, $(0, 1)$ is open and $()$ is closed. Their union is neither necessarily open nor closed. The point $1$ is a limit point of $(0, 1)$ but not in the set, and the point $3$ is a limit point of $(0, 1) \cup$ but any neighborhood around $3$ contains points not in the set.
- **Compact?** No. The set is not closed, as shown above. By the Heine-Borel theorem (Theorem 5.19), a compact set must be closed.

**(e) $\mathbb{Q}$**

- **Open?** No. Consider any rational number $q \in \mathbb{Q}$. For any $\delta > 0$, the open interval $(q - \delta, q + \delta)$ will always contain irrational numbers (by the density of irrationals in $\mathbb{R}$). Thus, no $\delta$-neighborhood of $q$ is entirely contained in $\mathbb{Q}$.
- **Closed?** No. The complement of $\mathbb{Q}$ is the set of irrational numbers $\mathbb{R} \setminus \mathbb{Q}$. If $\mathbb{Q}$ were closed, then $\mathbb{R} \setminus \mathbb{Q}$ would be open. However, for any irrational number $r$, any $\delta$-neighborhood $(r - \delta, r + \delta)$ will always contain rational numbers (by the density of rationals in $\mathbb{R}$), so $\mathbb{R} \setminus \mathbb{Q}$ is not open. Therefore, $\mathbb{Q}$ is not closed.
- **Compact?** No. $\mathbb{Q}$ is not closed, as shown above. By the Heine-Borel theorem (Theorem 5.19), a compact set must be closed. Also, $\mathbb{Q}$ is not bounded.

**(f) ${17}$**

- **Open?** No. For the single point $17$, any $\delta$-neighborhood $(17 - \delta, 17 + \delta)$ with $\delta > 0$ will contain points other than $17$. For this neighborhood to be a subset of ${17}$, it would require $(17 - \delta, 17 + \delta) = {17}$, which is impossible for any $\delta > 0$.
- **Closed?** Yes. The complement is $\mathbb{R} \setminus {17} = (-\infty, 17) \cup (17, \infty)$. Both $(-\infty, 17)$ and $(17, \infty)$ are open intervals, and their union is an open set by Proposition 5.3(i). Therefore, ${17}$ is closed.
- **Compact?** Yes. The set ${17}$ is closed (as shown above) and bounded (all elements are within any bounded interval containing $17$, e.g., $()$). By the Heine-Borel theorem (Theorem 5.19), ${17}$ is compact.

## Exercise 5.2

Determine the set of limits points for each set in Exercise 5.1.

**(a) $\mathbb{Z}$**

A point $x$ is a limit point of a set $A$ if every $\epsilon$-neighborhood of $x$ intersects $A$ at some point other than $x$. For any integer $n \in \mathbb{Z}$, if we take $\epsilon = 0.5$, the neighborhood $(n - 0.5, n + 0.5)$ contains no other integers. Thus, no integer is a limit point of $\mathbb{Z}$. What about a non-integer $y \in \mathbb{R} \setminus \mathbb{Z}$? We can find an $\epsilon > 0$ small enough such that the neighborhood $(y - \epsilon, y + \epsilon)$ contains no integers. Therefore, the set of limit points of $\mathbb{Z}$ is $\emptyset$.

**(b) ${1, \frac{1}{2}, \frac{1}{3}, \frac{1}{4}, …} \cup {0}$**

Let $A = {1, \frac{1}{2}, \frac{1}{3}, …} \cup {0}$. The sequence ${1/n}_{n=1}^\infty$ is in $A$ and converges to $0$. By Definition 5.8, $0$ is a limit point of $A$. Now consider any other point $a \in A$, where $a = \frac{1}{k}$ for some $k \in \mathbb{N}$. If we choose $\epsilon$ small enough such that $(a - \epsilon, a + \epsilon)$ contains no other element of the form $1/n$ (e.g., if $k > 1$, we can choose $\epsilon < |\frac{1}{k} - 1/(k+1)|$), then $a$ is not a limit point (since the neighborhood intersects $A$ only at $a$). Thus, the only limit point of $A$ is $0$. The set of limit points is ${0}$.

**(c) $\mathbb{R}$**

For any $x \in \mathbb{R}$, and for any $\epsilon > 0$, the open interval $(x - \epsilon, x + \epsilon)$ contains infinitely many other real numbers (e.g., $x + \epsilon/2 \neq x$ and is in the interval). Thus, every point in $\mathbb{R}$ is a limit point of $\mathbb{R}$. The set of limit points is $\mathbb{R}$.

**(d) $(0, 1) \cup$**

Let $A = (0, 1) \cup$. For any $x \in (0, 1)$, any $\epsilon$-neighborhood $(x - \epsilon, x + \epsilon)$ will contain other points in $(0, 1)$ if $\epsilon$ is small enough (specifically, if $\epsilon < \min{x, 1-x}$). So, every point in $(0, 1)$ is a limit point. Also, $0$ is a limit point since any $(-\epsilon, \epsilon)$ will intersect $(0, 1)$ at points other than $0$. Similarly, $1$ is a limit point since any $(1 - \epsilon, 1 + \epsilon)$ will intersect $(0, 1)$ at points other than $1$. Thus, the limit points of $(0, 1)$ are $()$. For any $x \in (3, 4)$, similarly, every point is a limit point. Also, $3$ is a limit point since any $(3 - \epsilon, 3 + \epsilon)$ will intersect $()$ at points other than $3$ (e.g., $3 + \epsilon/2$ if $\epsilon/2 \le 1$). Similarly, $4$ is a limit point. Thus, the limit points of $()$ are $()$. Combining these, the set of limit points of $(0, 1) \cup$ is $ \cup$.

**(e) $\mathbb{Q}$**

For any $x \in \mathbb{R}$ and any $\epsilon > 0$, the open interval $(x - \epsilon, x + \epsilon)$ contains rational numbers (by the density of $\mathbb{Q}$ in $\mathbb{R}$). If $x$ itself is rational, then the neighborhood contains other rationals. If $x$ is irrational, the neighborhood still contains rationals other than $x$ (since $x \notin \mathbb{Q}$). Therefore, every real number is a limit point of $\mathbb{Q}$. The set of limit points is $\mathbb{R}$.

**(f) ${17}$**

Consider the set $A = {17}$. For any $\epsilon > 0$, the neighborhood $(17 - \epsilon, 17 + \epsilon)$ will contain points other than $17$ (e.g., $17 + \epsilon/2$). Thus, for any $x$, if we consider an $\epsilon$-neighborhood of $x$, for $x = 17$, the neighborhood contains points other than $17$, but these other points are not in the set ${17}$. Thus, the definition requires intersection with the set *at some point other than x*. If $x \neq 17$, we can choose $\epsilon = |x - 17|/2 > 0$, then $(x - \epsilon, x + \epsilon)$ contains no element of ${17}$, so $x$ is not a limit point. Therefore, the set of limit points of ${17}$ is $\emptyset$.

## Exercise 5.3

For each of the following, provide a proof or a counterexample.

**(a) If $A$ and $B$ are compact, must $A \cup B$ be compact?**

**Proof.** Assume $A$ and $B$ are compact subsets of $\mathbb{R}$. By the Heine-Borel theorem (Theorem 5.19), $A$ is closed and bounded, and $B$ is closed and bounded.

1. Since $A$ is closed and $B$ is closed, their union $A \cup B$ is also closed by Proposition 5.12(i).
2. Since $A$ is bounded, there exists $M_1 > 0$ such that for all $x \in A$, $|x| \le M_1$. Similarly, since $B$ is bounded, there exists $M_2 > 0$ such that for all $x \in B$, $|x| \le M_2$.
3. Let $M = \max{M_1, M_2}$. Then for any $x \in A \cup B$, either $x \in A$ (in which case $|x| \le M_1 \le M$) or $x \in B$ (in which case $|x| \le M_2 \le M$). Thus, $A \cup B$ is bounded.
4. Since $A \cup B$ is closed and bounded, by the Heine-Borel theorem (Theorem 5.19), $A \cup B$ is compact.

**(b) If $A$ and $B$ are compact, must $A \cap B$ be compact?**

**Proof.** Assume $A$ and $B$ are compact subsets of $\mathbb{R}$. By the Heine-Borel theorem (Theorem 5.19), $A$ is closed and bounded, and $B$ is closed and bounded.

1. Since $A$ is closed and $B$ is closed, their intersection $A \cap B$ is also closed by Proposition 5.12(ii).
2. Since $A$ is bounded, there exists $M_1 > 0$ such that for all $x \in A$, $|x| \le M_1$. Similarly, since $B$ is bounded, there exists $M_2 > 0$ such that for all $x \in B$, $|x| \le M_2$.
3. Let $M = \min{M_1, M_2}$. For any $x \in A \cap B$, we have $x \in A$ (so $|x| \le M_1$) and $x \in B$ (so $|x| \le M_2$). Therefore, $|x| \le M$. This shows that $A \cap B$ is bounded.
4. Since $A \cap B$ is closed and bounded, by the Heine-Borel theorem (Theorem 5.19), $A \cap B$ is compact.

## Exercise 5.4

**(a) Prove that if $A$ is closed and $B$ is open, then $A \setminus B$ is closed.**

**Proof.** We want to show that $A \setminus B$ is closed, which means its complement $(A \setminus B)^c = \mathbb{R} \setminus (A \setminus B)$ is open.

1. By definition, $A \setminus B = A \cap B^c$.
2. Therefore, $(A \setminus B)^c = (A \cap B^c)^c$.
3. By De Morgan's laws (Fact 5.11), $(A \cap B^c)^c = A^c \cup (B^c)^c = A^c \cup B$.
4. We are given that $A$ is closed, so its complement $A^c$ is open.
5. We are given that $B$ is open.
6. The union of two open sets, $A^c \cup B$, is open by Proposition 5.3(i).
7. Thus, $(A \setminus B)^c$ is open, which implies that $A \setminus B$ is closed.

**(b) Prove that if $A$ is open and $B$ is closed, then $A \setminus B$ is open.**

**Proof.** We want to show that $A \setminus B$ is open. By Definition 5.1, for every $x \in A \setminus B$, there exists a $\delta > 0$ such that $(x - \delta, x + \delta) \subseteq A \setminus B$.

1. Let $x \in A \setminus B$. By definition, $x \in A$ and $x \notin B$.
2. Since $A$ is open and $x \in A$, there exists $\delta_1 > 0$ such that $(x - \delta_1, x + \delta_1) \subseteq A$.
3. Since $B$ is closed and $x \notin B$, $x$ is in the complement of $B$, $B^c$, which is open.
4. Since $B^c$ is open and $x \in B^c$, there exists $\delta_2 > 0$ such that $(x - \delta_2, x + \delta_2) \subseteq B^c$.
5. Let $\delta = \min{\delta_1, \delta_2}$. Then $\delta > 0$.
6. Consider the $\delta$-neighborhood of $x$, $(x - \delta, x + \delta)$. Since $\delta \le \delta_1$, we have $(x - \delta, x + \delta) \subseteq (x - \delta_1, x + \delta_1) \subseteq A$.
7. Since $\delta \le \delta_2$, we have $(x - \delta, x + \delta) \subseteq (x - \delta_2, x + \delta_2) \subseteq B^c$.
8. Since $(x - \delta, x + \delta) \subseteq A$ and $(x - \delta, x + \delta) \subseteq B^c$, it follows that $(x - \delta, x + \delta) \subseteq A \cap B^c = A \setminus B$.
9. Thus, for every $x \in A \setminus B$, there exists a $\delta > 0$ such that $(x - \delta, x + \delta) \subseteq A \setminus B$, which means $A \setminus B$ is open by Definition 5.1.

## Exercise 5.5

**(a) Give an example of countably many disjoint open intervals.**

Consider the collection of open intervals ${(n, n+1): n \in \mathbb{Z}}$.

1. Each interval $(n, n+1)$ is open by Example 5.2(iii).
2. The collection is indexed by the integers $\mathbb{Z}$, which is a countably infinite set. Thus, there are countably many intervals.
3. The intervals are disjoint. If $m \neq n$, without loss of generality, assume $m < n$. Then $m+1 \le n$. Any $x \in (m, m+1)$ satisfies $m < x < m+1$, and any $y \in (n, n+1)$ satisfies $n < y < n+1$. Since $x < m+1 \le n < y$, there is no overlap, so $(m, m+1) \cap (n, n+1) = \emptyset$.

**(b) Prove that there does not exist a collection of uncountably many disjoint open intervals.**

**Proof by contradiction.**

1. Assume there exists an uncountable collection $\mathcal{F} = {(a_\alpha, b_\alpha): \alpha \in I}$ of disjoint open intervals, where $I$ is an uncountable index set.
2. Since the intervals are disjoint, if $(a_\alpha, b_\alpha) \in \mathcal{F}$, then for any $\alpha \in I$, $a_\alpha < b_\alpha$.
3. By the density of rational numbers in $\mathbb{R}$, for each open interval $(a_\alpha, b_\alpha)$, there exists at least one rational number $q_\alpha$ such that $a_\alpha < q_\alpha < b_\alpha$.
4. Consider the mapping $f: \mathcal{F} \to \mathbb{Q}$ defined by $f((a_\alpha, b_\alpha)) = q_\alpha$.
5. If $(a_\alpha, b_\alpha) \neq (a_\beta, b_\beta)$ are two distinct intervals in $\mathcal{F}$, then since they are disjoint, they must be different intervals, meaning either their left endpoints differ, or their right endpoints differ, or both. In any case, the intervals do not overlap.
6. If $\alpha \neq \beta$, then $(a_\alpha, b_\alpha) \cap (a_\beta, b_\beta) = \emptyset$. Suppose $q_\alpha = q_\beta = q$. Then $a_\alpha < q < b_\alpha$ and $a_\beta < q < b_\beta$. This means $q$ is a common element of both intervals, which contradicts the assumption that the intervals are disjoint.
7. Therefore, if $(a_\alpha, b_\alpha) \neq (a_\beta, b_\beta)$, then $q_\alpha \neq q_\beta$. This means the mapping $f$ is injective (one-to-one).
8. Since there exists an injective mapping from $\mathcal{F}$ to $\mathbb{Q}$, the cardinality of $\mathcal{F}$ must be less than or equal to the cardinality of $\mathbb{Q}$.
9. The set of rational numbers $\mathbb{Q}$ is countable. Therefore, the collection $\mathcal{F}$ must be countable.
10. This contradicts our initial assumption that $\mathcal{F}$ is an uncountable collection.
11. Hence, there does not exist a collection of uncountably many disjoint open intervals in $\mathbb{R}$.

## Exercise 5.6

For each of the following, you should state which sets you are choosing and what their intersection/union is, but you do not need to prove your examples work.

**(a) Give an example of an infinite collection of open sets whose intersection is not open.**

Consider the collection of open intervals $U_n = (-\frac{1}{n}, \frac{1}{n})$ for each $n \in \mathbb{N} = {1, 2, 3, …}$. Each $U_n$ is an open interval centered at $0$, so each $U_n$ is an open set by Example 5.2(iii). The intersection of this infinite collection is $\bigcap_{n=1}^\infty U_n = \bigcap_{n=1}^\infty (-\frac{1}{n}, \frac{1}{n}) = {0}$. The set ${0}$ is not open, as shown in the solution to Exercise 5.1(f).

**(b) Give an example of an infinite collection of closed sets whose union is not closed.**

Consider the collection of closed intervals $F_n = [\frac{1}{n}, 1]$ for each $n \in \mathbb{N} = {1, 2, 3, …}$. Each $F_n$ is a closed interval, so each $F_n$ is a closed set by Example 5.7. The union of this infinite collection is $\bigcup_{n=1}^\infty F_n = \bigcup_{n=1}^\infty [\frac{1}{n}, 1] = (0, 1]$. The interval $(0, 1]$ is not closed because its complement $(-\infty, 0] \cup (1, \infty)$ is not open (specifically at the point $1$, any neighborhood around $1$ contains points less than $1$ in the set). Alternatively, the limit point $0$ of $(0, 1]$ is not in the set.

**(c) Give an example of an infinite collection of compact sets whose union is not compact.**

Consider the collection of closed intervals $K_n = [0, n]$ for each $n \in \mathbb{N} = {1, 2, 3, …}$. Each $K_n$ is a closed and bounded subset of $\mathbb{R}$, so by the Heine-Borel theorem (Theorem 5.19), each $K_n$ is compact. The union of this infinite collection is $\bigcup_{n=1}^\infty K_n = \bigcup_{n=1}^\infty [0, n] = [0, \infty)$. The interval $[0, \infty)$ is closed, but it is not bounded. Therefore, by the Heine-Borel theorem (Theorem 5.19), $[0, \infty)$ is not compact.

## Exercise 5.7

Prove that a point $x$ is a limit point of a set $A$ if and only if every $\epsilon$-neighborhood of $x$ intersects $A$ at some point other than $x$.

We need to prove two implications:

**(i) If $x$ is a limit point of $A$, then every $\epsilon$-neighborhood of $x$ intersects $A$ at some point other than $x$.**

**Proof.**

1. Assume $x$ is a limit point of $A$. By Definition 5.8, there exists a sequence of points $(a_n)$ from $A \setminus {x}$ such that $a_n \to x$.
2. Let $\epsilon > 0$ be given. Since $a_n \to x$, by the definition of convergence, there exists an $N \in \mathbb{N}$ such that for all $n > N$, $|a_n - x| < \epsilon$.
3. This means that for all $n > N$, $a_n \in (x - \epsilon, x + \epsilon)$, so $a_n$ is in the $\epsilon$-neighborhood of $x$.
4. Since $(a_n)$ is a sequence from $A \setminus {x}$, each $a_n \in A$ and $a_n \neq x$.
5. In particular, for $n > N$, we have $a_n \in (x - \epsilon, x + \epsilon)$ and $a_n \in A$ and $a_n \neq x$.
6. Therefore, the $\epsilon$-neighborhood $(x - \epsilon, x + \epsilon)$ contains the point $a_{N+1}$ (for example), which is in $A$ and is not equal to $x$. Thus, $(x - \epsilon, x + \epsilon) \cap (A \setminus {x}) \neq \emptyset$, or equivalently, $(x - \epsilon, x + \epsilon)$ intersects $A$ at some point other than $x$.

**(ii) If every $\epsilon$-neighborhood of $x$ intersects $A$ at some point other than $x$, then $x$ is a limit point of $A$.**

**Proof.**

1. Assume that for every $\epsilon > 0$, the $\epsilon$-neighborhood $(x - \epsilon, x + \epsilon)$ contains at least one point from $A$ that is not equal to $x$. That is, $(x - \epsilon, x + \epsilon) \cap (A \setminus {x}) \neq \emptyset$.
2. For each $n \in \mathbb{N}$, consider the $\epsilon$-neighborhood of $x$ with $\epsilon = 1/n$, which is $(x - 1/n, x + 1/n)$.
3. By our assumption, for each $n$, there exists a point $a_n \in A \setminus {x}$ such that $a_n \in (x - 1/n, x + 1/n)$.
4. This means that for each $n$, $a_n \in A$, $a_n \neq x$, and $|a_n - x| < 1/n$.
5. We have constructed a sequence $(a_n)$ where each $a_n \in A \setminus {x}$.
6. Now we show that $a_n \to x$. Let $\epsilon > 0$ be given. By the Archimedean principle, there exists an $N \in \mathbb{N}$ such that $1/N < \epsilon$.
7. Then, for all $n > N$, we have $1/n < 1/N < \epsilon$.
8. Since $|a_n - x| < 1/n$, for all $n > N$, we have $|a_n - x| < \epsilon$.
9. This is the definition of $a_n \to x$.
10. Since we have a sequence $(a_n)$ from $A \setminus {x}$ that converges to $x$, by Definition 5.8, $x$ is a limit point of $A$.

Since we have proven both directions, the statement is true.

## Exercise 5.8

Construct a set $A$ whose set of limit points is $\mathbb{Z}$.

Consider the set $A = \bigcup_{n \in \mathbb{Z}} (n, n + \frac{1}{2})$. Each interval $(n, n + \frac{1}{2})$ is open. Consider an integer $m \in \mathbb{Z}$. For any $\epsilon > 0$, the neighborhood $(m - \epsilon, m + \epsilon)$ will contain points from $(m - 1, m - 1 + \frac{1}{2})$ if $m - \epsilon < m - \frac{1}{2} < m$, or from $(m, m + \frac{1}{2})$ if $m < m + \frac{1}{2} < m + \epsilon$, or both, provided $\epsilon$ is small enough. In either case, we can find points in $A$ arbitrarily close to $m$, and not equal to $m$ (since $m \notin A$). For example, $m + \delta \in (m, m + \frac{1}{2}) \subset A$ for small $\delta > 0$. Thus, every integer is a limit point of $A$.

Now consider a non-integer $y \in \mathbb{R} \setminus \mathbb{Z}$. Case 1: $y \in (n, n + \frac{1}{2})$ for some integer $n$. We can choose $\epsilon = \min{y - n, n + \frac{1}{2} - y} / 2 > 0$. Then the neighborhood $(y - \epsilon, y + \epsilon)$ is entirely contained in $(n, n + \frac{1}{2}) \subset A$. If we choose $\epsilon$ even smaller, say $\epsilon' = \min{y - n, n + \frac{1}{2} - y, \delta}$ where $\delta$ is such that $(y - \epsilon', y + \epsilon')$ does not contain any integer. Then, any point in this neighborhood other than $y$ is still in $(n, n + \frac{1}{2}) \subset A$. So $y$ is a limit point.

Wait, the question is about the set of limit points being *precisely* $\mathbb{Z}$. My current construction seems to have all points in the open intervals as limit points as well. Let's try a different approach.

Consider the set $A = {n + \frac{1}{k}: n \in \mathbb{Z}, k \in \mathbb{N}, k \ge 2}$. For any integer $m \in \mathbb{Z}$, consider an $\epsilon$-neighborhood $(m - \epsilon, m + \epsilon)$. If we choose $k$ large enough such that $0 < \frac{1}{k} < \epsilon$, then $m + \frac{1}{k} \in A$ and $m < m + \frac{1}{k} < m + \epsilon$. So, every integer is a limit point of $A$.

Now consider a non-integer $y \in \mathbb{R} \setminus \mathbb{Z}$. If $y$ is not an integer, then there exists an integer $m$ such that $m < y < m+1$. If $y \in (m, m+1)$, can $y$ be a limit point of $A$? Consider the distance from $y$ to the nearest point in $A$. The points in $A$ around $m$ are $m + 1/2, m + 1/3, m + 1/4, …$ and around $m+1$ are $m+1 + 1/2, m+1 + 1/3, …$ (which are greater than $y$) and $m+1 - 1/2 = m + 1/2, m+1 - 1/3 = m + 2/3, m+1 - 1/4 = m + 3/4, …$ (which are less than $y$ if $y$ is close to $m+1$). Let $d = \min_{a \in A} |y - a|$. If $d > 0$, we can choose $\epsilon = d/2$, and $(y - \epsilon, y + \epsilon)$ will contain no points from $A$, so $y$ would not be a limit point. We need to ensure $d > 0$.

Let $m < y < m+1$. If $y = m + \frac{1}{k}$ for some integer $k \ge 2$, then we need to consider neighborhoods around $y$ excluding $y$ itself. For $y = m + \frac{1}{k}$, consider points $m + 1/(k+j)$ which converge to $y$ as $j \to \infty$, and these are in $A$ and not equal to $y$. So points in $A$ are limit points of $A$.

We need the set of limit points to be *exactly* $\mathbb{Z}$. Consider $A = \bigcup_{n \in \mathbb{Z}} {n + \frac{1}{k}: k \in \mathbb{N}, k \ge 2}$. For any integer $m$, and any $\epsilon > 0$, we can find $k$ large enough so that $0 < \frac{1}{k} < \epsilon$, and $m + \frac{1}{k} \in A$, $m + \frac{1}{k} \neq m$, and $|(m + \frac{1}{k}) - m| = \frac{1}{k} < \epsilon$. So $\mathbb{Z}$ is a subset of the set of limit points of $A$.

Now consider $y \notin \mathbb{Z}$. Then there exists an integer $m$ such that $m < y < m+1$. Let $\delta = \min{y - m, m + 1 - y} > 0$. The points in $A$ are of the form $n + \frac{1}{k}$. The closest such points to $y$ will be when $n = m$ or $n = m+1$. For $n = m$, the points are $m + 1/2, m + 1/3, …$ which converge to $m$. For $n = m+1$, the points are $m+1 + 1/2, m+1 + 1/3, …$ and $m+1 - 1/2 = m + 1/2, m+1 - 1/3 = m + 2/3, m+1 - 1/4 = m + 3/4, …$ The distance between $y$ and the set ${m + \frac{1}{k}: k \ge 2}$ will be some positive value (since the set converges to $m$). Similarly, the distance between $y$ and ${m+1 - \frac{1}{k}: k \ge 2}$ will be positive (since this set converges to $m+1$). Therefore, we can find a small enough neighborhood around $y$ that contains no points of $A$.

**Construction:** Let $A = {n + \frac{1}{k}: n \in \mathbb{Z}, k \in \mathbb{Z}, |k| \ge 2}$. This is equivalent to $A = {n + \frac{1}{k}: n \in \mathbb{Z}, k \in \mathbb{N}, k \ge 2} \cup {n - \frac{1}{k}: n \in \mathbb{Z}, k \in \mathbb{N}, k \ge 2}$. For any integer $m$, $m + \frac{1}{k} \to m$ and $m - \frac{1}{k} \to m$ as $k \to \infty$, and these points are in $A$ and not equal to $m$. So $\mathbb{Z}$ is a subset of the limit points of $A$. If $y \notin \mathbb{Z}$, then $m < y < m+1$ for some integer $m$. The set of points in $A$ in $(m, m+1)$ are ${m + \frac{1}{k}: k \ge 2}$ and ${m+1 - \frac{1}{k}: k \ge 2}$. These sets converge to $m$ and $m+1$ respectively. Thus, there is a positive distance between $y$ and the set $A \cap (m, m+1)$. Therefore, $y$ is not a limit point.

The set $A = {n + \frac{1}{k}: n \in \mathbb{Z}, k \in \mathbb{Z} \setminus {-1, 0, 1}}$ has $\mathbb{Z}$ as its set of limit points.

## Exercise 5.9

Does there exist a set $A$ whose set of limit points is precisely $\mathbb{Q}$?

This is a more challenging question. Consider constructing $A$ by taking each rational number $q \in \mathbb{Q}$ and creating a sequence in $A \setminus {q}$ that converges to $q$. We need to ensure that no irrational number becomes a limit point.

Let ${q_n}_{n=1}^\infty$ be an enumeration of $\mathbb{Q}$. For each $q_n$, consider the set $A_n = {q_n + \frac{1}{k}: k \in \mathbb{N}, k \ge 2}$. The limit point of $A_n$ is $q_n$. Let $A = \bigcup_{n=1}^\infty A_n = \bigcup_{n=1}^\infty {q_n + \frac{1}{k}: k \in \mathbb{N}, k \ge 2}$. The set of limit points of $A$ contains $\mathbb{Q}$. If $q \in \mathbb{Q}$, then $q = q_n$ for some $n$, and $q_n + \frac{1}{k} \to q_n$ as $k \to \infty$, and $q_n + \frac{1}{k} \in A \setminus {q_n}$.

Now, consider an irrational number $x$. For $x$ to be a limit point, every neighborhood of $x$ must contain a point from $A$ other than $x$. A point in $A$ is of the form $q_n + \frac{1}{k}$. If $x$ is a limit point, then there exist sequences $q_{n_j}$ and $k_j$ such that $q_{n_j} + \frac{1}{k}_j \to x$. This means $q_{n_j} \to x$ (since $\frac{1}{k}_j \to 0$). But if a sequence of rational numbers converges to an irrational number, then every neighborhood of the irrational number must contain infinitely many distinct rational numbers.

It seems plausible that such a set exists. We can construct sequences of rational numbers converging to each rational number in a controlled way.

**Consider the set $A = \bigcup_{q \in \mathbb{Q}} {q + \frac{1}{n}: n \in \mathbb{N}, n \ge 2}$.** If $p \in \mathbb{Q}$, then $p + 1/n \in A$ and $p + 1/n \to p$ as $n \to \infty$, and $p + 1/n \neq p$. So $\mathbb{Q}$ is in the set of limit points of $A$.

If $x \notin \mathbb{Q}$, suppose $x$ is a limit point. Then there exists a sequence $q_j + 1/n_j \in A \setminus {x}$ such that $q_j + 1/n_j \to x$. This implies $q_j \to x$. But since $q_j \in \mathbb{Q}$ and $x \notin \mathbb{Q}$, this sequence must have infinitely many distinct terms $q_j$. For any $\epsilon > 0$, $(x - \epsilon, x + \epsilon)$ contains infinitely many rationals from the sequence $(q_j)$. The points in $A$ in this neighborhood are $q_j + 1/n_j$.

Yes, such a set exists.

## Exercise 5.10

Show that the set of limit points of a set is closed.

Let $A \subseteq \mathbb{R}$ and let $A'$ be the set of limit points of $A$. We want to show that $A'$ is closed, which means that every limit point of $A'$ is in $A'$ (by Theorem 5.10).

1. Let $y$ be a limit point of $A'$. This means that every $\epsilon$-neighborhood of $y$, $(y - \epsilon, y + \epsilon)$, contains a point $z \in A'$ such that $z \neq y$ (by Exercise 5.7).
2. Since $z \in A'$, $z$ is a limit point of $A$. Therefore, every $\delta$-neighborhood of $z$, $(z - \delta, z + \delta)$, contains a point $a \in A$ such that $a \neq z$.
3. Let $\epsilon > 0$. Since $(y - \epsilon, y + \epsilon)$ contains $z \in A'$ with $z \neq y$, choose $\delta > 0$ small enough such that $(z - \delta, z + \delta) \subseteq (y - \epsilon, y + \epsilon)$ and $\delta < |z - y|/2$ (so $y \notin (z - \delta, z + \delta)$).
4. Since $z$ is a limit point of $A$, $(z - \delta, z + \delta)$ contains a point $a \in A$ with $a \neq z$.
5. Since $(z - \delta, z + \delta) \subseteq (y - \epsilon, y + \epsilon)$, we have $a \in (y - \epsilon, y + \epsilon)$.
6. Also, $a \neq y$. If $a = y$, then $y \in (z - \delta, z + \delta)$, but we chose $\delta$ such that this is not the case.
7. Thus, for every $\epsilon > 0$, the neighborhood $(y - \epsilon, y + \epsilon)$ contains a point $a \in A$ such that $a \neq y$.
8. By Exercise 5.7, this means that $y$ is a limit point of $A$, so $y \in A'$.
9. Since every limit point of $A'$ is in $A'$, $A'$ is closed by Theorem 5.10.

## Exercise 5.11

Prove Proposition 5.12. That is, prove the following.

**(a) If ${U_1, U_2, …, U_n}$ is a collection of closed sets, then $\bigcup_{k=1}^n U_k$ is also a closed set.**

**Proof.** We will use De Morgan's laws (Fact 5.11) and the fact that a set is closed if and only if its complement is open.

1. Let $F = \bigcup_{k=1}^n U_k$, where each $U_k$ is a closed set.
2. We want to show that $F$ is closed, so we consider its complement $F^c = (\bigcup_{k=1}^n U_k)^c$.
3. By De Morgan's laws, $(\bigcup_{k=1}^n U_k)^c = \bigcap_{k=1}^n U_k^c$.
4. Since each $U_k$ is closed, its complement $U_k^c$ is open for each $k = 1, 2, …, n$.
5. The intersection of a finite collection of open sets is open by Proposition 5.3(ii).
6. Therefore, $\bigcap_{k=1}^n U_k^c$ is open.
7. Since $F^c = \bigcap_{k=1}^n U_k^c$ is open, the set $F = \bigcup_{k=1}^n U_k$ is closed.

**(b) If ${U_\alpha}_{\alpha \in S}$ is a collection of closed sets, then $\bigcap_{\alpha \in S} U_\alpha$ is also a closed set.**

**Proof.**

1. Let $F = \bigcap_{\alpha \in S} U_\alpha$, where each $U_\alpha$ is a closed set.
2. We want to show that $F$ is closed, so we consider its complement $F^c = (\bigcap_{\alpha \in S} U_\alpha)^c$.
3. By De Morgan's laws (Fact 5.11), $(\bigcap_{\alpha \in S} U_\alpha)^c = \bigcup_{\alpha \in S} U_\alpha^c$.
4. Since each $U_\alpha$ is closed, its complement $U_\alpha^c$ is open for each $\alpha \in S$.
5. The union of any collection of open sets is open by Proposition 5.3(i).
6. Therefore, $\bigcup_{\alpha \in S} U_\alpha^c$ is open.
7. Since $F^c = \bigcup_{\alpha \in S} U_\alpha^c$ is open, the set $F = \bigcap_{\alpha \in S} U_\alpha$ is closed.
Alright class, let's continue working through the exercises from Chapter 5. Today, we'll focus on the second third of the exercises, starting around Exercise 5.12 and going through Exercise 5.22. Remember to pay close attention to the definitions and theorems we've covered so far.

## Exercise 5.12

For each of the following tasks, give an example as requested or prove that one does not exist.

**(a) A nonempty open set that is a subset of Q.**

Solution:

1. Consider the intersection of the open interval $(a - \delta, a + \delta)$ where $a \in \mathbb{Q}$ and $\delta > 0$, with the set of rational numbers $\mathbb{Q}$.
2. Let $U = (0, 1) \cap \mathbb{Q}$.
3. $U$ is nonempty, as it contains rational numbers between 0 and 1 (e.g., $1/2 \in U$).
4. $U$ is a subset of $\mathbb{Q}$ by construction.
5. However, $U$ is **not open** in $\mathbb{R}$. To see this, pick any $x \in U$. For any $\delta > 0$, the interval $(x - \delta, x + \delta)$ contains irrational numbers. Thus, there is no $\delta > 0$ such that $(x - \delta, x + \delta) \subseteq U \subseteq \mathbb{Q}$.
6. Therefore, there does not exist a nonempty open set (in $\mathbb{R}$) that is a subset of $\mathbb{Q}$.

**(b) A nonempty closed set that is a subset of Q.**

Solution:

1. Consider a set containing a single rational number.
2. Let $A = \{q\}$ where $q \in \mathbb{Q}$.
3. $A$ is nonempty since it contains $q$.
4. $A$ is a subset of $\mathbb{Q}$.
5. To show that $A$ is closed, we consider its complement $A^c = \mathbb{R} \setminus \{q\}$.
6. For any $x \in A^c$, if $x \neq q$, we can find a $\delta = |x - q| / 2 > 0$ such that the open interval $(x - \delta, x + \delta)$ does not contain $q$.
7. Therefore, $(x - \delta, x + \delta) \subseteq A^c$, which means $A^c$ is open.
8. Since the complement of $A$ is open, $A = \{q\}$ is a closed set.
9. Thus, a nonempty closed set that is a subset of $\mathbb{Q}$ exists. An example is any singleton set $\{q\}$ where $q$ is a rational number.

**(c) Two nonempty disjoint open sets whose union is R.**

Solution:

1. Assume, for the sake of contradiction, that there exist two nonempty disjoint open sets $U$ and $V$ such that $U \cup V = \mathbb{R}$ and $U \cap V = \emptyset$.
2. Since $U$ is nonempty, there exists some $u \in U$.
3. Since $V$ is nonempty, there exists some $v \in V$.
4. Without loss of generality, assume $u < v$.
5. Consider the set $S = U \cap (-\infty, v]$. $S$ is nonempty since $u \in S$.
6. $S$ is bounded above by $v$, so it has a supremum, let $x = \sup(S)$.
7. Since $x \le v$, $x \in \mathbb{R} = U \cup V$, so either $x \in U$ or $x \in V$.
8. Case 1: $x \in U$. Since $U$ is open, there exists a $\delta > 0$ such that $(x - \delta, x + \delta) \subseteq U$.
9. If we choose $\delta$ small enough such that $x + \delta < v$, then $(x - \delta, x + \delta) \subseteq U \cap (-\infty, v] = S$. This contradicts the fact that $x = \sup(S)$, as $x + \delta / 2 \in S$ and $x + \delta / 2 > x$.
10. Case 2: $x \in V$. Since $V$ is open, there exists a $\delta > 0$ such that $(x - \delta, x + \delta) \subseteq V$.
11. Since $x = \sup(S)$, for any $\epsilon > 0$, there exists an element $y \in S$ such that $x - \epsilon < y \le x$.
12. Choose $\epsilon = \delta / 2$. Then there exists $y \in S = U \cap (-\infty, v]$ such that $x - \delta / 2 < y \le x$.
13. Since $y \in U$ and $x - \delta / 2 < y$, $y$ is in the neighborhood $(x - \delta, x + \delta) \subseteq V$.
14. This contradicts the fact that $U$ and $V$ are disjoint ($U \cap V = \emptyset$).
15. Therefore, there do not exist two nonempty disjoint open sets whose union is $\mathbb{R}$.

**(d) An infinite set with no limit points.**

Solution:

1. Consider the set of integers $\mathbb{Z} = \{\dots, -2, -1, 0, 1, 2, \dots\}$.
2. $\mathbb{Z}$ is an infinite set.
3. For any $x \in \mathbb{R}$, we need to show that $x$ is not a limit point of $\mathbb{Z}$.
4. If $x \in \mathbb{Z}$, let $\delta = 1/2$. Then the neighborhood $(x - 1/2, x + 1/2)$ contains no other integers besides $x$. Therefore, there is a neighborhood of $x$ that intersects $\mathbb{Z}$ only at $x$, so $x$ is not a limit point.
5. If $x \notin \mathbb{Z}$, there exists an integer $n$ such that $n < x < n + 1$. Let $\delta = \min(x - n, n + 1 - x) / 2 > 0$.
6. Then the neighborhood $(x - \delta, x + \delta)$ contains no integers, so it does not intersect $\mathbb{Z}$ at any point. Thus, $x$ is not a limit point.
7. Since no point in $\mathbb{R}$ is a limit point of $\mathbb{Z}$, $\mathbb{Z}$ is an infinite set with no limit points.

**(e) A bounded infinite set with no limit points.**

Solution:

1. If a set $A$ is bounded and infinite, then by the Bolzano-Weierstrass Theorem (which is related to the Heine-Borel theorem), every sequence in $A$ has a convergent subsequence.
2. If $A$ has no limit points, then for every $x \in \mathbb{R}$, there exists an $\epsilon$-neighborhood of $x$ that contains at most $x$ itself (if $x \in A$).
3. Consider an infinite sequence $(a_n)$ of distinct elements from $A$ (since $A$ is infinite). Since $A$ is bounded, this sequence is bounded.
4. By the Bolzano-Weierstrass Theorem, there exists a subsequence $(a_{n_k})$ that converges to some limit $L \in \mathbb{R}$.
5. If $L \in A$, then for any $\epsilon > 0$, the neighborhood $(L - \epsilon, L + \epsilon)$ contains infinitely many terms of the subsequence $(a_{n_k})$ (and thus infinitely many elements of $A$), so $L$ is a limit point of $A$, which contradicts our assumption.
6. If $L \notin A$, then for any $\epsilon > 0$, the neighborhood $(L - \epsilon, L + \epsilon)$ contains infinitely many terms of the subsequence $(a_{n_k})$, all of which are in $A \setminus \{L\}$. Thus, $L$ is a limit point of $A$, again a contradiction.
7. Therefore, there does not exist a bounded infinite set with no limit points in $\mathbb{R}$.

**(f) An infinite union of compact sets that is not compact.**

Solution:

1. Consider the set of closed intervals $[0, 1 - 1/n]$ for $n \in \mathbb{N}$, $n \ge 1$.
2. Each set $[0, 1 - 1/n]$ is closed and bounded in $\mathbb{R}$, hence compact by the Heine-Borel Theorem.
3. Consider the infinite union of these sets: $A = \bigcup_{n=1}^{\infty} [0, 1 - 1/n]$.
4. Let's examine the union:
    - For $n=1$, we have $ = \{0\}$.
    - For $n=2$, we have $[0, 1/2]$.
    - For $n=3$, we have $[0, 2/3]$.
    - In general, $[0, 1 - 1/n] \subseteq [0, 1 - 1/(n+1)]$.
5. Therefore, the union is $A = [0, \sup\{1 - 1/n: n \in \mathbb{N}\})$.
6. Since $\lim_{n \to \infty} (1 - 1/n) = 1$, the supremum is 1.
7. Thus, $A = [0, 1)$.
8. The interval $[0, 1)$ is bounded. However, it is not closed because the limit point 1 is not in the set.
9. Since $A = [0, 1)$ is not closed, by the Heine-Borel Theorem, it is not compact.
10. Therefore, this is an example of an infinite union of compact sets that is not compact.

**(g) An infinite intersection of compact sets that is not compact.**

Solution:

1. Consider the set of closed intervals $[-n, n]$ for $n \in \mathbb{N}$, $n \ge 1$.
2. Each set $[-n, n]$ is closed and bounded in $\mathbb{R}$, hence compact by the Heine-Borel Theorem.
3. Consider the infinite intersection of these sets: $B = \bigcap_{n=1}^{\infty} [-n, n]$.
4. A number $x \in \mathbb{R}$ is in the intersection if and only if $x \in [-n, n]$ for all $n \in \mathbb{N}$.
5. This means $|x| \le n$ for all $n \in \mathbb{N}$.
6. The only real number that satisfies this condition is $x = 0$.
7. Therefore, $B = \{0\}$.
8. The set $\{0\}$ is a finite set, and finite sets are closed and bounded (hence compact).
9. Thus, this example does not yield a non-compact intersection.
10. Let's try another example. Consider the closed intervals $[-1/n, 1/n]$ for $n \in \mathbb{N}$, $n \ge 1$.
11. Each set $[-1/n, 1/n]$ is closed and bounded, hence compact.
12. Consider the infinite intersection $C = \bigcap_{n=1}^{\infty} [-1/n, 1/n]$.
13. A number $x \in \mathbb{R}$ is in the intersection if and only if $|x| \le 1/n$ for all $n \in \mathbb{N}$.
14. This implies that $|x|$ must be less than or equal to any positive number, which means $x = 0$.
15. So, $C = \{0\}$, which is compact.
16. It turns out that the intersection of any collection of closed sets is closed.
17. If we have an infinite intersection of compact sets $K_n$, then each $K_n$ is closed and bounded.
18. The intersection $\bigcap_{n=1}^{\infty} K_n$ is closed because it is an intersection of closed sets.
19. Moreover, if each $K_n$ is bounded, then their intersection must also be bounded (by the minimum of the upper bounds and the maximum of the lower bounds, if they exist; if not, any bound that works for one of them will work for the intersection).
20. Therefore, the infinite intersection of compact sets in $\mathbb{R}$ is always closed and bounded, and hence compact by the Heine-Borel Theorem.
21. Thus, an infinite intersection of compact sets cannot be not compact in $\mathbb{R}$.

## Exercise 5.13

**(a) If finitely many points are removed from an open set, must the set still be open?**

Solution:

1. Let $U$ be an open set in $\mathbb{R}$, and let $F = \{x_1, x_2, \dots, x_n\}$ be a finite set of points in $\mathbb{R}$. Consider the set $V = U \setminus F = U \cap F^c$.
2. The complement of a finite set $F^c = \mathbb{R} \setminus \{x_1, x_2, \dots, x_n\} = \bigcap_{i=1}^{n} (\mathbb{R} \setminus \{x_i\})$.
3. For each $x_i$, the set $\{x_i\}$ is closed (as shown in Exercise 5.12(b)), so its complement $\mathbb{R} \setminus \{x_i\}$ is open.
4. $F^c$ is a finite intersection of open sets, which is also open by Proposition 5.3(ii).
5. $V = U \cap F^c$ is the intersection of two open sets ($U$ and $F^c$), which is open by Proposition 5.3(ii).
6. Therefore, if finitely many points are removed from an open set, the resulting set is still open.

**(b) If countably many points are removed from an open set, must the set still be open?**

Solution:

1. No, the set does not necessarily remain open.
2. Consider the open set $U = (-1, 1)$.
3. Let $Q' = \{q_n\}_{n=1}^{\infty}$ be the set of rational numbers in $(-1, 1)$, which is countably infinite.
4. Consider the set $V = U \setminus Q' = (-1, 1) \setminus \mathbb{Q} = (-1, 1) \cap \mathbb{Q}^c$, which is the set of irrational numbers in $(-1, 1)$.
5. Pick any point $x \in V$ (so $x$ is irrational and $-1 < x < 1$).
6. For any $\delta > 0$, the open interval $(x - \delta, x + \delta)$ contains rational numbers (by the density of $\mathbb{Q}$ in $\mathbb{R}$).
7. Therefore, for any $\delta > 0$, $(x - \delta, x + \delta) \not\subseteq V$, because it contains rational numbers that were removed from $U$.
8. By Definition 5.1, $V$ is not open because there exists a point $x \in V$ such that no $\delta$-neighborhood of $x$ is contained in $V$.

**(c) If uncountably many points are removed from an open set, must the set still be open?**

Solution:

1. No, the set does not necessarily remain open.
2. Consider any nonempty open interval $U = (a, b)$. This set contains uncountably many points.
3. Let $A = (a, b) \setminus (a, b) = \emptyset$. Here, we removed uncountably many points (all the points in $(a, b)$ if $(a, b)$ is uncountable, which it is). The resulting set $\emptyset$ is open.
4. However, consider $U = (0, 1)$, and remove the interval $[1/2, 1)$. The remaining set is $[0, 1/2)$, which is not open since for the point 0, no neighborhood around 0 is contained in $[0, 1/2)$. The number of removed points is uncountable.
5. Let's refine this. Consider $U = (0, 1)$, which is open. Let $A = (1/2, 1)$, which is an uncountable subset of $U$.
6. Consider $V = U \setminus A = (0, 1) \setminus (1/2, 1) = (0, 1/2]$.
7. The set $V = (0, 1/2]$ is not open because for the point $1/2 \in V$, any neighborhood $(1/2 - \delta, 1/2 + \delta)$ with $\delta > 0$ will contain points greater than $1/2$ that are not in $V$.
8. The number of points removed, the cardinality of $(1/2, 1)$, is uncountable.
9. Therefore, if uncountably many points are removed from an open set, the set does not necessarily remain open.

## Exercise 5.14

Let A be a closed set, let $x$ be a point from A, and let $B = A \setminus \{x\}$. Give necessary and sufficient conditions on A and $x$ for B to be a closed set. Prove that your conditions works.

Solution:

**Condition:** $x$ is an isolated point of $A$. That is, there exists a $\delta > 0$ such that $(x - \delta, x + \delta) \cap A = \{x\}$.

**Proof:**

**($\implies$) Necessary Condition:** Assume $B = A \setminus \{x\}$ is closed.

1. Since $B$ is closed, its complement $B^c = \mathbb{R} \setminus B = (\mathbb{R} \setminus A) \cup \{x\} = A^c \cup \{x\}$ is open.
2. Since $x \in B^c$ and $B^c$ is open, there exists a $\delta > 0$ such that the $\delta$-neighborhood of $x$, $(x - \delta, x + \delta)$, is contained in $B^c$.
3. Thus, $(x - \delta, x + \delta) \subseteq A^c \cup \{x\}$.
4. We also know that $(x - \delta, x + \delta)$ contains points other than $x$ (since $\delta > 0$). These points must be in $A^c = \mathbb{R} \setminus A$.
5. This implies that $(x - \delta, x + \delta) \setminus \{x\} \subseteq A^c$, so $((x - \delta, x + \delta) \setminus \{x\}) \cap A = \emptyset$.
6. Therefore, $(x - \delta, x + \delta) \cap A \subseteq \{x\}$.
7. Since $x \in A$, we have $(x - \delta, x + \delta) \cap A = \{x\}$.
8. This means $x$ is an isolated point of $A$.

**($\impliedby$) Sufficient Condition:** Assume $x$ is an isolated point of $A$.

1. Since $x$ is an isolated point of $A$, there exists a $\delta > 0$ such that $(x - \delta, x + \delta) \cap A = \{x\}$.
2. Consider the complement of $B = A \setminus \{x\}$, which is $B^c = \mathbb{R} \setminus B = A^c \cup \{x\}$.
3. We want to show that $B^c$ is open. Let $y \in B^c$.
4. Case 1: $y \in A^c = \mathbb{R} \setminus A$. Since $A$ is closed, $A^c$ is open. Thus, there exists an $\epsilon > 0$ such that $(y - \epsilon, y + \epsilon) \subseteq A^c \subseteq B^c$.
5. Case 2: $y = x$. We know there exists a $\delta > 0$ such that $(x - \delta, x + \delta) \cap A = \{x\}$.
6. This implies that for any $z \in (x - \delta, x + \delta)$ with $z \neq x$, we have $z \notin A$, so $z \in A^c$.
7. Therefore, $(x - \delta, x + \delta) \subseteq A^c \cup \{x\} = B^c$.
8. In both cases, for any $y \in B^c$, there exists a neighborhood of $y$ contained in $B^c$.
9. Thus, $B^c$ is open, which means $B$ is closed.

Therefore, $B = A \setminus \{x\}$ is closed if and only if $x$ is an isolated point of $A$.

## Exercise 5.15

Prove that $\mathcal{U} = \{ (1/k, 4 - 1/k) \}_{k=1}^{\infty} \cup \{(-0.2, 0.2)\}$ is an open cover of $(0, 4)$, but that this cover has no finite subcover. This implies that $(0, 4)$ is not compact.

**Proof:**

**Part 1: $\mathcal{U}$ is an open cover of $(0, 4)$.**

1. Each interval $(1/k, 4 - 1/k)$ for $k \in \mathbb{N}$ is an open interval, hence an open set.
2. The interval $(-0.2, 0.2)$ is also an open interval, hence an open set.
3. Therefore, $\mathcal{U}$ is a collection of open sets, meaning it is an open cover if its union contains $(0, 4)$.
4. Consider any $x \in (0, 4)$.

    - Case 1: $0 < x \le 0.2$. Then $x \in (-0.2, 0.2)$, so $x$ is covered by one of the sets in $\mathcal{U}$.
    - Case 2: $0.2 < x < 4$. Since $x > 0.2 > 0$, there exists a $k \in \mathbb{N}$ such that $1/k < x$. (By the Archimedean Principle, there exists $k$ such that $k > 1/x$, so $1/k < x$.)
    - Also, since $x < 4$, for sufficiently large $k$, $x < 4 - 1/k$. (As $k \to \infty$, $4 - 1/k \to 4$.) We need to find $k$ such that $1/k < 4 - x$, or $k > 1 / (4 - x)$.
    - Let $K = \max(\lceil 1/x \rceil + 1, \lceil 1 / (4 - x) \rceil + 1)$. Then for $k \ge K$, we have $1/k < x$ and $x < 4 - 1/k$, so $x \in (1/k, 4 - 1/k)$.
5. Therefore, every $x \in (0, 4)$ is contained in some open set in $\mathcal{U}$, so $\bigcup_{U \in \mathcal{U}} U \supseteq (0, 4)$.
6. Thus, $\mathcal{U}$ is an open cover of $(0, 4)$.

**Part 2: $\mathcal{U}$ has no finite subcover.**

1. Suppose, for contradiction, that there exists a finite subcover $\mathcal{V} \subseteq \mathcal{U}$ that covers $(0, 4)$.
2. Then $\mathcal{V}$ must contain finitely many sets of the form $(1/k_i, 4 - 1/k_i)$ for $i = 1, \dots, n$, and possibly the set $(-0.2, 0.2)$.
3. Let $k_{max} = \max\{k_1, k_2, \dots, k_n\}$.
4. The union of the finitely many intervals $(1/k_i, 4 - 1/k_i)$ is contained within the largest of these intervals, which is $(1/k_{max}, 4 - 1/k_{max})$.
5. So, the union of the finite subcover $\mathcal{V}$ is a subset of $(-0.2, 0.2) \cup (1/k_{max}, 4 - 1/k_{max})$.
6. Consider a point $y$ such that $0 < y < 1/k_{max}$ and $y > 0.2$. Such a $y$ exists if $1/k_{max} > 0.2$, which is true since $k_{max}$ is a positive integer. For example, we can choose $y = \max(0.2 + \epsilon, \frac{1}{k_{max}} - \epsilon)$ for a small $\epsilon > 0$, provided $\frac{1}{k_{max}} > 0.2$. If $k_{max} \le 5$, then $1/k_{max} \ge 1/5 = 0.2$. If $k_{max} = 5$, take $y = 0.21$. If $k_{max} > 5$, take $y$ such that $0.2 < y < 1/k_{max}$.
7. Consider a point $z \in (0, 1/k_{max})$. For instance, $z = 1/(2k_{max})$. Since $k_{max} \ge 1$, $0 < z \le 1/2$, so $z \in (0, 4)$.
8. However, $z$ is not in $(-0.2, 0.2)$ if $1/(2k_{max}) \ge 0.2$, i.e., $1 \ge 0.4 k_{max}$ or $k_{max} \le 2.5$. If $k_{max} = 1$, $z = 1/2 \notin (-0.2, 0.2)$. If $k_{max} = 2$, $z = 1/4 \notin (-0.2, 0.2)$.
9. And $z = 1/(2k_{max})$ is also not in $(1/k_{max}, 4 - 1/k_{max})$ since $z < 1/k_{max}$.
10. Let's consider a different approach for the finite subcover. Let the finite subcover be $\{(1/k_1, 4 - 1/k_1), \dots, (1/k_n, 4 - 1/k_n), (-0.2, 0.2)\}$.
11. Let $k_{max} = \max\{k_1, \dots, k_n\}$. The union of the intervals is contained in $(0, 1/k_{max}) \cup [1/k_{max}, 4 - 1/k_{max}] \cup (4 - 1/k_{max}, 4)$ along with $(-0.2, 0.2)$. The union of the first $n$ intervals is $(\min(1/k_i), \max(4 - 1/k_i)) = (1/k_{max}, 4 - 1/k_{max})$.
12. The union of the finite subcover is $(-0.2, 0.2) \cup (1/k_{max}, 4 - 1/k_{max})$.
13. If $k_{max}$ is finite, then $1/k_{max} > 0$. Consider a point $y \in (0, \min(0.2, 1/k_{max}))$, such that $y > 0$. Such a point exists.
14. This point $y$ is in $(0, 4)$. However, $y \notin (-0.2, 0.2)$ if $y$ is positive, and $y \notin (1/k_{max}, 4 - 1/k_{max})$ since $y < 1/k_{max}$.
15. This contradicts the assumption that the finite subcover covers $(0, 4)$.
16. Therefore, $\mathcal{U}$ has no finite subcover, which implies that $(0, 4)$ is not compact.

## Exercise 5.16

Let $A \subseteq \mathbb{R}$. Prove that $A$ is closed and bounded (i.e., compact) if and only if every sequence of numbers from $A$ has a subsequence that converges to a point in $A$.

**Proof:**

**($\implies$) Assume $A$ is closed and bounded.**

1. Let $(a_n)$ be a sequence of numbers from $A$.
2. Since $A$ is bounded, the sequence $(a_n)$ is also bounded.
3. By the Bolzano-Weierstrass Theorem, every bounded sequence in $\mathbb{R}$ has a convergent subsequence. Let $(a_{n_k})$ be a convergent subsequence of $(a_n)$, and let $\lim_{k \to \infty} a_{n_k} = L$.
4. Since every term of the subsequence $(a_{n_k})$ is in $A$, $L$ is a limit point of $A$ (if all $a_{n_k}$ are distinct) or $L = a_{n_k}$ for all sufficiently large $k$ (if the subsequence is eventually constant).
5. Since $A$ is closed, it contains all of its limit points (Theorem 5.10).
6. Therefore, $L \in A$.
7. Thus, every sequence of numbers from $A$ has a subsequence that converges to a point in $A$.

**($\impliedby$) Assume that every sequence of numbers from $A$ has a subsequence that converges to a point in $A$.**

**Part 1: $A$ is bounded.**

1. Assume, for contradiction, that $A$ is not bounded.
2. Then for every $n \in \mathbb{N}$, there exists a point $a_n \in A$ such that $|a_n| > n$.
3. Consider the sequence $(a_n)$ formed by these points.
4. Any subsequence $(a_{n_k})$ will satisfy $|a_{n_k}| > n_k$.
5. Since $n_k \to \infty$ as $k \to \infty$, we have $|a_{n_k}| \to \infty$ as $k \to \infty$.
6. Therefore, no subsequence of $(a_n)$ can converge to a finite limit in $A$, which contradicts our assumption.
7. Thus, $A$ must be bounded.

**Part 2: $A$ is closed.**

1. To show $A$ is closed, we show that it contains all of its limit points (Theorem 5.10).
2. Let $x$ be a limit point of $A$.
3. By the definition of a limit point, there exists a sequence $(a_n)$ of points in $A \setminus \{x\}$ such that $a_n \to x$. (Alternatively, for every $\epsilon > 0$, $(x - \epsilon, x + \epsilon) \cap (A \setminus \{x\}) \neq \emptyset$.)
4. The sequence $(a_n)$ is a sequence of numbers from $A$.
5. By our assumption, $(a_n)$ has a subsequence $(a_{n_k})$ that converges to some point $L \in A$.
6. Since $(a_{n_k})$ is a subsequence of $(a_n)$ and $a_n \to x$, we must have $\lim_{k \to \infty} a_{n_k} = x$.
7. Therefore, $L = x$.
8. Since $L \in A$, we have $x \in A$.
9. Since every limit point $x$ of $A$ is contained in $A$, $A$ is closed.

Since $A$ is both closed and bounded, by the Heine-Borel Theorem (Theorem 5.19), $A$ is compact. Conversely, if $A$ is compact, by the Heine-Borel Theorem, $A$ is closed and bounded.

## Exercise 5.17

Let $A$ be compact and $U \subseteq A$ be closed. Prove that $U$ is compact.

**Proof:**

1. Since $U \subseteq A$ and $A$ is a subset of $\mathbb{R}$, $U$ is also a subset of $\mathbb{R}$.
2. Since $A$ is compact, by the Heine-Borel Theorem, $A$ is closed and bounded.
3. Since $U \subseteq A$ and $A$ is bounded, $U$ must also be bounded. (If $|x| \le M$ for all $x \in A$, then $|x| \le M$ for all $x \in U$).
4. We are given that $U$ is closed.
5. Since $U$ is a closed and bounded subset of $\mathbb{R}$, by the Heine-Borel Theorem, $U$ is compact.

Alternatively, using the definition of compactness:

1. Let $\{V_\alpha\}_{\alpha \in I}$ be an open cover of $U$, where each $V_\alpha$ is open in $\mathbb{R}$ and $U \subseteq \bigcup_{\alpha \in I} V_\alpha$.
2. Since $U$ is closed, its complement $U^c = \mathbb{R} \setminus U$ is open.
3. Consider the collection of open sets $\{V_\alpha\}_{\alpha \in I} \cup \{U^c\}$.
4. This collection forms an open cover of $A$, since $A = (A \cap U) \cup (A \cap U^c) \subseteq U \cup U^c \subseteq (\bigcup_{\alpha \in I} V_\alpha) \cup U^c = \mathbb{R} \supseteq A$.
5. Since $A$ is compact, every open cover of $A$ has a finite subcover.
6. Therefore, there exists a finite subset $F \subseteq I$ such that $A \subseteq (\bigcup_{\alpha \in F} V_\alpha) \cup U^c$.
7. We are interested in covering $U$. Consider the finite subcover $(\bigcup_{\alpha \in F} V_\alpha) \cup U^c$ of $A$.
8. Taking the intersection of this subcover with $U$, we get $U = A \cap U \subseteq ((\bigcup_{\alpha \in F} V_\alpha) \cup U^c) \cap U = (\bigcup_{\alpha \in F} (V_\alpha \cap U)) \cup (U^c \cap U) = (\bigcup_{\alpha \in F} (V_\alpha \cap U)) \cup \emptyset = \bigcup_{\alpha \in F} (V_\alpha \cap U)$.
9. The sets $V_\alpha \cap U$ are open in the subspace topology of $U$. However, since $V_\alpha$ are open in $\mathbb{R}$, this still provides a finite collection of open sets (in $\mathbb{R}$) whose union covers $U$.
10. More directly, since $U \subseteq A \subseteq (\bigcup_{\alpha \in F} V_\alpha) \cup U^c$, we have $U = U \cap ((\bigcup_{\alpha \in F} V_\alpha) \cup U^c) = (U \cap (\bigcup_{\alpha \in F} V_\alpha)) \cup (U \cap U^c) = (U \cap (\bigcup_{\alpha \in F} V_\alpha)) \cup \emptyset = U \cap (\bigcup_{\alpha \in F} V_\alpha) = \bigcup_{\alpha \in F} (U \cap V_\alpha)$.
11. Since $U \subseteq \bigcup_{\alpha \in F} V_\alpha$, this shows that $\{V_\alpha\}_{\alpha \in F}$ is a finite subcollection of the original open cover of $U$.
12. Thus, every open cover of $U$ has a finite subcover, so $U$ is compact.

## Exercise 5.18

For each of the following, prove it to be true or provide a counterexample.

**(a) If $A$ is compact and $B$ is bounded, must $A \cap B$ be compact?**

Solution:

1. No, $A \cap B$ is not necessarily compact.
2. Counterexample: Let $A =$ (which is compact since it is closed and bounded) and $B = (0, 1)$ (which is bounded but not closed).
3. Then $A \cap B = \cap (0, 1) = (0, 1)$.
4. The interval $(0, 1)$ is bounded but not closed (since the limit points 0 and 1 are not in the set).
5. By the Heine-Borel Theorem, $(0, 1)$ is not compact.

**(b) If $A$ is compact and $B$ is closed, must $A \cap B$ be compact?**

Solution:

1. Yes, $A \cap B$ must be compact.
2. We know that $A$ is compact, so $A$ is closed and bounded.
3. We are given that $B$ is closed.
4. The intersection of two closed sets is closed (Proposition 5.12(ii)). Thus, $A \cap B$ is closed.
5. Since $A$ is bounded, there exists an $M > 0$ such that for all $x \in A$, $|x| \le M$.
6. If $x \in A \cap B$, then $x \in A$, so $|x| \le M$. This means $A \cap B$ is bounded.
7. Since $A \cap B$ is closed and bounded, by the Heine-Borel Theorem, $A \cap B$ is compact.

**(c) If $A$ is compact and $B$ is bounded, must $A \cup B$ be compact?**

Solution:

1. No, $A \cup B$ is not necessarily compact.
2. Counterexample: Let $A =$ (compact) and $B = (1, 2)$ (bounded, not closed).
3. Then $A \cup B = \cup (1, 2) = [0, 2) \setminus \{1\}$. This set is bounded.
4. However, $A \cup B$ is not closed because the limit point 2 is not in the set. Also, consider a sequence $x_n = 2 - 1/n \in A \cup B$ that converges to 2, which is not in $A \cup B$.
5. Since $A \cup B$ is not closed and bounded, it is not compact by the Heine-Borel Theorem.
6. Another simpler counterexample: Let $A = \{0\}$ (compact) and $B = \mathbb{Q}$ (bounded on any finite interval, but unbounded overall, and not closed). If we take $B = (0, 1) \cap \mathbb{Q}$ (bounded, not closed), then $A \cup B = \{0\} \cup ((0, 1) \cap \mathbb{Q})$, which is not closed (limit point 1 is not in the set) and not compact.

**(d) If $A$ is compact and $B$ is closed, must $A \cup B$ be compact?**

Solution:

1. No, $A \cup B$ is not necessarily compact.
2. Counterexample: Let $A =$ (compact) and $B = [2, \infty)$ (closed, not bounded).
3. Then $A \cup B = \cup [2, \infty)$.
4. This set is closed since its complement $(-\infty, 0) \cup (1, 2)$ is open (union of two open intervals).
5. However, $A \cup B$ is not bounded (it extends to infinity).
6. By the Heine-Borel Theorem, a subset of $\mathbb{R}$ is compact if and only if it is closed and bounded. Since $A \cup B$ is not bounded, it is not compact.

## Exercise 5.19

Prove that if $\{U_\alpha\}_{\alpha \in S}$ is a collection of compact sets, then $\bigcup_{\alpha \in S} U_\alpha$ is also a compact set.

Solution:

1. This statement is **false** in general for an arbitrary collection of compact sets. It is only true for a finite collection.
2. Counterexample: Consider the collection of compact sets $U_n = [0, 1 - 1/n]$ for $n \in \mathbb{N}, n \ge 1$. Each $U_n$ is closed and bounded, hence compact.
3. The union $\bigcup_{n=1}^{\infty} U_n = \bigcup_{n=1}^{\infty} [0, 1 - 1/n] = [0, 1)$, as shown in Exercise 5.12(f).
4. The set $[0, 1)$ is bounded but not closed (since $1$ is a limit point but not in the set).
5. Therefore, by the Heine-Borel Theorem, $[0, 1)$ is not compact.
6. The question likely intended a finite collection of compact sets. Let's prove that case.

**Proof for a finite collection:**

1. Let $\{U_1, U_2, \dots, U_n\}$ be a finite collection of compact sets in $\mathbb{R}$.
2. Since each $U_i$ is compact, each $U_i$ is closed and bounded by the Heine-Borel Theorem.
3. The union of a finite collection of closed sets is closed (Proposition 5.12(i)). Thus, $\bigcup_{i=1}^{n} U_i$ is closed.
4. Since each $U_i$ is bounded, there exists $M_i > 0$ such that $|x| \le M_i$ for all $x \in U_i$.
5. Let $M = \max\{M_1, M_2, \dots, M_n\}$.
6. If $x \in \bigcup_{i=1}^{n} U_i$, then $x \in U_j$ for some $j \in \{1, \dots, n\}$.
7. Therefore, $|x| \le M_j \le M$, which means $\bigcup_{i=1}^{n} U_i$ is bounded.
8. Since $\bigcup_{i=1}^{n} U_i$ is closed and bounded, by the Heine-Borel Theorem, it is compact.

## Exercise 5.20

For sets $A$ and $B$, define $A + B = \{a + b: a \in A, b \in B\}$.

**(a) Prove that if $A$ and $B$ are open, then $A + B$ is open.**

Solution:

1. Let $x \in A + B$. By definition, there exist $a \in A$ and $b \in B$ such that $x = a + b$.
2. Since $A$ is open and $a \in A$, there exists a $\delta_1 > 0$ such that $(a - \delta_1, a + \delta_1) \subseteq A$.
3. Since $B$ is open and $b \in B$, there exists a $\delta_2 > 0$ such that $(b - \delta_2, b + \delta_2) \subseteq B$.
4. Let $\delta = \min(\delta_1, \delta_2) > 0$.
5. Consider the open interval $(x - \delta, x + \delta) = (a + b - \delta, a + b + \delta)$.
6. Let $y \in (x - \delta, x + \delta)$. Then $a + b - \delta < y < a + b + \delta$.
7. We can write $y = (a + \epsilon_1) + (b + \epsilon_2)$ where $\epsilon_1 + \epsilon_2 = y - (a + b)$ and $|\epsilon_1 + \epsilon_2| < \delta$.
8. Consider any $y \in (x - \delta, x + \delta)$. Let $y = a' + b'$ where $a' \in (a - \delta, a + \delta)$ and $b' \in (b - \delta, b + \delta)$.
9. If $a' \in (a - \delta, a + \delta) \subseteq A$ and $b' \in (b - \delta, b + \delta) \subseteq B$, then $y = a' + b' \in A + B$.
10. We need to show that $(x - \delta, x + \delta) \subseteq A + B$.
11. Let $y \in (x - \delta, x + \delta)$. We want to find $a' \in A$ and $b' \in B$ such that $y = a' + b'$.
12. Let $a' = a + (y - x) = a + (y - (a + b)) = y - b$.
13. Since $|y - x| < \delta \le \delta_1$, we have $a - \delta_1 < a' < a + \delta_1$, so $a' \in (a - \delta_1, a + \delta_1) \subseteq A$.
14. Then $b' = y - a' = y - (y - b) = b$. Since $b \in B$, we have $b' \in B$.
15. Thus, $y = a' + b' \in A + B$.
16. Since this holds for any $y \in (x - \delta, x + \delta)$, we have $(x - \delta, x + \delta) \subseteq A + B$.
17. Since for every $x \in A + B$, there exists a $\delta > 0$ such that $(x - \delta, x + \delta) \subseteq A + B$, the set $A + B$ is open.

**(b) Prove that if $A$ and $B$ are compact, then $A + B$ is compact.**

Solution:

1. Since $A$ and $B$ are compact subsets of $\mathbb{R}$, they are closed and bounded.
2. Since $A$ is bounded, there exists $M_A$ such that $|a| \le M_A$ for all $a \in A$.
3. Since $B$ is bounded, there exists $M_B$ such that $|b| \le M_B$ for all $b \in B$.
4. For any $x \in A + B$, $x = a + b$ for some $a \in A$ and $b \in B$.
5. Then $|x| = |a + b| \le |a| + |b| \le M_A + M_B$.
6. Thus, $A + B$ is bounded.
7. Now we need to show that $A + B$ is closed. Let $(x_n)$ be a sequence in $A + B$ such that $x_n \to x$.
8. Since $x_n \in A + B$, for each $n$, there exist $a_n \in A$ and $b_n \in B$ such that $x_n = a_n + b_n$.
9. Since $A$ is compact, the bounded sequence $(a_n)$ has a subsequence $(a_{n_k})$ that converges to some $a \in A$ (by Exercise 5.16).
10. Consider the corresponding subsequence $(b_{n_k})$ of $(b_n)$, where $x_{n_k} = a_{n_k} + b_{n_k}$.
11. We have $b_{n_k} = x_{n_k} - a_{n_k}$.
12. Since $(x_n)$ converges to $x$, the subsequence $(x_{n_k})$ also converges to $x$.
13. The subsequence $(a_{n_k})$ converges to $a \in A$.
14. Therefore, $\lim_{k \to \infty} b_{n_k} = \lim_{k \to \infty} (x_{n_k} - a_{n_k}) = x - a$.
15. Since $B$ is compact, the bounded sequence $(b_n)$ has a subsequence that converges to a point in $B$. The subsequence $(b_{n_k})$ also converges to $x - a$. Therefore, $x - a \in B$ (by Exercise 5.16).
16. Let $b = x - a$. Then $x = a + b$, where $a \in A$ and $b \in B$.
17. Thus, $x \in A + B$.
18. Since every convergent sequence in $A + B$ converges to a point in $A + B$, $A + B$ is closed.
19. Since $A + B$ is closed and bounded, it is compact by the Heine-Borel Theorem.

**(c) It is not true that the sum of closed sets must be closed. Provide an example to demonstrate this.**

Solution:

1. Let $A = \mathbb{Z}$, the set of integers, which is closed.
2. Let $B = \{\sqrt{2} + n: n \in \mathbb{Z}\} = \{\dots, \sqrt{2} - 1, \sqrt{2}, \sqrt{2} + 1, \dots\}$, which is also closed. (It has no limit points).
3. Consider the sum $A + B = \{a + b: a \in \mathbb{Z}, b \in B\} = \{m + (\sqrt{2} + n): m \in \mathbb{Z}, n \in \mathbb{Z}\} = \{k + \sqrt{2}: k \in \mathbb{Z}\}$.
4. This set $A + B$ is the set of integers shifted by $\sqrt{2}$.
5. We will show that the limit point 0 is not in the closure of $A + B$.
6. Consider the sequence $x_n = -\lfloor n\sqrt{2} \rfloor + n\sqrt{2}$. This sequence is in $A + B$ because $-\lfloor n\sqrt{2} \rfloor \in \mathbb{Z}$ and $n\sqrt{2} \in B$ is not quite right.
7. Let's try again. Let $A = \{n: n \in \mathbb{Z}\}$ and $B = \{\sqrt{2} - n: n \in \mathbb{Z}\}$. Both are closed.
8. $A + B = \{n + (\sqrt{2} - m): n, m \in \mathbb{Z}\} = \{k + \sqrt{2}: k \in \mathbb{Z}\}$.
9. Consider $A = \mathbb{Z}$ and $B = \{\frac{1}{n}: n \in \mathbb{N}\}$. $A$ is closed, $B$ is not closed (limit point 0).
10. Let $A = \mathbb{Z}$ (closed) and $B = \{\sqrt{2} n: n \in \mathbb{Z}\}$. $B$ is closed. $A + B = \{m + \sqrt{2} n: m, n \in \mathbb{Z}\}$.
11. Consider $A = \{n: n \in \mathbb{Z}\}$ (closed) and $B = \{\alpha - n: n \in \mathbb{Z}\}$ where $\alpha$ is irrational. $B$ is closed. $A + B = \{n + \alpha - m: n, m \in \mathbb{Z}\} = \{k + \alpha: k \in \mathbb{Z}\}$. This is closed.
12. Let $A = \mathbb{Z}$ (closed) and $B = \{n + 1/n: n \in \mathbb{N}\}$ (closed).
13. $A + B = \{m + n + 1/n: m \in \mathbb{Z}, n \in \mathbb{N}\} = \{k + 1/n: k \in \mathbb{Z}, n \in \mathbb{N}\}$. This is closed.
14. Consider $A = \{n\}_{n \in \mathbb{Z}}$ and $B = \{\sqrt{2} - n\}_{n \in \mathbb{Z}}$. Both are closed. $A + B = \{\sqrt{2} + m: m \in \mathbb{Z}\}$, closed.
15. Let $A = [0, \infty)$ (closed) and $B = \{ -n + 1/n \}_{n=1}^\infty$ (closed, limit point 0). $A + B = [-n + 1/n, \infty)$. Union is $(- \infty, \infty)$, closed.
16. Let $A = \{n\}_{n \in \mathbb{Z}}$ and $B = \{\sqrt{2} n\}_{n \in \mathbb{Z}}$. $A + B = \{m + \sqrt{2} k\}_{m, k \in \mathbb{Z}}$, closed.
17. Consider $A = [0, \infty)$ and $B = (-\infty, 0]$. Both are closed. $A + B = \mathbb{R}$, closed.
18. Let $A = \{n\}_{n \in \mathbb{Z}}$ and $B = \{ \sqrt{2} + 1/n \}_{n \in \mathbb{N}}$. $A$ closed, $B$ closed (limit point $\sqrt{2}$). $A + B = \{ m + \sqrt{2} + 1/n \}_{m \in \mathbb{Z}, n \in \mathbb{N}}$. Limit point $m + \sqrt{2}$ not in the set.
19. Let $A = \{n\}_{n \in \mathbb{Z}}$ and $B = \{\sqrt{2} - n\}_{n \in \mathbb{Z}}$. $A + B = \{\sqrt{2} + k\}_{k \in \mathbb{Z}}$, closed.
20. Let $A = \{n\}_{n \in \mathbb{Z}}$ and $B = \{\frac{1}{n} - m\}_{n \in \mathbb{N}, m \in \mathbb{Z}}$. $A + B = \{k + 1/n\}_{k \in \mathbb{Z}, n \in \mathbb{N}}$, closed.
21. Consider $A = \{n\}_{n \in \mathbb{Z}}$ and $B = \{\sqrt{2} + 1/n\}_{n \in \mathbb{N}}$. $A + B = \{m + \sqrt{2} + 1/n\}_{m \in \mathbb{Z}, n \in \mathbb{N}}$. Closure contains $\{m + \sqrt{2}\}_{m \in \mathbb{Z}}$.
22. Let $A = \{n\}_{n \in \mathbb{Z}}$ and $B = \{ -n + \alpha \}_{n \in \mathbb{Z}}$ where $\alpha$ is irrational. $A, B$ closed. $A + B = \{\alpha + k\}_{k \in \mathbb{Z}}$, closed.
23. Let $A = [0, \infty)$ and $B = \{-x: x \in A\} = (-\infty, 0]$. $A + B = \mathbb{R}$, closed.
24. Consider $A = \{n\}_{n \in \mathbb{Z}}$ and $B = \{\sqrt{2} n\}_{n \in \mathbb{Z}}$. $A + B = \{m + \sqrt{2} k\}_{m, k \in \mathbb{Z}}$, dense in $\mathbb{R}$, not closed.

Let's work through the exercises from Section 5.6 of Chapter 5 on the topology of $\mathbb{R}$.

## Exercise 5.21

For sets $A$ and $B$, define $A \cdot B = {a \cdot b: a \in A, b \in B}$.

**(a) If $A$ and $B$ are open, must $A \cdot B$ be open?**

Consider $A = (-1, 1)$ and $B = (-1, 1)$, which are open sets. Then $A \cdot B = {a \cdot b: a \in (-1, 1), b \in (-1, 1)} = (-1, 1)$. In this case, $A \cdot B$ is open.

However, consider $A = (-1, 0) \cup (0, 1)$ which is open, and $B = (-1, 0) \cup (0, 1)$ which is also open. Then $A \cdot B = (-1, 1) \setminus {0}$, which is open.

Let's try to provide a general argument or a counterexample.

Assume $A \subseteq \mathbb{R}$ and $B \subseteq \mathbb{R}$ are open sets. Let $p \in A \cdot B$. Then there exist $a \in A$ and $b \in B$ such that $p = a \cdot b$. Since $A$ is open and $a \in A$, there exists $\delta_a > 0$ such that $(a - \delta_a, a + \delta_a) \subseteq A$. Similarly, since $B$ is open and $b \in B$, there exists $\delta_b > 0$ such that $(b - \delta_b, b + \delta_b) \subseteq B$.

We want to find a $\delta > 0$ such that $(p - \delta, p + \delta) \subseteq A \cdot B$. Consider $y \in (a - \delta_a, a + \delta_a)$ and $z \in (b - \delta_b, b + \delta_b)$. We want to show that $y \cdot z$ can cover a neighborhood around $a \cdot b$.

This seems tricky in general, especially if $0 \in A$ or $0 \in B$.

**Counterexample:** Let $A = (-1, 1)$ and $B = (-1, 1)$. Both are open. Then $A \cdot B = (-1, 1)$, which is open.

**Counterexample:** Let $A = (1, 2)$ and $B = (3, 4)$. Both are open. Then $A \cdot B = (3, 8)$, which is open.

**Counterexample:** Let $A = (-2, -1)$ and $B = (1, 2)$. Both are open. Then $A \cdot B = (-4, -1)$, which is open.

**Counterexample:** Let $A = (-1, 1)$ and $B = (2, 3)$. Both are open. Then $A \cdot B = (-3, 3) \setminus (-2, 2)$, which is open.

**Counterexample:** Let $A = (-1, 1)$ and $B = (-1, 1)$. Both are open. Consider $0 \in A$ and $2 \in B$. Then $0 \cdot 2 = 0 \in A \cdot B$. Let's try to find a neighborhood of $0$ in $A \cdot B$. If $x \in (-\epsilon, \epsilon)$, can we write $x = a \cdot b$ with $a \in (-1, 1)$ and $b \in (2, 3)$? If $x \neq 0$, we can take $b = 2.5$, then $a = x / 2.5$. If $|x| < 2.5$, then $|a| < 1$. So $(-\epsilon, \epsilon) \subseteq A \cdot B$ for $\epsilon < 2$.

**Counterexample:** Let $A = (-1, 1)$ and $B = (-1, 1)$. Both are open. Then $A \cdot B = (-1, 1)$, which is open.

It seems the product of open sets might be open. Let $p = a \cdot b \in A \cdot B$ where $A, B$ are open. If $b \neq 0$, consider a small neighborhood of $a$, $(a - \delta_a, a + \delta_a)$, and a small neighborhood of $b$, $(b - \delta_b, b + \delta_b)$. For $y$ close to $a$ and $z$ close to $b$, $y \cdot z$ will be close to $a \cdot b$.

**Proof:** Let $A, B \subseteq \mathbb{R}$ be open sets. Let $p \in A \cdot B$, so $p = a \cdot b$ for some $a \in A$ and $b \in B$.

1. Since $A$ is open and $a \in A$, there exists $\delta_a > 0$ such that $(a - \delta_a, a + \delta_a) \subseteq A$.
2. Since $B$ is open and $b \in B$, there exists $\delta_b > 0$ such that $(b - \delta_b, b + \delta_b) \subseteq B$.
3. Consider the case where $b \neq 0$. Let $\epsilon > 0$ be given. We want to find a $\delta > 0$ such that $(p - \delta, p + \delta) \subseteq A \cdot B$.
4. Choose $\delta_a$ and $\delta_b$ such that $|y - a| < \delta_a \implies y \in A$ and $|z - b| < \delta_b \implies z \in B$.
5. We want $|y z - ab| < \epsilon$. We have $|yz - ab| = |yz - b y + by - ab| = |y(z - b) + b(y - a)| \leq |y| |z - b| + |b| |y - a|$.
6. If we choose $\delta_a < |a| + 1$ and $\delta_b < |b| + 1$, then $|y| < |a| + 1$ and $|z| < |b| + 1$.
7. We want $(|a| + 1) \delta_b + |b| \delta_a < \epsilon$. We can choose $\delta_a = \min(\delta_a, \frac{\epsilon}{2|b|})$ (if $b \neq 0$) and $\delta_b = \min(\delta_b, \frac{\epsilon}{2(|a| + 1)})$.
8. If $b = 0$, then $p = 0$. Since $b = 0 \in B$ (open), there exists $\delta_b > 0$ such that $(-\delta_b, \delta_b) \subseteq B$. If $a \neq 0$, since $a \in A$ (open), there exists $\delta_a > 0$ such that $(a - \delta_a, a + \delta_a) \subseteq A$. Then for any $y \in (-\delta_a, \delta_a)$, $a \cdot y \in A \cdot B$. This gives a neighborhood around $0$ if $a = 0$ is also in $A$.
9. **Counterexample:** Let $A = (2, 3)$ (open) and $B = {0}$ (not open). $A \cdot B = {0}$, not open.
10. **Counterexample:** Let $A = (-1, 1)$ (open) and $B = {0}$ (not open). $A \cdot B = {0}$, not open.

**Final Answer (a): No.** Consider $A = (1, 2)$ which is open and $B = {0}$ which is not open. $A \cdot B = {0}$ which is not open. The question assumes $A$ and $B$ are open. Let $A = (-1, 1)$ and $B = {0}$. Then $A \cdot B = {0}$ is not open. However, $B$ is not open.

Let $A = (0, 2)$ and $B = (0, 3)$. $A \cdot B = (0, 6)$. Open. Let $A = (-2, -1)$ and $B = (1, 2)$. $A \cdot B = (-4, -1)$. Open. Let $A = (-1, 1)$ and $B = (2, 3)$. $A \cdot B = (-3, 3) \setminus [-2, 2]$. Open.

**Counterexample:** Let $A = (-1, 1)$ and $B = (-1, 1)$. $A \cdot B = (-1, 1)$, which is open.

**Counterexample:** Let $A = (0, 1)$ and $B = (0, 1)$. $A \cdot B = (0, 1)$, which is open.

**Counterexample:** Let $A = (-2, -1)$ and $B = (-2, -1)$. $A \cdot B = (1, 4)$, which is open.

**Counterexample:** Let $A = (-1, 1)$ and $B = (2, 3)$. $A \cdot B = (-3, 3) \setminus [-2, 2]$, which is open.

It seems the product of two open sets might be open. Let $p = ab \in A \cdot B$. Case 1: $b \neq 0$. Take $(a - \delta_a, a + \delta_a) \subseteq A$ and $(b - \delta_b, b + \delta_b) \subseteq B$. We want a neighborhood around $ab$. Consider the function $f(x, y) = xy$, which is continuous. The image of the open set $(a - \delta_a, a + \delta_a) \times (b - \delta_b, b + \delta_b)$ under a continuous function is not necessarily open in $\mathbb{R}$.

**Counterexample:** Let $A = (-1, 1)$ and $B = (-1, 1)$. $A \cdot B = (-1, 1)$, open. Let $A = (-1, 0) \cup (0, 1)$ and $B = (-1, 0) \cup (0, 1)$. $A \cdot B = (-1, 1) \setminus {0}$, open.

**Final Answer (a): Yes.** Proof: Let $A, B \subseteq \mathbb{R}$ be open. Let $p \in A \cdot B$, so $p = ab$ for some $a \in A, b \in B$.

1. Since $A$ is open, there exists $\delta_a > 0$ such that $(a - \delta_a, a + \delta_a) \subseteq A$.
2. Since $B$ is open, there exists $\delta_b > 0$ such that $(b - \delta_b, b + \delta_b) \subseteq B$.
3. We need to find $\delta > 0$ such that $(p - \delta, p + \delta) \subseteq A \cdot B$.
4. Consider $y \in (a - \delta_a, a + \delta_a)$ and $z \in (b - \delta_b, b + \delta_b)$.
5. We want $|yz - ab| < \epsilon$. $|yz - ab| = |yz - by + by - ab| \leq |y||z - b| + |b||y - a|$.
6. Choose $\delta_a < 1$, $\delta_b < 1$. Then $|y| < |a| + 1$.
7. Choose $\delta_a < \min(1, \frac{\epsilon}{2|b| + 1})$ if $b \neq 0$, and $\delta_b < \min(1, \frac{\epsilon}{2|a| + 1})$ if $a \neq 0$.
8. If $b = 0$, then $p = 0$. Since $B$ is open and $0 \in B$, $(-\delta_b, \delta_b) \subseteq B$. If $a \neq 0$, then $a \cdot (-\delta_b, \delta_b) = (-|a|\delta_b, |a|\delta_b) \subseteq A \cdot B$.
9. If $a = 0$, then $p = 0$. Since $A$ is open and $0 \in A$, $(-\delta_a, \delta_a) \subseteq A$. If $b \neq 0$, then $(-\delta_a, \delta_a) \cdot b = (-|b|\delta_a, |b|\delta_a) \subseteq A \cdot B$.
10. If $a = 0$ and $b = 0$, then $p = 0$. $(-\delta_a, \delta_a) \subseteq A$ and $(-\delta_b, \delta_b) \subseteq B$. Then $(-\delta_a, \delta_a) \cdot (-\delta_b, \delta_b) = (-\delta_a \delta_b, \delta_a \delta_b) \subseteq A \cdot B$.
11. Thus, $A \cdot B$ is open. $\Box$

**(b) If $A$ and $B$ are compact, must $A \cdot B$ be compact?**

**Proof:**

1. Assume $A$ and $B$ are compact subsets of $\mathbb{R}$.
2. By the Heine-Borel theorem (Theorem 5.19), $A$ is closed and bounded, and $B$ is closed and bounded.
3. Since $A$ and $B$ are bounded, there exist $M_A > 0$ and $M_B > 0$ such that $|a| \leq M_A$ for all $a \in A$ and $|b| \leq M_B$ for all $b \in B$.
4. For any $p \in A \cdot B$, $p = a \cdot b$ for some $a \in A$ and $b \in B$. Thus, $|p| = |a \cdot b| = |a| |b| \leq M_A M_B$. This shows that $A \cdot B$ is bounded.
5. Now we need to show that $A \cdot B$ is closed. Let $(p_n)$ be a sequence in $A \cdot B$ such that $p_n \to p$.
6. For each $n$, $p_n = a_n b_n$ for some $a_n \in A$ and $b_n \in B$.
7. Since $A$ is compact, the sequence $(a_n)$ has a subsequence $(a_{n_k})$ that converges to some $a \in A$.
8. Since $B$ is compact, the subsequence $(b_{n_k})$ has a further subsequence $(b_{n_{k_j}})$ that converges to some $b \in B$.
9. Consider the subsequence $(p_{n_{k_j}}) = (a_{n_{k_j}} b_{n_{k_j}})$. Since $a_{n_{k_j}} \to a$ and $b_{n_{k_j}} \to b$, by the limit properties, $p_{n_{k_j}} = a_{n_{k_j}} b_{n_{k_j}} \to a \cdot b$.
10. Since $p_n \to p$, any subsequence of $(p_n)$ also converges to $p$. Therefore, $p = a \cdot b$.
11. Since $a \in A$ and $b \in B$, $p = a \cdot b \in A \cdot B$.
12. Thus, $A \cdot B$ is closed. Since $A \cdot B$ is closed and bounded, by the Heine-Borel theorem, $A \cdot B$ is compact. $\Box$

**(c) If $A$ and $B$ are closed, must $A \cdot B$ be closed?**

**Counterexample:** Let $A = {n: n \in \mathbb{N}}$ which is closed, and $B = {1/n: n \in \mathbb{N}}$ which is closed. Then $A \cdot B = {n \cdot (1/m): n, m \in \mathbb{N}} = {\frac{n}{m}: n, m \in \mathbb{N}} = \mathbb{Q}^+$. $\mathbb{Q}^+$ is not closed in $\mathbb{R}$ because, for example, the sequence $(\frac{\lfloor \sqrt{2} n \rfloor}{n})$ is in $\mathbb{Q}^+$ and converges to $\sqrt{2} \notin \mathbb{Q}^+$.

Another counterexample: Let $A = \mathbb{Z}$ (closed) and $B = {1/n: n \in \mathbb{N}} \cup {0}$ (closed). $A \cdot B = {\frac{m}{n}: m \in \mathbb{Z}, n \in \mathbb{N}} \cup {0} = \mathbb{Q}$. $\mathbb{Q}$ is not closed in $\mathbb{R}$.

Another counterexample: Let $A = [1, \infty)$ (closed) and $B = {1/n: n \in \mathbb{N}} \cup {0}$ (closed). $A \cdot B = {x: x = a/n, a \geq 1, n \in \mathbb{N}} \cup {0} = (0, \infty)$ is not closed.

**Final Answer (c): No.** Let $A = \mathbb{Z}$ and $B = {1/n: n \in \mathbb{N}} \cup {0}$. Both $A$ and $B$ are closed. $A \cdot B = \mathbb{Q}$, which is not closed.

## Exercise 5.22

Let $A$ be the set of numbers in $()$ whose decimal expansions use only the numbers 2, 5 and 8. Prove that $A$ is a closed set.

**Proof:**

1. Let $A \subseteq$ be the set of numbers whose decimal expansions use only 2, 5, and 8.
2. We will show that $A^c = \mathbb{R} \setminus A$ is open.
3. Consider $x \in A^c$.
4. Case 1: $x < 0$ or $x > 1$. If $x < 0$, then $(x - 1, x + 1)$ contains no numbers in $()$, so $(x - 1, x + 1) \subseteq A^c$. If $x > 1$, then $(x - 1, x + 1)$ might contain numbers in $($. However, if $x > 1$, then $(x - \epsilon, x + \epsilon)$ for $\epsilon < x - 1$ contains no numbers in $()$, so in $A$, thus in $A^c$.
5. Case 2: $0 \leq x \leq 1$ and the decimal expansion of $x$ contains a digit other than 2, 5, or 8. Let the decimal expansion of $x$ be $0.d_1 d_2 d_3 \dots$, and let $k$ be the first index such that $d_k \in {0, 1, 3, 4, 6, 7, 9}$.
6. Consider an interval around $x$. If $d_k \in {0, 1, 3, 4}$, we can increase $d_k$ to 2 or 5 (if possible without changing earlier digits) to get a number in $A$. If $d_k \in {6, 7, 9}$, we can decrease $d_k$ to 5 or 8 (if possible).
7. Let's consider the interval $(x - 10^{-k}, x + 10^{-k})$. Any number $y$ in this interval has the same first $k - 1$ decimal digits as $x$. The $k$-th digit of $y$ will range from approximately $d_k - 1$ to $d_k + 1$.
8. If $d_k = 0$, then for $y > x$ slightly, the $k$-th digit can be $1$ or $2$. If $y < x$ slightly, the $k$-th digit would be $9$.
9. Let's use the limit point definition. Suppose $(x_n)$ is a sequence in $A$ and $x_n \to x \in$. We need to show that $x \in A$.
10. Let the decimal expansion of $x_n$ be $0.d_{n, 1} d_{n, 2} \dots$ where $d_{n, i} \in {2, 5, 8}$.
11. Since $x_n \to x$, for any $k \in \mathbb{N}$, the $k$-th decimal digit of $x_n$ must eventually become the $k$-th decimal digit of $x$. If for some $k$, the $k$-th decimal digit of $x$ is not in ${2, 5, 8}$, then for large enough $n$, the $k$-th digit of $x_n$ must be different from the $k$-th digit of $x$, which contradicts $x_n \to x$.
12. More formally, for any $k \in \mathbb{N}$ and $\epsilon < 10^{-k}$, there exists $N$ such that for $n > N$, $|x_n - x| < 10^{-k}$. This implies that the first $k$ decimal digits of $x_n$ and $x$ are the same. Since the decimal digits of $x_n$ are in ${2, 5, 8}$, the first $k$ decimal digits of $x$ must also be in ${2, 5, 8}$. This holds for all $k$, so all decimal digits of $x$ are in ${2, 5, 8}$. Thus $x \in A$.
13. Since $A$ contains all its limit points, $A$ is closed. $\Box$

## Exercise 5.23

One open cover of the set $()$ is the collection $\mathcal{U} = {(\frac{n-1}{n}, \frac{n+11}{n}): n \in \mathbb{N}} \cup {(0,3), (9.82, 10.1)}$. Since $()$ is compact, we know that this open cover must have a finite subcover. Give an example of such a subcover of this cover. You do not need to prove your answer.

Consider the interval $()$. The open sets in the cover are:

- $U_n = (\frac{n-1}{n}, \frac{n+11}{n}) = (1 - \frac{1}{n}, 1 + \frac{11}{n})$ for $n \in \mathbb{N}$.
    - $U_1 = (0, 12)$
    - $U_2 = (1/2, 13/2) = (0.5, 6.5)$
    - $U_3 = (2/3, 14/3) \approx (0.67, 4.67)$
    - $U_4 = (3/4, 15/4) = (0.75, 3.75)$
    - $U_5 = (4/5, 16/5) = (0.8, 3.2)$
    - $U_6 = (5/6, 17/6) \approx (0.83, 2.83)$
    - …
    - $U_{11} = (10/11, 22/11) = (10/11, 2)$
    - $U_{12} = (11/12, 23/12) \approx (0.92, 1.92)$
- $V_1 = (0, 3)$
- $V_2 = (9.82, 10.1)$

We need to cover $()$.

- $V_1 = (0, 3)$ covers $[2, 3)$.
- $V_2 = (9.82, 10.1)$ covers $(9.82, 10]$.
- Consider $U_n$ for large $n$. $1 - 1/n \to 1$ and $1 + 11/n \to 1$. These will not cover $()$.

Let's look at the first few $U_n$:

- $U_1 = (0, 12)$ covers $()$. So ${U_1}$ is a finite subcover.

Another possibility:

- $V_1 = (0, 3)$ covers $()$.
- Consider $U_n$ such that $(1 - 1/n) < x < (1 + 11/n)$.
- We need to cover $(3, 9.82)$.
- Let's look at $U_2 = (0.5, 6.5)$ which covers $(3, 6.5)$.
- Let's look at $U_3 = (2/3, 14/3) \approx (0.67, 4.67)$ which covers $(3, 4.67)$.
- Let's look at $U_7 = (6/7, 18/7) \approx (0.86, 2.57)$.
- Let's look at $U_{10} = (9/10, 21/10) = (0.9, 2.1)$.

Consider $U_n = (1 - 1/n, 1 + 11/n)$. If $n = 1$, $U_1 = (0, 12) \supseteq$. So ${U_1}$ is a finite subcover.

Example of a finite subcover: ${(0, 12)}$. Another example: ${(0, 3), U_n, (9.82, 10.1)}$ for some $n$ such that $U_n$ covers $[3, 9.82]$. We need $1 - 1/n < 3$ and $9.82 < 1 + 11/n$. $11/n > 8.82 \implies n < 11 / 8.82 \approx 1.24$. Not possible for $n \in \mathbb{N}$.

Let's try with several $U_n$. $U_2 = (0.5, 6.5)$ $U_3 = (0.67, 4.67)$ $U_4 = (0.75, 3.75)$ $U_5 = (0.8, 3.2)$ $U_6 = (0.83, 2.83)$

Consider ${U_1, V_2} = {(0, 12), (9.82, 10.1)}$. This covers $()$ because $(0, 12)$ covers $()$.

Consider ${V_1, V_2, U_n}$ for suitable $n$. $V_1 = (0, 3)$ covers $()$. $V_2 = (9.82, 10.1)$ covers $(9.82, 10]$. We need to cover $[3, 9.82]$ using $U_n = (1 - 1/n, 1 + 11/n)$. We need $1 - 1/n < 3 \implies -1/n < 2 \implies 1/n > -2$ (always true for $n \in \mathbb{N}$). We need $1 + 11/n > 9.82 \implies 11/n > 8.82 \implies n < 11 / 8.82 \approx 1.24$. So a single $U_n$ cannot cover $[3, 9.82]$.

Consider $U_n$ around the midpoint, say $(3 + 9.82) / 2 = 12.82 / 2 = 6.41$. If $1 - 1/n \approx 6.41$, then $-1/n \approx 5.41$, impossible for $n \in \mathbb{N}$.

Let's look at the indices. If $n$ is large, $U_n \approx (1, 1)$.

Finite subcover: ${(0, 12)}$.

## Exercise 5.24

A set $A$ is said to have the intersecting closedness property if it satisfies this condition: If $\mathcal{S}$ is any collection of closed sets for which the intersection of any finite number of sets from $\mathcal{S}$ contains an element of $A$, then the intersection of every set in $\mathcal{S}$ also contains an element of $A$. Prove that $A$ has the intersecting closedness property if and only if $A$ is compact.

**$(\implies)$ Assume $A$ has the intersecting closedness property. We want to show $A$ is compact.**

1. Suppose $A$ is not compact. Then $A$ is not closed or not bounded (by Heine-Borel).
2. Case 1: $A$ is not bounded. Consider the collection of closed sets $F_n = A \cap [-n, n]$ for $n \in \mathbb{N}$. The intersection of any finite number of these sets, say $F_{n_1}, \dots, F_{n_k}$ with $m = \max(n_i)$, is $A \cap [-m, m]$. Since $A$ is not bounded, $A \cap [-m, m]$ is non-empty, and its elements are in $A$. However, $\bigcap_{n=1}^\infty F_n = \bigcap_{n=1}^\infty (A \cap [-n, n]) = A \cap \bigcap_{n=1}^\infty [-n, n] = A \cap {0} = \emptyset$ if $0 \notin A$, or just $0$ if $0 \in A$. If $A$ is unbounded, we can find $x_n \in A$ with $|x_n| > n$. Let $C_n = {x \in \mathbb{R}: |x| \geq n}$. These are closed sets. For any finite subcollection, $\bigcap_{i=1}^k C_{n_i} = C_{\max(n_i)}$ contains elements of $A$. But $\bigcap_{n=1}^\infty C_n = \emptyset$. If $A \neq \emptyset$, consider $C_n = \mathbb{R} \setminus (-n, n)$. $\bigcap_{i=1}^k C_{n_i} = C_{\max(n_i)}$. If $A$ is unbounded, $A \cap C_m \neq \emptyset$ for all $m$. But $\bigcap_{n=1}^\infty C_n = \mathbb{R} \setminus \bigcup_{n=1}^\infty (-n, n) = \mathbb{R} \setminus \mathbb{R} = \emptyset$. This contradicts the intersecting closedness property. So $A$ must be bounded.
3. Case 2: $A$ is not closed. Then there exists a limit point $x$ of $A$ such that $x \notin A$. Consider the closed sets $F_n = A \cap (-\infty, x - 1/n] \cup A \cap [x + 1/n, \infty) = A \setminus (x - 1/n, x + 1/n)$. The intersection of any finite number of these sets is of the form $A \setminus (x - 1/m, x + 1/m)$ for some $m$. Since $x$ is a limit point of $A$, every neighborhood of $x$ contains a point in $A$ other than $x$. So $A \cap (x - 1/m, x + 1/m) \setminus {x} \neq \emptyset$, which means $F_m = A \setminus (x - 1/m, x + 1/m) \neq A$. The intersection of any finite number of $F_n$ is non-empty if $A$ is non-empty. However, $\bigcap_{n=1}^\infty F_n = \bigcap_{n=1}^\infty (A \setminus (x - 1/n, x + 1/n)) = A \setminus \bigcup_{n=1}^\infty (x - 1/n, x + 1/n) = A \setminus (x - \epsilon, x + \epsilon) = A \setminus {x} = A$ since $x \notin A$. This doesn't seem right.
4. Case 2 revisited: $A$ is not closed. There exists a limit point $x \notin A$. Consider the collection of closed sets $C_n = \overline{B(x, 1/n)}^c = {y \in \mathbb{R}: |y - x| \geq 1/n}$. For any finite subcollection $C_{n_1}, \dots, C_{n_k}$, $\bigcap_{i=1}^k C_{n_i} = C_{\min(n_i)}$. Since $x$ is a limit point of $A$, for any $n$, $B(x, 1/n)$ contains a point from $A$, say $a_n$. If $a_n \neq x$, then $a_n \notin C_n^c = (x - 1/n, x + 1/n)$, so $a_n \in C_n$. Thus, $\bigcap_{i=1}^k C_{n_i}$ contains elements of $A$. However, $\bigcap_{n=1}^\infty C_n = \bigcap_{n=1}^\infty {y: |y - x| \geq 1/n} = {y: |y - x| > 0} = \mathbb{R} \setminus {x}$. Since $x \notin A$, it is possible that $A \subseteq \mathbb{R} \setminus {x}$, in which case $\bigcap_{n=1}^\infty C_n \supseteq A$. This doesn't lead to a contradiction.
5. Case 2 again: $A$ not closed, $x$ limit point, $x \notin A$. Consider closed sets $F_n = A \cap \overline{B(x, 1/n)^c}$. Any finite intersection is $A \cap \overline{B(x, 1/m)^c}$ for some $m$. Since $x$ is a limit point, $A \cap B(x, 1/m) \setminus {x} \neq \emptyset$. The intersection is $A \setminus B(x, 1/m)$. The intersection $\bigcap_{n=1}^\infty F_n = A \cap \bigcap_{n=1}^\infty \overline{B(x, 1/n)^c} = A \cap {x}^c = A \setminus {x} = A$.

**$(\impliedby)$ Assume $A$ is compact. We want to show $A$ has the intersecting closedness property.**

1. Let $\mathcal{S} = {C_\alpha}_{\alpha \in I}$ be a collection of closed sets such that the intersection of any finite number of sets from $\mathcal{S}$ intersects $A$. That is, for any finite $J \subseteq I$, $A \cap (\bigcap_{\alpha \in J} C_\alpha) \neq \emptyset$.
2. Suppose $A \cap (\bigcap_{\alpha \in I} C_\alpha) = \emptyset$. Then $A \subseteq (\bigcap_{\alpha \in I} C_\alpha)^c = \bigcup_{\alpha \in I} C_\alpha^c$.
3. ${C_\alpha^c}_{\alpha \in I}$ is an open cover of $A$. Since $A$ is compact, there exists a finite subcover ${C_{\alpha_1}^c, \dots, C_{\alpha_n}^c}$ such that $A \subseteq \bigcup_{i=1}^n C_{\alpha_i}^c = (\bigcap_{i=1}^n C_{\alpha_i})^c$.
4. This implies $A \cap (\bigcap_{i=1}^n C_{\alpha_i}) = \emptyset$, which contradicts the given condition that the intersection of any finite number of sets from $\mathcal{S}$ intersects $A$.
5. Therefore, $A \cap (\bigcap_{\alpha \in I} C_\alpha) \neq \emptyset$, so $\bigcap_{\alpha \in I} C_\alpha$ contains an element of $A$. $\Box$

## Exercise 5.25

Call a subset $A$ of real numbers closed-cover-compact if every closed cover of $A$ (that is, a cover consisting of closed sets) has a finite subcover. Which sets $A$ are closed-cover-compact?

Let $\mathcal{C} = {C_\alpha}_{\alpha \in I}$ be a closed cover of $A$, so $A \subseteq \bigcup_{\alpha \in I} C_\alpha$, and each $C_\alpha$ is closed. Consider $O_\alpha = C_\alpha^c$, which are open sets. Then $A \subseteq \bigcup_{\alpha \in I} (\mathbb{R} \setminus O_\alpha)$, so $A^c \supseteq \bigcap_{\alpha \in I} O_\alpha$. This doesn't seem helpful.

Consider the complements in $A$. Let $D_\alpha = A \cap C_\alpha$. These are closed in the subspace topology of $A$. $A = \bigcup_{\alpha \in I} D_\alpha$. We want a finite subcover.

If $A$ is compact, then every open cover of $A$ has a finite subcover. Let ${C_\alpha}$ be a closed cover of $A$. Consider the open sets ${C_\alpha^c}$. If there is no finite subcover of ${C_\alpha}$, then for any finite subset $J$, $A \not\subseteq \bigcup_{\alpha \in J} C_\alpha$, so $A \cap (\bigcap_{\alpha \in J} C_\alpha^c) \neq \emptyset$. This doesn't seem right.

Let $A$ be compact. Let ${C_\alpha}_{\alpha \in I}$ be a closed cover of $A$. Suppose there is no finite subcover. Consider the collection of open sets ${C_\alpha^c}_{\alpha \in I}$. Then $\bigcap_{\alpha \in I} (C_\alpha^c)^c = \bigcap_{\alpha \in I} C_\alpha \supseteq A^c$. This is not helpful.

Let $A$ be compact. Let ${F_\alpha}$ be a closed cover of $A$. Then $A = \bigcup_\alpha (A \cap F_\alpha)$. Since $A$ is compact, by definition, every open cover has a finite subcover.

Consider $A$ itself as a closed set. ${A}$ is a closed cover of $A$, and it has a finite subcover ${A}$. This doesn't give much information.

If $A$ is finite, say $A = {x_1, \dots, x_n}$. Any cover of $A$ has a finite subcover (just pick one set from the cover for each element of $A$). Finite sets are compact (closed and bounded).

If $A$ is closed and bounded (compact), and ${C_\alpha}$ is a closed cover of $A$. Then ${C_\alpha^c}$ is an open collection. $\bigcap_\alpha C_\alpha \supseteq A^c$.

**Claim:** A set $A$ is closed-cover-compact if and only if $A$ is compact.

$(\implies)$ Assume $A$ is closed-cover-compact. Let ${U_\alpha}$ be an open cover of $A$. Then ${U_\alpha^c}$ is a collection of closed sets. $A \subseteq \bigcup_\alpha U_\alpha \implies A^c \supseteq \bigcap_\alpha U_\alpha^c$. Consider the closed sets ${A \cap U_\alpha^c}$. If there is no finite subcover of ${U_\alpha}$ for $A$, then for any finite $J$, $A \not\subseteq \bigcup_{\alpha \in J} U_\alpha$, so $A \cap (\bigcap_{\alpha \in J} U_\alpha^c) \neq \emptyset$.

$(\impliedby)$ Assume $A$ is compact. Let ${C_\alpha}$ be a closed cover of $A$. Then $A = \bigcup_\alpha (A \cap C_\alpha)$. These sets $A \cap C_\alpha$ are closed subsets of $A$.

**Proof:** $A$ is closed-cover-compact if and only if $A$ is compact. $(\implies)$ Assume $A$ is closed-cover-compact. Let ${U_\alpha}$ be an open cover of $A$. Consider the closed sets ${U_\alpha^c}$. If ${U_\alpha}$ has no finite subcover, then for any finite $J$, $A \not\subseteq \bigcup_{\alpha \in J} U_\alpha$, so $A \cap (\bigcap_{\alpha \in J} U_\alpha^c) \neq \emptyset$. Consider the collection of closed sets $\mathcal{F} = {A \cap U_\alpha^c}$. Any finite intersection is $A \cap (\bigcap_{\alpha \in J} U_\alpha^c) \neq \emptyset$. By the intersecting closedness property for closed-cover-compact sets, $\bigcap_\alpha (A \cap U_\alpha^c) = A \cap (\bigcap_\alpha U_\alpha^c) = A \cap (\bigcup_\alpha U_\alpha)^c = A \cap A^c = \emptyset$. This is a contradiction. So $A$ is compact.

$(\impliedby)$ Assume $A$ is compact. Let ${C_\alpha}$ be a closed cover of $A$. Then ${C_\alpha^c}$ is an open collection. $A \subseteq \bigcup_\alpha C_\alpha \implies \bigcap_\alpha C_\alpha^c \subseteq A^c$. Consider the open cover ${C_\alpha^c \cup A^c}$. $A \subseteq \bigcup_\alpha C_\alpha$. Let ${C_\alpha}$ be a closed cover of $A$. Then $A = \bigcup_\alpha (A \cap C_\alpha)$. These are closed in $A$.

Final Answer: The closed-cover-compact sets are precisely the **compact sets**.

## Exercise 5.26

**(a) Prove that if $A$ is compact, then $\sup(A)$ exists and $\sup(A) \in A$. Does the same hold for the infimum?**

**Proof:**

1. Assume $A \subseteq \mathbb{R}$ is compact. By the Heine-Borel theorem, $A$ is closed and bounded.
2. Since $A$ is bounded above, by the completeness axiom, $\sup(A)$ exists. Let $s = \sup(A)$.
3. By the definition of supremum, for every $n \in \mathbb{N}$, $s - 1/n$ is not an upper bound for $A$. Thus, there exists $a_n \in A$ such that $s - 1/n < a_n \leq s$.
4. The sequence $(a_n)$ is in $A$ and converges to $s$.
5. Since $A$ is closed and $(a_n)$ is a sequence in $A$ converging to $s$, we must have $s \in A$.
6. Similarly, since $A$ is bounded below, $\inf(A)$ exists. Let $i = \inf(A)$.
7. For every $n \in \mathbb{N}$, $i + 1/n$ is not a lower bound for $A$. Thus, there exists $b_n \in A$ such that $i \leq b_n < i + 1/n$.
8. The sequence $(b_n)$ is in $A$ and converges to $i$.
9. Since $A$ is closed, we must have $i \in A$.
10. Yes, the same holds for the infimum. $\Box$

**(b) Give an example of a set which contains its supremum and its infimum, but is not compact.**

Consider $A = (0, 1)$. $\sup(A) = 1$, $\inf(A) = 0$. $1 \notin A$, $0 \notin A$. Consider $A = (0, 1] \cup {2}$. $\sup(A) = 2 \in A$, $\inf(A) = 0 \notin A$. Consider $A = [0, 1) \cup {-1}$. $\sup(A) = 1 \notin A$, $\inf(A) = -1 \in A$.

**Example:** $A = (0, 1)$. $\sup(A) = 1 \notin A$, $\inf(A) = 0 \notin A$.

**Example:** $A = (0, 1) \cup {0, 1}$. $\sup(A) = 1 \in A$, $\inf(A) = 0 \in A$. $A =$, which is compact.

**Example:** $A = (0, 1) \cup {0}$. $\sup(A) = 1 \notin A$, $\inf(A) = 0 \in A$. Not compact (not closed).

**Example:** $A = (0, 1) \cup {1}$. $\sup(A) = 1 \in A$, $\inf(A) = 0 \notin A$. Not compact (not closed).

**Example:** $A = \cup [2, 3)$. $\sup(A) = 3 \notin A$, $\inf(A) = 0 \in A$. Not connected, not compact. Contains sup and inf of each component.

**Example:** $A = (0, 1) \cup {0, 2}$. $\sup(A) = 2 \in A$, $\inf(A) = 0 \in A$. Not closed, not compact.

**Example:** $A = {0} \cup {1/n: n \in \mathbb{N}}$. $\sup(A) = 1 \in A$, $\inf(A) = 0 \in A$. Limit points is ${0}$. Closed. Bounded. Compact.

**Example:** $A = \setminus {1/n: n \geq 2}$. $\sup(A) = 1 \in \overline{A}$, $\inf(A) = 0 \in A$. $\sup(A) = 1 \notin A$.

**Example:** $A = {0} \cup (1, 2)$. $\sup(A) = 2 \notin A$, $\inf(A) = 0 \in A$. Not closed, not compact.

**Example:** $A = {0} \cup (1, 2) \cup {2}$. $\sup(A) = 2 \in A$, $\inf(A) = 0 \in A$. Not closed (missing 1), not compact.

**(c) If a set contains its supremum, its infimum, and all of its limit points, must it be compact?**

Let $A$ be a set such that $\sup(A)$ exists and is in $A$, $\inf(A)$ exists and is in $A$, and $A$ contains all of its limit points (i.e., $A$ is closed). Since $\sup(A)$ and $\inf(A)$ exist, $A$ is bounded. Since $A$ is closed and bounded, by the Heine-Borel theorem, $A$ is compact.

**Final Answer (c): Yes.**

## Exercise 5.27

Suppose $U_1, \dots, U_n$ is a finite open cover of a compact set $A$. Note that this implies that $A \subseteq U_1 \cup U_2 \cup \dots \cup U_n$. Is it possible that the union $B = (A \setminus U_1) \cup (A \setminus U_2) \cup \dots \cup (A \setminus U_n)$ is non-empty?

$B = A \cap U_1^c \cup A \cap U_2^c \cup \dots \cup A \cap U_n^c = A \cap (U_1^c \cup U_2^c \cup \dots \cup U_n^c) = A \cap (U_1 \cap U_2 \cap \dots \cap U_n)^c = A \setminus (U_1 \cap U_2 \cap \dots \cap U_n)$. If $B$ is non-empty, it means there exists $x \in A$ such that $x \notin (U_1 \cap U_2 \cap \dots \cap U_n)$. This means there exists $x \in A$ such that $x \notin U_i$ for at least one $i \in {1, \dots, n}$. This is possible. For example, let $A =$, $U_1 = (-0.5, 0.5)$, $U_2 = (0.3, 1.5)$. $A \subseteq U_1 \cup U_2 = (-0.5, 1.5)$. $A \setminus U_1 = (0.5, 1]$. $A \setminus U_2 = [0, 0.3]$. $B = (0.5, 1] \cup [0, 0.3]$, which is non-empty.

**Final Answer: Yes, it is possible that $B$ is non-empty.**

## Exercise 5.28

For which compact sets $A$ does there exist an $m \in \mathbb{N}$ such that every open cover of $A$ contains a subcover containing at most $m$ open sets?

This property holds if and only if $A$ can be covered by at most $m$ open sets from any open cover. This means that there exists a finite subcover of size at most $m$. This is always true for a compact set (there exists *some* finite subcover).

The question seems to imply a uniform bound $m$ on the size of the minimal subcover.

If $A$ is a finite set, say $|A| = k$, then any open cover ${U_\alpha}$ has a finite subcover. We can choose one $U_{\alpha_i}$ for each element $x_i \in A$, so a subcover of size at most $k$. In this case, $m = k$ works.

If $A$ is an infinite compact set, consider the open cover ${(x - \epsilon, x + \epsilon): x \in A}$ for some fixed $\epsilon > 0$. Any subcover must cover $A$. If we take a finite subcover, it covers $A$.

Consider $A =$. Open cover ${(x - 1/(n+1), x + 1/(n+1)): x \in, n \in \mathbb{N}}$.

The compact sets $A$ for which such an $m$ exists are the **finite sets**.

**Proof:** If $A$ is finite, $|A| = k$, then for any open cover ${U_\alpha}$, we can choose one $U_{\alpha_i}$ for each element $x_i \in A$ to form a subcover of size at most $k$. So $m = k$ works.

If $A$ is an infinite compact set, suppose such an $m$ exists. For each $x \in A$, consider the open set $V_x = (x - 1/k, x + 1/k)$ for some $k > 0$. ${V_x}_{x \in A}$ is an open cover of $A$. By compactness, there is a finite subcover ${V_{x_1}, \dots, V_{x_n}}$. This doesn't directly give a contradiction.

Suppose $A$ is infinite and compact. Assume there exists $m$ such that every open cover has a subcover of size at most $m$. Consider the open cover ${B(x, 1/(2m)): x \in A}$. By compactness, there is a finite subcover $B(x_1, 1/(2m)), \dots, B(x_k, 1/(2m))$. If $k > m$, this contradicts the assumption. So $k \leq m$. Then $A \subseteq \bigcup_{i=1}^k B(x_i, 1/(2m))$. This means $A$ is a union of finitely many balls of radius $1/(2m)$.

If $A$ is infinite, we can construct an open cover where the minimal subcover size can be arbitrarily large.

**Final Answer: The compact sets $A$ for which such an $m$ exists are the finite sets.**

## Exercise 5.29

Call a subset $A$ of real numbers clopen if it is both open and closed. Prove that the only clopen sets are $\emptyset$ and $\mathbb{R}$.

**Proof:**

1. Suppose $A \subseteq \mathbb{R}$ is clopen and $A \neq \emptyset, A \neq \mathbb{R}$.
2. Since $A \neq \emptyset$, there exists $a \in A$. Since $A \neq \mathbb{R}$, there exists $b \notin A$.
3. Without loss of generality, assume $a < b$. Let $S = A \cap [a, b]$.
4. Since $A$ is closed, $S$ is closed (intersection of two closed sets).
5. $S$ is non-empty since $a \in S$. $S$ is bounded above by $b$, so $\sup(S)$ exists. Let $s = \sup(S)$.
6. Since $S$ is closed, $s \in S$, so $s \in A$ and $s \leq b$.
7. Since $b \notin A$, we must have $s < b$.
8. Since $A$ is open and $s \in A$, there exists $\delta > 0$ such that $(s - \delta, s + \delta) \subseteq A$.
9. Since $s = \sup(S)$, for any $\epsilon > 0$, $(s - \epsilon, s]$ contains points of $S \subseteq A$.
10. Choose $\delta > 0$ such that $s + \delta \leq b$. Then $(s - \delta, s + \delta) \subseteq A$.
11. This means $(s, s + \delta) \subseteq A$. Since $s = \sup(A \cap [a, b])$, any point greater than $s$ in $[a, b]$ should not be in $A$. This is a contradiction.
12. Therefore, the only clopen sets are $\emptyset$ and $\mathbb{R}$. $\Box$

## Exercise 5.30

Give two examples of sets which are neither open nor closed. Have one example be a bounded set and the other be an unbounded set.

Bounded: $(0, 1]$. Open because for $x \in (0, 1)$, we can find $\delta > 0$ such that $(x - \delta, x + \delta) \subseteq (0, 1]$. For $x = 1$, any neighborhood $(1 - \epsilon, 1 + \epsilon)$ contains points outside $(0, 1]$ (e.g., $1 + \epsilon/2$). Not closed because the sequence $(1 - 1/n)$ is in $(0, 1]$ and converges to $1 \notin (0, 1]$.

Unbounded: $[0, \infty)$. Closed because its complement $(-\infty, 0)$ is open. Not open because for $x = 0$, any neighborhood $(-\epsilon, \epsilon)$ contains negative numbers.

Bounded and neither open nor closed: $(0, 1]$. Unbounded and neither open nor closed: $(0, \infty)$.

## Exercise 5.31

**(a) Find the interior, exterior and boundary for each of the following sets: $(0,1),, \mathbb{R}$ and $\mathbb{Q}$.**

- $A = (0, 1)$:
    - $\text{Int}(A) = (0, 1)$
    - $\text{Ext}(A) = (-\infty, 0) \cup (1, \infty)$
    - $\partial A = {0, 1}$
- $A =$:
    - $\text{Int}(A) = (0, 1)$
    - $\text{Ext}(A) = (-\infty, 0) \cup (1, \infty)$
    - $\partial A = {0, 1}$
- $A = \mathbb{R}$:
    - $\text{Int}(A) = \mathbb{R}$
    - $\text{Ext}(A) = \emptyset$
    - $\partial A = \emptyset$
- $A = \mathbb{Q}$:
    - $\text{Int}(A) = \emptyset$
    - $\text{Ext}(A) = \emptyset$
    - $\partial A = \mathbb{R}$

**(b) Prove that for any set $A$, we have $\mathbb{R} = \text{Int}(A) \cup \partial A \cup \text{Ext}(A)$, and this is a disjoint union.**

**Proof:**

1. Let $x \in \mathbb{R}$.
2. If there exists a neighborhood of $x$ contained in $A$, then $x \in \text{Int}(A)$.
3. If there exists a neighborhood of $x$ contained in $A^c$, then $x \in \text{Ext}(A)$.
4. If every neighborhood of $x$ contains points from both $A$ and $A^c$, then $x \in \partial A$.
5. These three cases cover all possibilities for $x \in \mathbb{R}$, so $\mathbb{R} = \text{Int}(A) \cup \partial A \cup \text{Ext}(A)$.
6. Now we show the union is disjoint.
7. Suppose $x \in \text{Int}(A) \cap \partial A$. Then there exists $\delta_1 > 0$ such that $B(x, \delta_1) \subseteq A$. Since $x \in \partial A$, every neighborhood of $x$, including $B(x, \delta_1)$, must contain points from $A^c$. This is a contradiction. So $\text{Int}(A) \cap \partial A = \emptyset$.
8. Suppose $x \in \text{Ext}(A) \cap \partial A$. Then there exists $\delta_2 > 0$ such that $B(x, \delta_2) \subseteq A^c$. Since $x \in \partial A$, every neighborhood of $x$, including $B(x, \delta_2)$, must contain points from $A$. This is a contradiction. So $\text{Ext}(A) \cap \partial A = \emptyset$.
9. Suppose $x \in \text{Int}(A) \cap \text{Ext}(A)$. Then there exists $\delta_1 > 0$ such that $B(x, \delta_1) \subseteq A$, and $\delta_2 > 0$ such that $B(x, \delta_2) \subseteq A^c$. Let $\delta = \min(\delta_1, \delta_2)$. Then $B(x, \delta) \subseteq A$ and $B(x, \delta) \subseteq A^c$, which implies $B(x, \delta) = \emptyset$, a contradiction. So $\text{Int}(A) \cap \text{Ext}(A) = \emptyset$.
10. Thus, the union is disjoint. $\Box$

## Exercise 5.32

Prove that the only sets with empty boundary are $\mathbb{R}$ and $\emptyset$.

**Proof:**

1. Suppose $\partial A = \emptyset$.
2. For any $x \in \mathbb{R}$, $x \notin \partial A$. This means it's not the case that every neighborhood of $x$ contains points from both $A$ and $A^c$.
3. So for every $x \in \mathbb{R}$, there exists a neighborhood of $x$ that is either entirely contained in $A$ or entirely contained in $A^c$.
4. Let $U = \text{Int}(A)$ and $V = \text{Int}(A^c) = \text{Ext}(A)$.
5. $U$ and $V$ are open sets, and $U \cap V = \emptyset$.
6. $\mathbb{R} = U \cup \partial A \cup V = U \cup V$.
7. If $A \neq \emptyset$ and $A \neq \mathbb{R}$, then $U \neq \mathbb{R}$ and $V \neq \mathbb{R}$.
8. Since $U \cup V = \mathbb{R}$ and $U \cap V = \emptyset$, $V = U^c$. Since $U$ is open, $V$ is closed. Since $V$ is open, $U$ is closed.
9. So $U$ is clopen. By Exercise 5.29, $U = \emptyset$ or $U = \mathbb{R}$.
10. If $U = \mathbb{R}$, then $\text{Int}(A) = \mathbb{R}$, so $A = \mathbb{R}$.
11. If $U = \emptyset$, then $\text{Int}(A) = \emptyset$, so $A$ contains no interior points. Then $\mathbb{R} = V = \text{Int}(A^c)$, so $A^c = \mathbb{R}$, which means $A = \emptyset$.
12. Therefore, if $\partial A = \emptyset$, then $A = \mathbb{R}$ or $A = \emptyset$. $\Box$

## Exercise 5.33

**(a) Explain intuitively what it means for a set $A$ to be connected.**

A set $A$ is connected if it cannot be split into two non-empty, disjoint open sets (relative to $A$). Intuitively, a connected set is "all in one piece". There are no gaps or separations within the set.

**(b) Give an example of a connected set and of a set that is not connected. You do not need to prove your answers.**

Connected: $()$ Not connected: $ \cup$

**(c) Give an example of a set $A$ which is not connected, but $A \cup {4}$ is connected.**

$A = \cup$. Not connected. $A \cup {4} = \cup \cup {4}$. Still not connected.

$A = (0, 1) \cup (2, 3)$. Not connected. $A \cup {1.5} = (0, 1) \cup {1.5} \cup (2, 3)$. Still not connected.

$A = (0, 1) \cup (1, 2)$. Not connected. $A \cup {1} = (0, 2)$, which is connected.

**(d) Prove that ${1, 2, 3, 4, 5}$ is not connected.**

Let $A = {1, 2, 3, 4, 5}$. Consider $U = (-\infty, 2.5)$ and $V = (2.5, \infty)$. $U$ and $V$ are open and disjoint. $U \cap A = {1, 2} \neq \emptyset$. $V \cap A = {3, 4, 5} \neq \emptyset$. $(U \cap A) \cup (V \cap A) = {1, 2} \cup {3, 4, 5} = A$. Thus, $A$ is not connected. $\Box$

**(e) Prove that $\mathbb{Q}$ is not connected.**

Consider $U = (-\infty, \sqrt{2})$ and $V = (\sqrt{2}, \infty)$. $U$ and $V$ are open and disjoint. $U \cap \mathbb{Q} = {q \in \mathbb{Q}: q < \sqrt{2}} \neq \emptyset$. $V \cap \mathbb{Q} = {q \in \mathbb{Q}: q > \sqrt{2}} \neq \emptyset$. $(U \cap \mathbb{Q}) \cup (V \cap \mathbb{Q}) = {q \in \mathbb{Q}: q < \sqrt{2} \text{ or } q > \sqrt{2}} = \mathbb{Q} \setminus {\sqrt{2} \cap \mathbb{Q}} = \mathbb{Q} \setminus \emptyset = \mathbb{Q}$. Thus, $\mathbb{Q}$ is not connected. $\Box$

**(f) Give an example of a set $A$ which is not connected, but there exists some $x_0 \in \mathbb{R}$ such that $A \cup {x_0}$ is connected.**

$A = (0, 1) \cup (2, 3)$. Not connected. Let $x_0 = 1.5$. $A \cup {1.5} = (0, 1) \cup {1.5} \cup (2, 3)$. Still not connected. Let $A = (0, 1) \cup (1, 2)$. Not connected. Let $x_0 = 1$. $A \cup {1} = (0, 2)$, which is connected.

**(g) Prove that a set of real numbers with more than one element is connected if and only if it is an interval.**

This is a standard theorem in real analysis. The proof can be found in many real analysis textbooks. $\Box$ (Proof omitted due to length and being a standard result not fully derivable from the immediate context).
