---
title: 01_the_reals_real_analysis_exercises
uuid: b0c8555c-1573-4461-b413-b599e9a5e174
aliases:
  - "Real Analysis: The Reals, Exercises"
  - "The Reals: Exercises"
  - "1. The Reals: Exercises"
  - the_reals_exercises
  - the reals exercises
  - real_analysis_the_reals_exercises
  - 01_the_reals_exercises_real_analysis
main_title: The Reals
subtitle: Exercises
author:
  - "[[cummings_jay|Jay Cummings]]"
editor:
translator:
year_published: 2019
publisher:
page_start: 1
page_end: 41
doi:
url: https://longformmath.com/analysis-home
library:
  - "[[01_the_reals_real_analysis|1. The Reals]]"
  - "[[cummings_2019_real_analysis|Real Analysis: A Long-form Mathematics Textbook]]"
cssclasses:
status: undetermined
type: book_chapter
file_class: lib_book_chapter
date_created: 2024-12-22T19:42
date_modified: 2025-10-05T17:48
tags:
---
# 1. The Reals: Exercises

---

## Exercise 1.1

Explain the error in the following "proof" that 2 = 1.

Let $x = y$. Then

$$
\begin{align}
x^2 - xy &= xy - y^2 \\
(x+y)(x-y) &= y(x-y) \\
x+y &= y \\
2y &= y \\
2 &= 1 \end{align}
$$

---

## Exercise 1.2

Which of the following statements are true? Give a short explanation for each of your answers.

1\. For every $n \in \mathbb{N}$, there is an integer $m \in \mathbb{N}$ such that $m > n$.

$$
\forall n \in \mathbb{N}, \exists m \in \mathbb{N} \, (m > n)
$$

2\. For every $m \in \mathbb{N}$, there is an integer $n \in \mathbb{N}$ such that $m > n$.

$$
\forall m \in \mathbb{N}, \exists n \in \mathbb{N} \, (m > n)
$$

3\. There is an $m \in \mathbb{N}$ such that for every $n \in \mathbb{N}$, $m \ge n$.

$$
\exists m \in \mathbb{N}, \forall n \in \mathbb{N} \, (m \ge n)
$$

4\. There is an $n \in \mathbb{N}$ such that for every $m \in \mathbb{N}$, $m \ge n$.

$$
\exists n \in \mathbb{N}, \forall m \in \mathbb{N} \, (m \ge n)
$$

5\. There is an $n \in \mathbb{R}$ such that for every $m \in \mathbb{R}$, $m \ge n$.

$$
\exists n \in \mathbb{R}, \forall m \in \mathbb{R} \, (m \ge n)
$$

6\. For every pair $x < y$ of integers, there is an integer $z$ such that $x < z < y$.

$$
\forall x, y \in \mathbb{Z}, \, (x < y) \implies \exists z \in \mathbb{Z} \, (x < z < y)
$$

7\. For every pair $x < y$ of real numbers, there is a real number $z$ such that $x < z < y$.

$$
\forall x, y \in \mathbb{R}, \, (x < y) \implies \exists z \in \mathbb{R} \, (x < z < y)
$$

---

## Exercise 1.3

If $A$ and $B$ are two boxes (possibly with things inside), describe the following in terms of boxes:

1\. $A \backslash B$

2\. $\mathcal{P}(a)$

3\. $|A|$

---

## Exercise 1.4

If $A_1, A_2, A_3, \dots, A_n$ are all boxes (possibly with things inside), describe the following in terms of boxes:

1\. $\bigcup_{i=1}^{n} A_i = A_1 \cup A_2 \cup \dots \cup A_n$

2\. $\bigcap_{i=1}^{n} A_i = A_1 \cap A_2 \cap \dots \cap A_n$

---

## Exercise 1.5

Prove that each of the following holds for any sets $A$ and $B$.

### Exercise 1.5.1. $A \cup B = A$ if and only if $B \subseteq A$

#### Proof

1. Assume $A \cup B = A$.
   - By definition of union, $A \cup B = \{x \mid x \in A \text{ or } x \in B\}$.
   - Since $A \cup B = A$, this implies $x \in B \implies x \in A$, so $B \subseteq A$.

2. Assume $B \subseteq A$.
   - Then every $x \in B$ is also in $A$.
   - By the definition of union, $A \cup B = \{x \mid x \in A \text{ or } x \in B\} = A$, since $B \subseteq A$.

$\therefore ~ \boxed{A \cup B = A \iff B \subseteq A}$

### Exercise 1.5.2. $A \cap B = A$ if and only if $A \subseteq B$

#### Proof

1. Assume $A \cap B = A$.
   - By definition of intersection, $A \cap B = \{x \mid x \in A \text{ and } x \in B\}$.
   - Since $A \cap B = A$, this implies $x \in A \implies x \in B$, so $A \subseteq B$.

2. Assume $A \subseteq B$.
   - Then every $x \in A$ is also in $B$.
   - By the definition of intersection, $A \cap B = \{x \mid x \in A \text{ and } x \in B\} = A$.

$\therefore ~ \boxed{A \cap B = A \iff A \subseteq B}$

### Exercise 1.5.3. $A \backslash B = A$ if and only if $A \cap B = \emptyset$

#### Proof

1. Assume $A \backslash B = A$.
   - By definition of set difference, $A \backslash B = \{x \mid x \in A \text{ and } x \notin B\}$.
   - Since $A \backslash B = A$, this implies $x \in A \implies x \notin B$, so $A \cap B = \emptyset$.

2. Assume $A \cap B = \emptyset$.
   - Then no $x \in A$ is in $B$, so $A \backslash B = \{x \mid x \in A \text{ and } x \notin B\} = A$.

$\therefore ~\boxed{A \backslash B = A \iff A \cap B = \emptyset}$

### Exercise 1.5.4. $A \backslash B = \emptyset$ if and only if $A \subseteq B$

#### Proof

1. Assume $A \backslash B = \emptyset$.
   - By definition of set difference, $A \backslash B = \{x \mid x \in A \text{ and } x \notin B\}$.
   - Since $A \backslash B = \emptyset$, this implies no $x \in A$ satisfies $x \notin B$, so $x \in A \implies x \in B$, meaning $A \subseteq B$.

2. Assume $A \subseteq B$.
   - Then every $x \in A$ is also in $B$.
   - By the definition of set difference, $A \backslash B = \{x \mid x \in A \text{ and } x \notin B\} = \emptyset$.

$\therefore ~ \boxed{A \backslash B = \emptyset \iff A \subseteq B}$

---

## Exercise 1.6

Suppose $f: X \rightarrow Y$ and $A \subseteq X$ and $B \subseteq Y$

### Exercise 1.6.1. Prove that $f(f^{-1}(b)) \subseteq B$

#### Proof

1\. By definition of $f^{-1}(B)$:

$$
f^{-1}(B) = \{x \in X \mid f(x) \in B\}.
$$

2\. Applying $f$ to $f^{-1}(B)$:

$$
f(f^{-1}(B)) = \{f(x) \mid x \in X \text{ and } f(x) \in B\}.
$$

3\. Since every $f(x) \in B$, it follows that:

$$
f(f^{-1}(B)) \subseteq B.
$$

$\boxed{\text{Q.E.D.}}$

### Exercise 1.6.2. Give an Example where $f(f^{-1}(b)) \ne B$

#### Example

Let $X = \{1, 2\}$, $Y = \{a, b, c\}$, and define $f: X \to Y$ by:

$$
f(1) = a, \quad f(2) = b.
$$

Let $B = \{a, b, c\}$.

- $f^{-1}(B) = \{1, 2\}$,
- $f(f^{-1}(B)) = \{a, b\} \neq B$.

### Exercise 1.6.3. Prove that $A \subseteq f^{-1}(f(a))$

#### Proof

1\. By definition of $f^{-1}(f(A))$:

$$
f^{-1}(f(A)) = \{x \in X \mid f(x) \in f(A)\}.
$$

2\. If $x \in A$, then $f(x) \in f(A)$, so $x \in f^{-1}(f(A))$.

3\. Thus, $A \subseteq f^{-1}(f(A))$.

$\boxed{\text{Q.E.D.}}$

### Exercise 1.6.4. Give an Example where $A \ne f^{-1}(f(a))$

#### Example

Let $X = \{1, 2\}$, $Y = \{a, b\}$, and define $f: X \to Y$ by:

$$
f(1) = a, \quad f(2) = a.
$$

Let $A = \{1\}$.

- $f(A) = \{a\}$,
- $f^{-1}(f(A)) = \{1, 2\} \neq A$.

---

## Exercise 1.7

Suppose that $f: X \rightarrow Y$ and $g: Y \rightarrow X$ are functions and that the composite $g \circ f$ is the identity function $id: X \rightarrow X$. (The identity function sends every element to itself: $id(x) = x$.) Show that $f$ must be a one-to-one function and that $g$ must be an onto function.

### Proof

1\. Since $g \circ f = id$, we have:

$$
(g \circ f)(x) = g(f(x)) = x, \quad \forall x \in X.
$$

2\. **$f$ is one-to-one:**
- Suppose $f(x_1) = f(x_2)$.
- Applying $g$, we get:

$$
g(f(x_1)) = g(f(x_2)) \implies x_1 = x_2.
$$

- Thus, $f$ is one-to-one.

3\. **$g$ is onto:**
   - Let $y \in Y$.
   - Since $g \circ f = id$, for each $x \in X$, there exists $y = f(x)$ such that $g(y) = x$.
   - Thus, $g$ maps every $y \in Y$ to some $x \in X$, so $g$ is onto.

$\boxed{\text{Q.E.D.}}$

---

## Exercise 1.8

The following are special cases of De Morgan's laws.

1\. Prove that $(A \cap B)^{c} = A^{c} \cup B^{c}$

2\. Prove that $(A \cup B)^{c} = A^{c} \cap B^{c}$

---

## Exercise 1.9

1\. Prove that $\sqrt{3}$ is irrational.

2\. What goes wrong when you try to adapt your argument from part 1 to show that $\sqrt{4}$ is irrational (which is absurd)?

3\. In part 1, you proved that $\sqrt{3}$ to be irrational, and essentially the same proof shows that $\sqrt{5}$ is irrational. By considering their product or otherwise, prove that $\sqrt{3} - \sqrt{5}$ and $\sqrt{3} + \sqrt{5}$ are either both rational or both irrational. Deduce that they must both be irrational.

---

## Exercise 1.10

Prove that the multiplicative identity in a field is unique.

### Exercise 1.10: Proof

1\. **Definition of Multiplicative Identity:** The multiplicative identity $e$ in a field $F$ satisfies:

$$
e \cdot a = a \cdot e = a \quad \text{for all } a \in F.
$$

2\. **Assume Two Multiplicative Identities Exist:** Suppose $e_1, e_2 \in F$ are both multiplicative identities. Then for all $a \in F$:

$$
e_1 \cdot a = a \quad \text{and} \quad e_2 \cdot a = a.
$$

3\. **Substitute $a = e_2$ into $e_1 \cdot a = a$:**

$$
e_1 \cdot e_2 = e_2.
$$

4\. **Substitute $a = e_1$ into $e_2 \cdot a = a$:**

$$
e_2 \cdot e_1 = e_1.
$$

5\. **Use Commutativity of Multiplication:** Since multiplication in a field is commutative:

$$
e_1 \cdot e_2 = e_2 \cdot e_1.
$$

Thus:

$$
e_2 = e_1.
$$

6\. This contradicts the assumption that $e_1$ and $e_2$ are distinct.

**Conclusion:** The multiplicative identity in a field is **unique**:

$$
\boxed{\text{Q.E.D.}}
$$

---

## Exercise 1.11

Given an ordered field F, recall that we defined the positive elements to be a nonempty subset $P \subseteq F$ that satisfies both of the following conditions:

- If $a, b \in P$, then $a + b \in P$ and $a \cdot b \in P$.
- If $a \in F$ and $a \ne 0$, then either $a \in P$ or $-a \in P$, but not both.

1\. Give an example of some $P_1 \subseteq \mathbb{R}$ that satisfies (i) but not (ii).

2\. Give an example of some $P_2 \subseteq \mathbb{R}$ that satisfies (ii) but not (i).

---

## Exercise 1.12

Assume that F is an ordered field and $a, b, c, d \in F$ with $a < b$ and $c < d$.

1\. Show that $a + c < b + d$.

2\. Prove that it is not necessarily true that $ac < bd$.

Note whenever you use an axiom.

---

## Exercise 1.13

Let $a$, $b$, and $\varepsilon$ be elements of an ordered field.

Note whenever you use an axiom.

### 1.13.1. Show that for Every $\varepsilon > 0$, if $a<b+\varepsilon$ then $a \le b$

#### **Proof:**

1\. Assume $a < b + \varepsilon$ for all $\varepsilon > 0$.

2\. By the **trichotomy property** of an ordered field, exactly one of the following must hold:
- $a < b$,
- $a = b$, or
- $a > b$.

3\. Suppose $a > b$. Then:

$$
a - b > 0.
$$

4\. Let $\varepsilon = \frac{a - b}{2}$. Since $a - b > 0$, this ensures $\varepsilon > 0$.

5\. Substitute $\varepsilon = \frac{a - b}{2}$ into $a < b + \varepsilon$:

$$
a < b + \frac{a - b}{2}.
$$

6\. Simplify:

$$
a < \frac{2b + a - b}{2} = \frac{a + b}{2}.
$$

7\. Multiply through by 2:

$$
2a < a + b.
$$

8\. Subtract $a$ from both sides:

$$
a < b.
$$

9\. This contradicts the assumption $a > b$.

10\. Hence, $a \le b$. If $a < b$, the statement is true. If $a = b$, then $a \le b$ holds trivially.

### 1.13.2. Show that for Every $\varepsilon > 0$, if $|a - b| < \varepsilon$, then $a = b$

#### **Proof:**

1\. Assume $|a - b| < \varepsilon$ for all $\varepsilon > 0$.

2\. By the definition of absolute value:

$$
|a - b| < \varepsilon \iff -\varepsilon < a - b < \varepsilon.
$$

3\. From $a - b < \varepsilon$ for all $\varepsilon > 0$, apply part 1:

$$
a - b \le 0.
$$

4\. Similarly, from $-\varepsilon < a - b$ for all $\varepsilon > 0$, apply part 1:

$$
a - b \ge 0.
$$

5\. Combining $a - b \le 0$ and $a - b \ge 0$, we conclude:

$$
a - b = 0 \implies a = b.
$$

---

## Exercise 1.14

Prove that the equality $|ab| = |a| \cdot |b|$ holds for all real numbers a and b.

---

## Exercise 1.15

For each of the following, find all numbers $x$ which satisfy the expression.

1\. $|x - 4| = 7$

2\. $|x - 4| < 7$

3\. $|x + 2| < 1$

4\. $|x - 1| + |x - 2| > 1$

5\. $|x - 1| + |x + 1| > 1$

6\. $|x - 1| - |x + 1| > 1$

7\. $|x - 1| \cdot |x + 1| = 0$

8\. $|x - 1| \cdot |x + 2| = 3$

---

## Exercise 1.16

Let $\max\{x, y\}$ denote the maximum of the real numbers $x$ and $y$, and let $\min\{x, y\}$ denote the minimum. For example, $\min\{-1, 4\} = \min\{-1, -1\} = -1$. Prove that

$$
\max\{x, y\} = \frac{x + y + |y - x|}{2} \quad \text{and}\quad \min\{x, y\} = \frac{x + y - |y - x|}{2}
$$

Then find a formula for $\max\{x, y, z\}$ and $\min\{x, y, z\}$.

### Proof of Maximum and Minimum Formulas

#### 1. Formula for $\max\{x, y\}$

We need to show:

$$
\max\{x, y\} = \frac{x + y + |y - x|}{2} = \begin{cases}
y & \text{if } x \leq y, \\
x & \text{if } x > y \end{cases}
$$

##### Case 1: $x \leq y$

1\. If $x \leq y$, then:

$$
y - x \geq 0.
$$

2\. From the definition of the absolute value:

$$
|y - x| = y - x.
$$

3\. Substituting into the formula:

$$
\frac{x + y + |y - x|}{2} = \frac{x + y + (y - x)}{2}.
$$

4\. Simplify:

$$
\frac{2y}{2} = y.
$$

$$
\boxed{\frac{x + y + |y - x|}{2} = y \quad \text{when } x \leq y.}
$$

##### Case 2: $x > y$

1\. If $x > y$, then:

$$
y - x < 0.
$$

2\. From the definition of the absolute value:

$$
|y - x| = x - y.
$$

3\. Substituting into the formula:

$$
\frac{x + y + |y - x|}{2} = \frac{x + y + (x - y)}{2}.
$$

4\. Simplify:

$$
\frac{2x}{2} = x.
$$

Thus:

$$
\frac{x + y + |y - x|}{2} = x \quad \text{when } x > y.
$$

##### Conclusion

In both cases:

$$
\boxed{\max\{x, y\} = \frac{x + y + |y - x|}{2}}.
$$

#### 2. Formula for $\min\{x, y\}$

##### Proof

We aim to show that:

$$
\frac{x + y - |y - x|}{2} =
\begin{cases}
x & \text{if } x \leq y, \\
y & \text{if } x > y.
\end{cases}
$$

##### Case 1: $x \leq y$

1\. If $x \leq y$, then:

$$
y - x \geq 0.
$$

2\. From the definition of the absolute value:

$$
|y - x| = y - x.
$$

3\. Substituting into the formula:

$$
\frac{x + y - |y - x|}{2} = \frac{x + y - (y - x)}{2}.
$$

4\. Simplify:

$$
\frac{2x}{2} = x.
$$

Thus:

$$
\frac{x + y - |y - x|}{2} = x \quad \text{when } x \leq y.
$$

##### Case 2: $x > y$

1\. If $x > y$, then:

$$
y - x < 0.
$$

2\. From the definition of the absolute value:

$$
|y - x| = x - y.
$$

3\. Substituting into the formula:

$$
\frac{x + y - |y - x|}{2} = \frac{x + y - (x - y)}{2}.
$$

4\. Simplify:

$$
\frac{2y}{2} = y.
$$

Thus:

$$
\frac{x + y - |y - x|}{2} = y \quad \text{when } x > y.
$$

##### Conclusion

$$
\frac{x + y - |y - x|}{2} =
\begin{cases}
x & \text{if } x \leq y, \\
y & \text{if } x > y.
\end{cases}
$$

Therefore:

$$
\boxed{\min\{x, y\} = \frac{x + y - |y - x|}{2}}.
$$

#### 3. Formula for $\max\{x,y,z\}$ and $\min\{x,y,z\}$

**For $\max\{x, y, z\}$**:

Using the two-variable formula:

$$
\max\{x, y, z\} = \max\{\max\{x, y\}, z\}.
$$

Substitute $\max\{x, y\} = \frac{x + y + |y - x|}{2}$:

$$
\max\{x, y, z\} = \frac{\frac{x + y + |y - x|}{2} + z + \left|z - \frac{x + y + |y - x|}{2}\right|}{2}.
$$

**For $\min\{x, y, z\}$**:

Using the two-variable formula:

$$
\min\{x, y, z\} = \min\{\min\{x, y\}, z\}.
$$

Substitute $\min\{x, y\} = \frac{x + y - |y - x|}{2}$:

$$
\min\{x, y, z\} = \frac{\frac{x + y - |y - x|}{2} + z - \left|z - \frac{x + y - |y - x|}{2}\right|}{2}.
$$

---

## Exercise 1.17

Prove that if $a, b \in \mathbb{R}$ and $0 < a < b$, then $a^n < b^n$ for any positive integer $n$.

---

## Exercise 1.18

Prove that if $a_1, a_2, \dots, a_n$ are real numbers, then

$$
|a_1 + a_2 + \dots + a_n| \le |a_1| + |a_2| + \dots + |a_n|
$$

---

## Exercise 1.19

Prove that for all $n \in \mathbb{N}$,

$$
\sum_{k=1}^{n}\frac{1}{k(k+1)}=\frac{n}{n+1}
$$

---

## Exercise 1.20

Determine which natural numbers, $n$, have the property that $\sqrt{n}$ is irrational.

---

## Exercise 1.21

Let $f: X \rightarrow Y$, and assume $A_1, A_2 \subseteq X$. Show that

$$
f(A_1 \cap A_2) \subseteq f(A_1) \cap f(A_2)
$$

Recall that if $A$ is a set, then $f(a) = \{f(x): x \in A\}$.

---

## Exercise 1.22

Give an example of a function $f$, and a pair of sets $A$ and $B$, for which

$$
f(A \cap B) \ne f(a) \cap f(b)
$$

Recall that if $A$ is a set, then $f(a) = \{f(x): x \in A\}$.

---

## Exercise 1.23

Assume that $A \subseteq B$ and both are bounded above. Prove that $\sup(a) \le \sup(b)$.

**Prove:** If $A \subseteq B$ and both $A$ and $B$ are bounded above, then $\sup(A) \leq \sup(B)$.

### Proof

1\. Let $\alpha = \sup(A)$ and $\beta = \sup(B)$.
2\. By the definition of $\sup(B)$, $\beta$ is an upper bound for $B$, so:

$$
b \leq \beta \quad \forall b \in B.
$$

3\. Since $A \subseteq B$, every $a \in A$ is also in $B$. Hence:

$$
a \leq \beta \quad \forall a \in A.
$$

4\. Thus, $\beta$ is also an upper bound for $A$.

5\. By the definition of $\sup(A)$, $\alpha$ is the least upper bound for $A$, so:

$$
\alpha \leq \beta.
$$

$\boxed{\text{Q.E.D.}}$

---

## Exercise 1.24

Suppose $A \subseteq \mathbb{R}$ has a maximal element — that is, there is an element $M \in A$ such that $x \le M$ for all $x \in A$. Likewise, assume $B \subseteq \mathbb{R}$ has a minimal element $m$.

### Exercise 1.24.1. Prove that $\sup(A) = M$, where $M$ is the Maximal Element of $A$

**Proof:**

1. By definition, $M \in A$ satisfies $x \leq M$ for all $x \in A$.
2. Since $M$ is an upper bound of $A$, $\sup(A) \leq M$.
3. Also, since $M \in A$, $M$ is the greatest element of $A$, and no smaller upper bound exists.
4. By the definition of the supremum as the least upper bound, $\sup(A) = M$.

$\boxed{\text{Q.E.D.}}$

### Exercise 1.24.2. Prove that $\inf(B) =m$, where $m$ is the Minimal Element of $B$

**Proof:**

1. By definition, $m \in B$ satisfies $m \leq y$ for all $y \in B$.
2. Since $m$ is a lower bound of $B$, $\inf(B) \geq m$.
3. Also, since $m \in B$, $m$ is the smallest element of $B$, and no larger lower bound exists.
4. By the definition of the infimum as the greatest lower bound, $\inf(B) = m$.

$\boxed{\text{Q.E.D.}}$

---

## Exercise 1.25

Suppose that $A$ is a nonempty set containing finitely many elements. Prove by induction that $A$ has a maximal element, and that $\max(a) \in A$.

### Proof by Induction

1\. **Base Case ($|A| = 1$):**
- If $A = \{a_1\}$, then $a_1$ is the only element of $A$ and trivially satisfies $a_1 = \max(A)$.

2\. **Inductive Step:**
- Assume the result holds for any set $A'$ with $n$ elements: $A'$ has a maximal element $\max(A') \in A'$.
- Let $A$ be a set with $n + 1$ elements. Write $A = A' \cup \{a_{n+1}\}$, where $A'$ has $n$ elements.
- By the inductive hypothesis, $A'$ has a maximal element, say $m = \max(A')$.
- Compare $m$ and $a_{n+1}$:
 - If $m \geq a_{n+1}$, then $m = \max(A)$.
 - Otherwise, $a_{n+1} = \max(A)$.

3\. By induction, $A$ has a maximal element for any finite set.

$\boxed{\text{Q.E.D.}}$

---

## Exercise 1.26

Prove that $\mathbb{N}$ is complete.

---

## Exercise 1.27

For each item, compute the requested supremum or infimum or carefully explain why it does not exist. Either way, prove that your answer is correct.

1\. Determine $\sup A$ for $A = \left\{\frac{(-1)^{n}}{2}: n \in \mathbb{N}\right\}$.

### Exercise 1.27.1. Find $\sup(A)$ for $A = \left\{\frac{(-1)^n}{2}:n\in \mathbb{N}\right\}$

**Solution:**

1. The set alternates between $\frac{1}{2}$ (for odd $n$) and $-\frac{1}{2}$ (for even $n$).
2. Thus:
   - $\sup(A) = \frac{1}{2}$,
   - $\inf(A) = -\frac{1}{2}$.

**Proof:**
- By definition, $\sup(A)$ is the largest value in $A$, which occurs at odd $n$.
- Similarly, $\inf(A)$ is the smallest value in $A$, which occurs at even $n$.

### Exercise 1.27.2. Find $\inf(B)$ for $B = \{\alpha^n:n\in \mathbb{N}\}$, where $\alpha \in (0, 1)$

**Solution:**

1. Since $0 < \alpha < 1$, $\alpha^n \to 0$ as $n \to \infty$.
2. Thus, $\inf(B) = 0$.

**Proof:**
- For any $\varepsilon > 0$, there exists $n$ such that $\alpha^n < \varepsilon$, proving that 0 is the greatest lower bound.

### Exercise 1.27.3. Find $\sup(C)$ for $C = \{\alpha^n:n\in \mathbb{N}\}$, where $\alpha \in (1, \infty)$

**Solution:**

1. Since $\alpha > 1$, $\alpha^n$ increases without bound as $n \to \infty$.
2. Thus, $\sup(C)$ does not exist (the set is unbounded).

**Proof:**
- For any $M > 0$, there exists $n$ such that $\alpha^n > M$, showing that $C$ has no finite upper bound.

$\boxed{\text{Q.E.D.}}$

---

## Exercise 1.28

Prove the infimum case of Theorem 1.24.

---

## Exercise 1.29

Prove that

$$
\sup \{\frac{n}{n+1} : n \in \mathbb{N}\} = 1 \quad \text{and}\quad \inf \{\frac{n}{n+1} : n \in \mathbb{N}\} = \frac{1}{2}
$$

---

## Exercise 1.30

Let $A, B \subseteq \mathbb{R}$, and assume that $\sup(a) < \sup(b)$.

### Exercise 1.30.1. Let $A, B \subseteq \mathbb{R}$ with $\sup(A) < \sup(B)$

#### (a) Show there Exists $b \in B$ that is an Upper Bound for $A$

##### Proof

1\. By definition of the supremum, $\sup(A)$ is the least upper bound of $A$. This means:
- $\sup(A) \geq a$ for all $a \in A$, and
- For any $\varepsilon > 0$, there exists $a \in A$ such that $\sup(A) - \varepsilon < a$.

2\. Similarly, since $\sup(B)$ is the least upper bound of $B$:
- $\sup(B) \geq b$ for all $b \in B$, and
- For any $\varepsilon > 0$, there exists $b \in B$ such that $b > \sup(B) - \varepsilon$.

3\. Given that $\sup(A) < \sup(B)$, let $\varepsilon = \frac{\sup(B) - \sup(A)}{2} > 0$. Then:
- By the property of $\sup(B)$, there exists $b \in B$ such that:

$$
b > \sup(B) - \varepsilon.
$$

- Substituting $\varepsilon = \frac{\sup(B) - \sup(A)}{2}$, we get:

$$
b > \sup(B) - \frac{\sup(B) - \sup(A)}{2} = \frac{\sup(B) + \sup(A)}{2}.
$$

4\. Now, since $\sup(A) < \frac{\sup(B) + \sup(A)}{2} < \sup(B)$, we have:
- $b > \sup(A)$,
- And $b$ is an upper bound for $A$, because $b \geq \sup(A) \geq a$ for all $a \in A$.

Thus, such a $b \in B$ exists. $\boxed{\text{Q.E.D.}}$

#### (b) Example where $\sup(A) \leq \sup(B)$ Does not Imply the Result

2\. Give an example to show that this is not necessarily the case if we instead only assume that $\sup(a) \le \sup(b)$. You do not need to prove your answer.

Let:

$$
A = [0, 1], \quad B = [1, 2].
$$

Here:
- $\sup(A) = 1$,
- $\sup(B) = 2$,
- But no element $b \in B$ is an upper bound for $A$, because every $b \in B$ satisfies $b > 1 = \sup(A)$, which cannot bound $A$ from above.

#### Rule for $\sup(A) \leq \sup(B)$ Without a Shared Upper Bound

Let $A, B \subseteq \mathbb{R}$ be nonempty sets that are bounded above. The condition $\sup(A) \leq \sup(B)$ implies that there exists $b \in B$ such that $b$ is an upper bound for $A$ **if and only if** $\sup(A) \in B$ or $B \cap A \neq \emptyset$:

$$
\bigl[\sup(A) < \sup(B) \implies \exists b \in B[\forall a \in A(b \geq a)]\bigr] \iff \sup(A) \in B ~ \text{ or } ~ B \cap A \neq \emptyset.
$$

##### Proof

###### $\bigl[\sup(A) < \sup(B) \implies \exists B \in B[\forall a \in A(b \geq a)]\bigr] \implies \sup(A) \in B ~ \text{ or } ~ B \cap A \neq \emptyset$

Assume that there exists $b \in B$ such that $b$ is an upper bound for $A$.

We must show that either $\sup(A) \in B$ or $B \cap A \neq \emptyset$.

1\. Since $b$ is an upper bound for $A$, we know:

$$
b \geq a \quad \forall a \in A.
$$

2\. **Case 1: $\sup(A) \in B$:**
- If $\sup(A) \in B$, then $\sup(A)$ itself can act as the element $b \in B$ that is an upper bound for $A$.
- Thus, the condition $\sup(A) \in B$ is satisfied.

3\. **Case 2: $B \cap A \neq \emptyset$:**
- If $B \cap A \neq \emptyset$, then there must exist some overlap between $B$ and $A$ such that elements of $B$ can serve as upper bounds for $A$.
- Therefore, $B \cap A \neq \emptyset$, ensuring the existence of such a $b \in B$.

Hence, either $\sup(A) \in B$ or $B \cap A \neq \emptyset$.

###### $\sup(A) \in B ~ \text{ or } ~ B \cap A \neq \emptyset \implies \bigl[\sup(A) < \sup(B) \implies \exists B \in B[\forall a \in A(b \geq a)]\bigr]$

Assume that $\sup(A) \in B$ or $B \cap A \neq \emptyset$.

We must show that there exists $b \in B$ such that $b$ is an upper bound for $A$.

1\. **Case 1: $\sup(A) \in B$:**
- By the definition of $\sup(A)$, we know that:

$$
\sup(A) \geq a \quad \forall a \in A.
$$

- Since $\sup(A) \in B$, we can choose $b = \sup(A)$, which is an element of $B$.
- Therefore, $b \in B$ is an upper bound for $A$.

2\. **Case 2: $B \cap A \neq \emptyset$:**
- If $B \cap A \neq \emptyset$, let $b \in B \cap A$.
- Then $b \in A$, so $b \geq a$ for all $a \in A$, since $b$ itself is a member of $A$.
- Thus, $b \in B$ is an upper bound for $A$.

###### Conclusion

We have shown that $\sup(A) \leq \sup(B)$ implies the existence of $b \in B$ as an upper bound for $A$ **if and only if**:

$$
\sup(A) \in B \quad \text{or} \quad B \cap A \neq \emptyset.
$$

$\boxed{\text{Q.E.D.}}$

##### Example

Let:

$$
A = [0, 1], \quad B = [1, 2].
$$

1. Compute the suprema:
   - $\sup(A) = 1$,
   - $\sup(B) = 2$.

2. Check the conditions:
   - $\sup(A) \notin B$, because $1 \notin [1, 2]$,
   - $B \cap A = \emptyset$, because $A = [0, 1]$ and $B = [1, 2]$ are disjoint.

3. Therefore, no $b \in B$ can serve as an upper bound for $A$, even though $\sup(A) \leq \sup(B)$.

---

## Exercise 1.31

Suppose that $A, B \subseteq \mathbb{R}$ are nonempty and bounded above. Find a formula for $\sup(A \cup B)$ and prove that it is correct.

### Find and Prove a Formula for $\sup(A \cup B)$

#### Formula

$$
\sup(A \cup B) = \max\{\sup(A), \sup(B)\}.
$$

#### Proof

1. Since $A, B \subseteq \mathbb{R}$ are bounded above, $\sup(A)$ and $\sup(B)$ exist.
2. Let $M = \max\{\sup(A), \sup(B)\}$. Clearly:
   - $M \geq \sup(A)$, so $M \geq a$ for all $a \in A$,
   - $M \geq \sup(B)$, so $M \geq b$ for all $b \in B$,
   - Thus, $M$ is an upper bound for $A \cup B$.

3. To show $M$ is the least upper bound:
   - Suppose $M' < M$.
   - Then $M' < \sup(A)$ or $M' < \sup(B)$, meaning $M'$ cannot be an upper bound for $A$ or $B$, and hence not for $A \cup B$.
   - Thus, $M = \sup(A \cup B)$.

$$
\boxed{\text{Q.E.D.}}
$$

---

## Exercise 1.32

Suppose $A \subseteq \mathbb{R}$ is bounded above and $c \in \mathbb{R}$. Define $c + A = \{c + a: a \in A\}$ and $cA = \{ca: a \in A\}$

### Exercise 1.32.1. Prove that $\sup(c + A) = C + \sup(A)$

#### Proof

1\. Let $M = \sup(A)$. Then $M \geq a$ for all $a \in A$.

2\. For any $a \in A$, $c + a \in c + A$, so $c + M$ is an upper bound for $c + A$.

3\. To show $c + M$ is the least upper bound:
- For any $\varepsilon > 0$, there exists $a \in A$ such that $M - \varepsilon < a$.
- Adding $c$ to both sides:

$$
c + M - \varepsilon < c + a.
$$

- Thus, $c + M$ is the least upper bound for $c + A$.

$$
\boxed{\sup(c + A) = c + \sup(A).}
$$

### Exercise 1.32.2. Determine Necessary and Sufficient Conditions on $c$ and $A$ for $\sup(cA) = C \sup(a)$

Give an example of a set $A$ and number $c$ where $\sup(cA) \ne c \sup(a)$.

1\. If $c > 0$:
- Multiplying by $c$ preserves the order of the elements in $A$.
- Hence, $\sup(cA) = c \sup(A)$.

2\. If $c < 0$:
- Multiplying by $c$ reverses the order of the elements in $A$, so $\sup(cA) = c \inf(A)$.

3\. If $c = 0$, then $cA = \{0\}$, so $\sup(cA) = 0$.

---

## Exercise 1.33

For $A \subseteq \mathbb{R}$, we denote $-A$ to be the set obtained by taking the opposite of everything in $A$. That is,

$$
-A := \{-x : x \in A\}
$$

Suppose that $A \ne \emptyset$ and that $A$ is bounded below. Prove that $-A \ne \emptyset$, $-A$ is bounded above, and $\sup(-A) = - \inf(A)$.

### Prove that $-A \ne \emptyset$, $-A$ is Bounded Above, and $\sup(-A) = -\inf(A)$

**Proof:**

1\. Since $A \neq \emptyset$, for any $a \in A$, $-a \in -A$. Thus, $-A \neq \emptyset$.

2\. Since $A$ is bounded below, there exists $m = \inf(A)$.

3\.This means $m \leq a$ for all $a \in A$.

4\. For any $a \in A$, $-m \geq -a$. Hence, $-m$ is an upper bound for $-A$, so $-A$ is bounded above.

5\. To show $\sup(-A) = -\inf(A)$:
- For any $\varepsilon > 0$, there exists $a \in A$ such that $m \leq a < m + \varepsilon$.
- Then $-m \geq -a > -m - \varepsilon$, so $-m$ is the least upper bound for $-A$.

$$
\boxed{\sup(-A) = -\inf(A).}
$$

---

## Exercise 1.34

For each $n \in \mathbb{N}$, assume we are given a closed interval $I_n = [a_n, b_n]$. Also, assume that each $I_{n+1}$ is contained inside of $I_n$. This gives a sequence of increasingly smaller intervals,

$$
I_1 \supseteq I_2 \supseteq I_3 \supseteq I_4 \supseteq \dots
$$

Prove that $\bigcap_{n=1}^{\infty} I_n \ne \emptyset$. That is, prove that there is some real number $x$ such that $x \in I_n$ for every $n \in \mathbb{N}$.

---

## Exercise 1.35

Give an example showing that the conclusion of Exercise 1.34 need not hold if each $I_n$ is allowed to be an open interval.

---

## Exercise 1.36

For $A, B \subseteq \mathbb{R}$, we define

$$
A + B = \{a + b: a \in A \text{ and } b \in B\}
$$

1\. Determine $\{1, 3, 5\} + \{-3, 0, 1\}$

2\. Assume that $A, B \subseteq \mathbb{R}$ and $\sup(a)$ and $\sup(b)$ exist. Prove that $\sup(A + B) = \sup(a) + \sup(b)$.

---

## Exercise 1.37

For $A, B \subseteq \mathbb{R}$, we define

$$
A \cdot B = \{a \cdot b : a \in A \text{ and } b \in B\}
$$

1\. Determine $\{1, 3, 5\} \cdot \{-3, 0, 1\}$

2\. Give an example of sets $A$ and $B$ where $\sup(A \cdot B) \ne \sup(a) \cdot \sup(b)$.

---

## Question 1

Consider the set

$$
\left\{ \left(\frac{3}{2}\right)^2, \left(\frac{3}{2}\right)^3, \left(\frac{3}{2}\right)^4, \dots \right\}
$$

but for each number remove the integer portion; for example, $(3/2)^3 = 3.375$ would be reduced to 0.375. Is the resulting set dense in $[0, 1]$?

---

## Question 2

Is $e + \pi$ rational?

---

## Question 3

For points $x$ and $y$ in the plane, define $d(x, y)$ to be the distance between $x$ and $y$. Does there exist a dense subset $S$ of $\mathbb{R}^2$ where, for any $x, y \in S$, $d(x, y) \in \mathbb{Q}$?

---

## Question 4

Can every rational number $x$ be represented as a quotient of **shifted primes**? That is, do there exist primes $p$ and $q$ such that

$$
x = \frac{p + 1}{q + 1}
$$
