---
title: sum_03_sequences_real_analysis_full
uuid: 39fa55b5-a258-48ba-b521-203cd445eaf4
aliases:
  - "Full Summary of Real Analysis: Sequences"
  - "full summary of real analysis: sequences"
  - full_summary_of_real_analysis_sequences
  - sum_03_sequences_real_analysis_full
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
  - "[[03_sequences_real_analysis|Real Analysis: Sequences]]"
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
# Full Summary of Real Analysis: Sequences

> [!Summary]
>
> - **Resource**: `dv: this.file.frontmatter.library[0]`
>
> - **Source**:: [[Cummings_2019_Real Analysis_03_Sequences.pdf|Real Analysis: Sequences, by Jay Cummings]]
>
> - **Parent**:: [[sum_03_sequences_real_analysis|Summary of Real Analysis: Sequences]]

---

## 3.1 Basic Sequence Definitions

1\. What is the formal definition of a sequence in terms of functions?
- A sequence of real numbers is a function from the natural numbers to the real numbers.

2\. How do we write sequences in subscript notation, and why is this notation useful?

3\. What are examples of sequences given explicitly versus recursively? How do they differ?

### Key Terms

#### Sequence Definition (Page 65, 3.1)

A sequence of real numbers is a function

$$
a \colon \mathbb{N} \to \mathbb{R}
$$

##### Note on Sequence Notation (Page 66, 3.2)

- **Notation**: Sequences are often written as $a_{n}$ instead of $a(n).$
- **Representation**: A sequence can be denoted in various forms, such as:
  - $(a_{n})_{n=1}^\infty,$ $(a_{n}),$ $(a_{1}, a_{2}, a_{3}, \dots),$ or simply $a_{1}, a_{2}, a_{3}, \dots.$
- **Formulas**:
  - Sequences can be defined **explicitly**, e.g., $a_{n} = n^{2}.$
  - They can also be defined **recursively**, e.g., the Fibonacci sequence:

$$
a_{n} = a_{n-1} + a_{n-2}, \text{ with } a_{1} = 1 \text{ and } a_{2} = 1.
$$

- **General Case**: Any function $f: \mathbb{N} \to \mathbb{R}$ creates a sequence, even without a simple formula. Example:

$$
  (14.238, 7, \pi, -\sqrt{2}, e, 0, e^\pi, \dots)
$$

## 3.2 Bounded Sequences

1\. What does it mean for a sequence to be bounded?
- A bounded sequence is a sequence with an upper and lower bound such that $L \leq a_{n} \leq U$

2\. What is Proposition 3.5, and how does it relate boundedness to the absolute value of sequence terms?

3\. How can you verify boundedness using examples like $a_{n} = (-1)^{n}$ or $a_{n} = 2 + \sin(n)$?

### Key Terms

#### Bounded Sequence (Page 67, 3.4)

A sequence $\langle x_{n} \rangle$ is **bounded** if the range $\{x_{n}: n \in \mathbb{N}\}$ is bounded.

That is, if there exists a lower bound $L \in \mathbb{R}$ and an upper bound $U \in \mathbb{R},$ such that:

$$
L \leq x_{n} \leq U \quad \text{for all } n \in \mathbb{N}.
$$

or, in logical notation:

$$
\exists (L \in \mathbb{R}), \exists (U \in \mathbb{R}), \forall (n \in \mathbb{N}) [ L \leq x_{n} \leq U]
$$

#### Bounded $\iff |x_{n}| \leq C$ (Page 67, 3.5)

A sequence $\langle x_{n} \rangle$ is **bounded** if and only if there exists some $C \in \mathbb{R}$ such that $|a_{n}| \leq C$ for all $n.$

---

##### Proof

To prove the proposition, we will demonstrate both directions of the equivalence:

###### 1. Backward Implication ($|a_{n}| \leq C \to a_{n} \text{ is bounded}$)

**Assumption**: There exists $C \in \mathbb{R}$ such that $|a_{n}| \leq C$ for all $n.$

1\. By definition of absolute value:

$$
-C \leq a_{n} \leq C, \quad \forall n \in \mathbb{N}.
$$

2\. **Set Upper and Lower Bounds**:

2.1. Let $L = -C$ and $U = C$

2.2. Then:

$$
L \leq a_{n} \leq U.
$$

$$
\therefore \, a_{n} \text{ is bounded}.
$$

###### 2. Forward Implication ($a_{n} \text{ is bounded} \to |a_{n}| \leq C$)

**Assumption**: $a_{n}$ is bounded, so there exist constants $L, U \in \mathbb{R}$ such that:

$$
L \leq a_{n} \leq U, \quad \forall n \in \mathbb{N}.
$$

1\. **Define $C$:**

1.1. Let $C = \max(|L|, |U|).$

1.2. This implies that

$$
C \geq U \quad \text{and} \quad -C \leq -|L| \quad(\text{because } C \geq |L|).
$$

2\. **Bounding $a_{n}$:**

2.1. Combining $L \leq a_{n} \leq U$ with $-C \leq L$ and $U \leq C,$ we get:

$$
-C \leq -|L| \leq L \leq a_{n} \leq U \leq C
$$

which equals:

$$
-C \leq a_{n} \leq C, \quad \forall n \in \mathbb{N}.
$$

3\. **Absolute Value**: Taking absolute values, this simplifies to:

$$
|a_{n}| \leq C, \quad \forall n \in \mathbb{N}.
$$

$$
\therefore \, |a_{n}| \leq C.
$$

###### Conclusion

Both directions have been proven:
1\. $|a_{n}| \leq C \to a_{n} \text{ is bounded.}$
2\. $a_{n} \text{ is bounded} \to |a_{n}| \leq C.$

Thus, the equivalence $a_{n} \text{ is bounded} \Leftrightarrow |a_{n}| \leq C$ is established. $\Box$

## 3.3 Convergent Sequences

1\. What is the precise definition of a convergent sequence, and why is it important?

2\. How does the $\varepsilon$-$\delta$ approach describe convergence?

3\. What techniques can be used to prove that specific sequences, such as $a_{n} = \frac{1}{n}$ or $a_{n} = 3 - \frac{5}{n+2},$ converge?

4\. What is the significance of $\varepsilon$-neighborhoods in the context of convergence?

### Key Terms

#### Convergent Sequence (Page 69, 3.7)

A sequence $\langle x_{n} \rangle$ *converges* to $L \in \mathbb{R}$ if for all $\varepsilon > 0$ there exists some $N$ such that $|x_{n} - L| < \varepsilon$ for all $n > N.$

##### Limit of a Sequence

When a sequence $\langle x_{n} \rangle$ converges to $L \in \mathbb{R},$ $L$ is referred to as the limit of $\langle x_{n} \rangle.$

#### Outline for Solution to Sequence Convergence of $\lim\limits_{ n\to \infty }a_{n} \to a$ (Page 72, 3.10)

1\. Preliminary Scratch Work: Finding $N$
   1. Start with $|a_{n} - a| < \varepsilon.$
   2. Unravel to solve for $n.$
2\. Proof:
   3. Let $\varepsilon > 0.$
   4. Let $N$ equal the final value of $n$ from the scratch work and let $n > N.$
   5. Redo scratch work without $\varepsilon,$ but at the end, use $N$ to show that $|a_{n} - a| < \varepsilon.$

#### $\varepsilon$-Neighborhood (Page 76, 3.14)

Let $\varepsilon > 0.$ The $\varepsilon$-neighborhood of a point $a$ is the interval

$$
(a - \varepsilon, a + \varepsilon)
$$

##### $\varepsilon$-Neighborhood Convergent Sequence (Page 77, 3.7)

A sequence $\langle x_{n} \rangle$ *converges* to $L \in \mathbb{R}$ if for all $\varepsilon > 0$ there exists some $N$ such that $x_{n}$ is in the $\varepsilon$-neighborhood of $L$ for all $n > N.$

## 3.4 Divergent Sequences

1\. What are the three main types of divergence, and how are they defined?

2\. How can you prove that a sequence like $a_{n} = n^{2}$ diverges to infinity?

3\. How can you use the negation of the definition of convergence to show that a sequence like $a_{n} = (-1)^{n}$ diverges?

### Key Terms

#### Divergent Sequence (Page 77, 3.15)

If a sequence $\langle x_{n} \rangle$ does not converge, then it diverges.

##### Three Forms of Divergent Sequences

- $\langle x_{n} \rangle$ diverges to $\infty$ (notation: $\lim\limits_{ n \to \infty } x_{n} = \infty$) if, for all $M > 0,$ there exists some $N$ such that $x_{n} > M$ for all $n > N.$
- $\langle x_{n} \rangle$ diverges to $-\infty$ (notation: $\lim\limits_{ n \to \infty } x_{n} = -\infty$) if, for all $M < 0,$ there exists some $N$ such that $x_{n} < M$ for all $n > N.$
- The limit of $\langle x_{n} \rangle$ does not exist.

#### Negation of Convergence Proof of Divergent Sequences (Page 78, 3.17)

One way to show that $a_{n}$ diverges is to show that $a_{n} \not \to a$ for any $a.$

Note first, by Definition 3.7, that $a_{n} \to a$ means that:

$$
\text{For every } \varepsilon > 0, \text{ there exists some } N \text{ such that for all } n > N, \, |a_{n} - a| < \varepsilon.
$$

So to show that $a_{n} \not \to a,$ we need to show the negation of that statement. That is, we must show that:

$$
\text{There exists some } \varepsilon > 0 \text{ where for all } N, \text{ there exists some } n > N \text{ such that } |a_{n} - a| \geq \varepsilon.
$$

> [!Note]
>
> In practice, this is usually done with a proof by contradiction. You assume that $a_{n} \to a,$ and then you demonstrate a specific $\varepsilon$ where it fails, giving the contradiction.

#### Uniqueness of Limits (Page 81-83, 3.19)

A sequence cannot have more than one limit.

##### Proof

Let $\langle x_{n} \rangle$ be a sequence, and assume $\lim\limits_{ n \to \infty } x_{n} = a$ and $\lim\limits_{ n \to \infty } x_{n} \to b,$ where $a, b \in \mathbb{R}.$ We will show that $a = b.$

1\. Let $\varepsilon > 0.$ By definition of convergence:
- Since $x_{n} \to a,$ there exists some $N_{1} \in \mathbb{N}$ such that for all $n > N_{1},$ $|x_{n} - a| < \frac{\varepsilon}{2}.$
- Since $x_{n} \to b,$ there exists some $N_{2} \in \mathbb{N}$ such that for all $n > N_{2},$ $|x_{n} - b| < \frac{\varepsilon}{2}.$

2\. Let $N = \max(N_{1}, N_{2}),$ and consider any $n > N.$ Then:
- For $n > N,$ we have $n > N_{1}$ and $n > N_{2},$ so both $|x_{n} - a| < \frac{\varepsilon}{2}$ and $|x_{n} - b| < \frac{\varepsilon}{2}$ hold.

3\. Using the triangle inequality:

$$
|a - b| = |(a - x_{n}) + (x_{n} - b)| \leq |a - x_{n}| + |x_{n} - b|.
$$

4\. By symmetry of absolute value:

$$
|a - x_{n}| + |x_{n} - b| = |x_{n} - a| + |x_{n} - b|
$$

5\. $n > N$ implies $n > N_{1}$ and $n > N_{2}$:

$$
|x_{n} - a| + |x_{n} - b| < \frac{\varepsilon}{2} + \frac{\varepsilon}{2}
$$

6\. Substituting the bounds:

$$
|a - b| \leq |x_{n} - a| + |x_{n} - b| < \frac{\varepsilon}{2} + \frac{\varepsilon}{2} = \varepsilon.
$$

7\. Since this holds for any $\varepsilon > 0,$ it follows that $|a - b| = 0.$ Hence, $a = b.$

The sequence $\langle x_{n} \rangle$ can converge to only one limit. Therefore, limits of sequences are unique. $\Box$

#### Convergent Sequences Are Bounded (Page 83, 3.20)

If $\langle x_{n} \rangle$ is a convergent sequence, then $x_{n}$ is bounded.

##### Proof

1\. Given a convergent sequence $x_{n},$ let $x$ be the value to which it is converging, such that

$$
\lim_{ n \to \infty } x_{n} = x
$$

2\. By definition of convergence, if $\varepsilon = 1,$ then

$$
\exists N \in \mathbb {N} \bigl[\forall n \in \mathbb {N} \left(n \geq N \implies |x_{n}-x| < 1 \right)\bigr],
$$

which implies

$$
x - 1 < x_{n} < x + 1, \quad \forall n > N
$$

3\. Let

$$
\begin{gather}
U = \max\{ x_{1}, x_{2}, \ldots, x_{N}, x + 1 \} \\
\text{and} \\
L = \min\{ x_{1}, x_{2}, \ldots, x_{N}, x - 1 \}
\end{gather}
$$

4\. Note that each $x_{n}$ is included in the sets of which we are taking minimum and maximum.

5\. Because $x_{n}$ is in both sets, if $n \leq N,$ then $L \leq x_{n} \leq U.$

6\. If $n > N,$ then $x - 1 < x_{n} < x + 1,$ which implies

$$
\begin{align}
&L \leq x - 1 < x_{n} < x + 1 \leq U \\
& \quad \implies L \leq x_{n} \leq U \end{align}
$$

for all $n$

$$
\therefore \quad \boxed{x_{n} \text{ is bounded}}
$$

##### Proof Using $|x_{n}| \leq C$

1\. Given a convergent sequence $x_{n},$ let $x$ be the value to which it is converging, such that

$$
\lim_{ n \to \infty } x_{n} = L
$$

2\. By definition of convergence, given $\varepsilon > 0,$ then there exists an $N \in \mathbb{N}$ for all $n \in \mathbb{N},$ if $n \geq N,$ then

$$
\begin{gather}
|x_{n} - L| < \varepsilon \\
-\varepsilon < x_{n} - L < \varepsilon \\
L - \varepsilon < x_{n}< L + \varepsilon \end{gather}
$$

3\. Let

$$
C = \max\{ |x_{1}|, |x_{2}|, \ldots, |x_{N}|, |x + L|, |x - L| \}
$$

4\. Note that each term, $x_{n},$ is included in the set above.

5\. If $n \leq N,$ then $x_{n} \leq |x_{n}| \leq C.$

6\. If $n > N,$ then $L - \varepsilon < x_{n}< L + \varepsilon,$ which implies

$$
|x_{n}| \leq \max \{ |x + L|, |x - L| \} \leq C.
$$

7\. Thus, for all $n,$ $|x_{n}| \leq C$

$$
\therefore \quad \boxed{x_{n} \text{ is bounded}}
$$

## 3.5 Limit Laws

1\. What are the fundamental limit laws for sequences, and how are they used?

2\. How does the proof for the product of limits ($\lim (a_{n} b_{n}) = (\lim a_{n})(\lim b_{n})$) work?

3\. How can the limit laws be applied to compute complex limits, such as $\lim_{n \to \infty} \frac{3n+1}{n+2}$?

### Key Terms

#### Limit Laws of Convergent, Real Sequences (Page 85, 3.21)

Given convergent real sequences $\langle a_{n} \rangle$ and $\langle b_{n} \rangle,$ such that

$$
\begin{align}
\lim_{ n \to \infty } a_{n} &= a \\
\lim_{ n \to \infty } b_{n} &= b \end{align}
$$

then the following hold:

1\. Sum Law: $\lim\limits_{n \to \infty }(a_{n} + b_{n}) = a + b$
2\. Difference Law: $\lim\limits_{n \to \infty }(a_{n} - b_{n}) = a - b$
3\. Product Law: $\lim\limits_{ n \to \infty }(a_{n} \cdot b_{n}) = a \cdot b$
4\. Quotient Law: $\lim\limits_{ n \to \infty }\left(\frac{a_{n}}{b_{n}} \right) = \frac{a}{b}, \text{ if } \lim\limits_{ n \to \infty }b_{n}\neq 0$
5\. Multiple Law: $\lim\limits_{ n \to \infty }(c \cdot a_{n}) = c \cdot a$
6\. Exponent Law: $\lim\limits_{ n \to \infty }(a_{n})^{p} = a^{p}$

##### Proof: Sum Law

1\. Let $\langle a_{n} \rangle, \langle b_{n} \rangle$ be convergent sequences such that:

$$
\lim_{n \to \infty} a_{n} = A \quad \text{and} \quad \lim_{n \to \infty} b_{n} = B.
$$

2\. By definition of sequence **convergence**, given any $\varepsilon > 0,$
- $a_{n} \to A$ implies there exists an $N_{1} \in \mathbb{N}$ such that for all $n \geq N_{1},$ $|a_{n} - A| < \frac{\varepsilon}{2}.$
- $b_{n} \to B$ implies there exists an $N_{2} \in \mathbb{N}$ such that for all $n \geq N_{2},$ $|b_{n} - B| < \frac{\varepsilon}{2}.$

3\. Let $N = \max(N_{1}, N_{2}).$ For all $n \geq N,$ both inequalities hold:

$$
|a_{n} - A| < \frac{\varepsilon}{2} \quad \text{and} \quad |b_{n} - B| < \frac{\varepsilon}{2}.
$$

4\. For $n \geq N,$ consider the sum $a_{n} + b_{n}$:

$$
|(a_{n} + b_{n}) - (A + B)| = |(a_{n} - A) + (b_{n} - B)|.
$$

5\. By the **triangle inequality for absolute values**:

$$
|(a_{n} - A) + (b_{n} - B)| \leq |a_{n} - A| + |b_{n} - B|.
$$

6\. Substituting the bounds for $|a_{n} - A|$ and $|b_{n} - B|$:

$$
|(a_{n} + b_{n}) - (A + B)| < \frac{\varepsilon}{2} + \frac{\varepsilon}{2} = \varepsilon.
$$

7\. Thus, By definition of sequence **convergence**:

$$
a_{n} + b_{n} \to A + B
$$

$$
\therefore ~ \boxed{\lim_{n \to \infty} (a_{n} + b_{n}) = A + B.}
$$

##### Proof: Difference Law

1\. Let $\langle a_{n} \rangle$ and $\langle b_{n} \rangle$ be convergent sequences such that:

$$
\lim_{n \to \infty} a_{n} = A \quad \text{and} \quad \lim_{n \to \infty} b_{n} = B.
$$

2\. By definition of sequence **convergence**, given any $\varepsilon > 0$:
- $\langle a_{n} \rangle \to A$ implies $\exists N_{1} \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that $n \geq N_{1} \implies |a_{n} - A| < \frac{\varepsilon}{2}.$
- $\langle b_{n} \rangle \to B$ implies $\exists N_{2} \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that $n \geq N_{2} \implies |b_{n} - B| < \frac{\varepsilon}{2}.$

3\. Let $N = \max(N_{1}, N_{2}).$ For all $n \geq N,$ both inequalities hold:

$$
|a_{n} - A| < \frac{\varepsilon}{2} \quad \text{and} \quad |b_{n} - B| < \frac{\varepsilon}{2}.
$$

4\. For $n \geq N,$ consider the difference $a_{n} - b_{n}$:

$$
\begin{align}
|(a_{n} - b_{n}) - (A - B)| &= |(a_{n} - A) - (b_{n} - B)| \\
&= |(a_{n} - A) + (-(b_{n} - B))|
\end{align}
$$

5\. By the **triangle inequality** for absolute values:

$$
|(a_{n} - A) + (-(b_{n} - B))| \leq |a_{n} - A| + |b_{n} - B|.
$$

6\. Substituting the bounds for $|a_{n} - A|$ and $|b_{n} - B|$:

$$
\begin{align}
|(a_{n} - b_{n}) - (A - B)| &\leq |a_{n} - A| + |b_{n} - B|  \\
&< \frac{\varepsilon}{2} + \frac{\varepsilon}{2} = \varepsilon.
\end{align}
$$

7\. Thus, By definition of sequence **convergence**:

$$
a_{n} - b_{n} \to A - B.
$$

$$
\therefore ~ \boxed{\lim_{n \to \infty} (a_{n} - b_{n}) = A - B.}
$$

##### Proof: Product Law (*Real Analysis* with revisions)

1\. Let $\langle a_{n} \rangle, \langle b_{n} \rangle$ be convergent sequences such that:

$$
\lim_{n \to \infty }a_{n} = A ~\text{ and }~ \lim_{n \to \infty }b_{n} = B
$$

2\. Since **convergent sequences are bounded**, for all $n \in \mathbb{N},$ there exists $C > 0$ such that

$$
|b_{n}| \leq C \tag{1}
$$

3\. Let $\varepsilon > 0$ and choose $\varepsilon_{1} = \frac{\varepsilon}{2(|A| + 1)}$ and $\varepsilon_{2} = \frac{\varepsilon}{2(C + 1)}.$

> [!Note]
>
> The additional $+1$ ensures we are not dividing by $0.$

4\. By definition of sequence **convergence**, $\forall \varepsilon > 0$:

- $a_{n} \to A$ implies there exists $N_{1} \in \mathbb{N}$ such that for all $n \geq N_{1}$:

$$
|a_{n} - A| < \frac{\varepsilon}{2(|A| + 1)} \tag{2}
$$

- $b_{n} \to B$ implies there exists $N_{2} \in \mathbb{N}$ such that for all $n \geq N_{2}$:

$$
|b_{n} - B| < \frac{\varepsilon}{2(C + 1)} \tag{3}
$$

5\. Define $N = \max(N_{1}, N_{2})$ so that for all $n \geq N,$ both inequalities **(2) and (3) hold**.

6\. For any $n \geq N,$ consider the expression:

$$
\begin{align}
|a_{n} \cdot b_{n} - A \cdot B| &= |a_{n} b_{n} - Ab_{n} + Ab_{n} - AB)| \\
&= |b_{n}(a_{n} - A) + A(b_{n} - B)|
\end{align}
$$

7\. By the **triangle inequality** of absolute values:

$$
|b_{n}(a_{n} - A) + A(b_{n} - B)| \leq |b_{n}(a_{n} - A)| + |A(b_{n} - B)|
$$

8\. By the **complete multiplicativity** of absolute values:

$$
|b_{n}(a_{n} - A)| + |A(b_{n} - B)| = |b_{n}| \cdot |(a_{n} - A)| + |A| \cdot |b_{n} - B|
$$

9\. Substituting $C$ for $|b_{n}|$ (1) and the bounds of (2) and (3):

$$
\begin{align}
|a_{n} \cdot b_{n} - A \cdot B| &\leq |b_{n}| \cdot |(a_{n} - A)| + |A| \cdot |b_{n} - B|  \\
&< C \cdot \frac{\varepsilon}{2(C + 1)} + |A| \cdot \frac{\varepsilon}{2(|A| + 1)} \\
&= \frac{\varepsilon}{2} + \frac{\varepsilon}{2} = \varepsilon
\end{align}
$$

10\. Thus, By definition of sequence **convergence**, $(a_{n} \cdot b_{n}) \to A \cdot B,$ since $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that

$$
n \geq N \implies |a_{n} \cdot b_{n} - A \cdot B| < \varepsilon,
$$

$$
\therefore ~ \boxed{\lim_{n \to \infty} (a_{n} \cdot b_{n}) = A \cdot B}
$$

---

##### Quotient Law for Convergent Sequences

**Theorem:**
If the limits of the sequences $\langle a_{n} \rangle$ and $\langle b_{n} \rangle$ are convergent, such that:

$$
\lim_{n \to \infty} a_{n} = A \quad \text{and} \quad \lim_{n \to \infty} b_{n} = B,
$$

then:

$$
\lim_{n \to \infty} \frac{a_{n}}{b_{n}} = \frac{A}{B},
$$

provided that $B \neq 0.$

---

###### Proof

1\. Let $\varepsilon > 0$ be given. We want to show that:

$$
\forall \varepsilon > 0, \exists N \in \mathbb{N} \text{ such that } n \geq N \implies \left| \frac{a_{n}}{b_{n}} - \frac{A}{B} \right| < \varepsilon.
$$

2\. **Rewrite the Expression:** Using algebraic manipulation and the triangle inequality, we can rewrite:

$$
\left| \frac{a_{n}}{b_{n}} - \frac{A}{B} \right| = \left| \frac{a_{n}}{b_{n}} - \frac{A}{b_{n}} + \frac{A}{b_{n}} - \frac{A}{B} \right|.
$$

Group terms:

$$
\left| \frac{a_{n}}{b_{n}} - \frac{A}{B} \right| = \left| \frac{a_{n} - A}{b_{n}} + \frac{A}{b_{n}} \cdot \frac{B - b_{n}}{B} \right|.
$$

3\. By the triangle inequality:

$$
\left| \frac{a_{n}}{b_{n}} - \frac{A}{B} \right| \leq \left| \frac{a_{n} - A}{b_{n}} \right| + \left| \frac{A}{b_{n}} \cdot \frac{B - b_{n}}{B} \right|.
$$

4\. **Bound the First Term:** Since $\langle b_{n} \rangle$ converges to $B,$ there exists an $N_{1}$ such that for all $n \geq N_{1}$:

$$
|b_{n}| > \frac{|B|}{2}.
$$

Substituting this bound into the first term:

$$
\left| \frac{a_{n} - A}{b_{n}} \right| \leq \frac{|a_{n} - A|}{|b_{n}|} \leq \frac{|a_{n} - A|}{\frac{|B|}{2}} = \frac{2}{|B|} |a_{n} - A|.
$$

5\. **Bound the Second Term:** For the second term:

$$
\left| \frac{A}{b_{n}} \cdot \frac{B - b_{n}}{B} \right| = \left| \frac{A}{b_{n}} \right| \cdot \left| \frac{B - b_{n}}{B} \right|.
$$

Using the bound $|b_{n}| > \frac{|B|}{2},$ we have:

$$
\left| \frac{A}{b_{n}} \right| \leq \frac{|A|}{\frac{|B|}{2}} = \frac{2|A|}{|B|}.
$$

Thus:

$$
\left| \frac{A}{b_{n}} \cdot \frac{B - b_{n}}{B} \right| \leq \frac{2|A|}{|B|} \cdot \frac{|B - b_{n}|}{|B|} = \frac{2|A|}{B^{2}} \cdot |B - b_{n}|.
$$

6\. **Combine the Bounds:** Adding the two bounds, we have:

$$
\left| \frac{a_{n}}{b_{n}} - \frac{A}{B} \right| \leq \frac{2}{|B|} |a_{n} - A| + \frac{2|A|}{B^{2}} |B - b_{n}|.
$$

7\. **Convergence and Final Bounds:** Since $a_{n} \to A$ and $b_{n} \to B,$ for any $\varepsilon > 0,$ choose $N = \max(N_{1}, N_{2})$ such that:
- $|a_{n} - A| < \frac{\varepsilon |B|}{4},$
- $|b_{n} - B| < \frac{\varepsilon B^{2}}{4|A|}.$

Substituting these bounds:

$$
\left| \frac{a_{n}}{b_{n}} - \frac{A}{B} \right| < \frac{\varepsilon}{2} + \frac{\varepsilon}{2} = \varepsilon.
$$

8\. Therefore, we have shown:

$$
\lim_{n \to \infty} \frac{a_{n}}{b_{n}} = \frac{A}{B}.
$$

###### **Proof: Quotient Law for Limits** (Alternate)

1\. Let $\langle a_{n} \rangle, \langle b_{n} \rangle$ be convergent sequences such that:

$$
\lim_{n \to \infty }a_{n} = A ~\text{ and }~ \lim_{n \to \infty }b_{n} = B, \quad B \neq 0.
$$

2\. By definition of sequence **convergence**, $\forall \varepsilon > 0$:

- Since $a_{n} \to A,$ there exists $N_{1} \in \mathbb{N}$ such that for all $n \geq N_{1}$:

$$
|a_{n} - A| < \varepsilon_{1} \tag{1}
$$

- Since $b_{n} \to B,$ there exists $N_{2} \in \mathbb{N}$ such that for all $n \geq N_{2}$:

$$
|b_{n} - B| < \varepsilon_{2} \tag{2}
$$

3\. Define $N = \max(N_{1}, N_{2})$ so that for all $n \geq N,$ both inequalities **(1) and (2) hold**.

4\. For any $n \geq N,$ consider the difference:

$$
\left| \frac{a_{n}}{b_{n}} - \frac{A}{B} \right| = \left| \frac{a_{n}}{b_{n}} - \frac{A}{b_{n}} + \frac{A}{b_{n}} - \frac{A}{B} \right|
$$

5\. By **rearranging terms** and applying the **triangle inequality**:

$$
\left| \frac{a_{n}}{b_{n}} - \frac{A}{B} \right| \leq \left| \frac{a_{n} - A}{b_{n}} \right| + \left| \frac{A}{b_{n}} \cdot \frac{B - b_{n}}{B} \right|.
$$

6\. Since $b_{n} \to B$ and $B \neq 0,$ then for all $n \geq N,$

$$
|b_{n}| > \frac{|B|}{2}.
$$

7\. Bounding the first term:

$$
\begin{align}
\left| \frac{a_{n} - A}{b_{n}} \right| &\leq \frac{|a_{n} - A|}{|b_{n}|}  \\
&\leq \frac{|a_{n} - A|}{\frac{|B|}{2}}  \\
&= \frac{2}{|B|} |a_{n} - A|.
\end{align}
$$

8\. Using the same bound $|b_{n}| > \frac{|B|}{2},$ we estimate:

$$
\left| \frac{A}{b_{n}} \right| \leq \frac{|A|}{\frac{|B|}{2}} = \frac{2|A|}{|B|}.
$$

9\. Therefore:

$$
\begin{align}
\left| \frac{A}{b_{n}} \cdot \frac{B - b_{n}}{B} \right| &\leq \frac{2|A|}{|B|} \cdot \frac{|B - b_{n}|}{|B|}  \\
&= \frac{2|A|}{B^{2}} |B - b_{n}|.
\end{align}
$$

10\. Adding the two bounds together:

$$
\left| \frac{a_{n}}{b_{n}} - \frac{A}{B} \right| \leq \frac{2}{|B|} |a_{n} - A| + \frac{2|A|}{B^{2}} |B - b_{n}|.
$$

11\. Let:

$$
\varepsilon_{1} = \frac{\varepsilon |B|}{4}, \quad \varepsilon_{2} = \frac{\varepsilon B^{2}}{4|A|}
$$

ensuring $\varepsilon_{2} \neq 0.$

12\. **Substituting these bounds**:

$$
\left| \frac{a_{n}}{b_{n}} - \frac{A}{B} \right| < \frac{\varepsilon}{2} + \frac{\varepsilon}{2} = \varepsilon.
$$

13\. By definition of limits, since for all $n \geq N,$

$$
\left| \frac{a_{n}}{b_{n}} - \frac{A}{B} \right| < \varepsilon,
$$

it follows that:

$$
\lim_{n \to \infty} \frac{a_{n}}{b_{n}} = \frac{A}{B}.
$$

$$
\therefore ~ \boxed{\lim_{n \to \infty} \frac{a_{n}}{b_{n}} = \frac{A}{B}}
$$

#### Proof: Scalar Multiplication Law

1\. Let $\langle x_{n} \rangle$ be a convergent sequence such that:

$$
\lim_{n \to \infty} x_{n} = L,
$$

2\. Let $k$ be a real number, $k \in \mathbb{R}.$

3\. By definition of sequence **convergence**, given any $\varepsilon > 0$ there exists an $N \in \mathbb{N}$ such that for all $n \in \mathbb{N},$ if $n \geq N,$ then

$$
\begin{align}
|x_{n} - L| < \frac{\varepsilon}{|k|}
\end{align}
$$

4\. Using the property of absolute values to factor the inequality:

$$
\begin{gather}
|(k \cdot x_{n}) - (k \cdot L)| = |k| \cdot |x_{n} - L| < \frac{\varepsilon}{|k|} = \varepsilon \\
|(k \cdot x_{n}) - (k \cdot L)| < \varepsilon \end{gather}
$$

5\. Thus, By definition of convergence, $k \cdot x_{n} \to k \cdot L$

$$
\therefore ~ \boxed{\lim_{n \to \infty} x_{n} = L}
$$

---

#### Comparison Theorems for Real Sequences ([Math Online Wike](http://mathonline.wikidot.com/comparison-theorems-for-sequences))

1. If $a_{n} \leq b_{n}$ for all $n \geq N$ and $\lim\limits_{ n \to \infty }a_{n} = \infty$ then $\lim\limits_{ n \to \infty }b_{n} = \infty.$
2. If $a_{n} \geq b_{n}$ for all $n \geq N$ and $\lim\limits_{ n \to \infty }a_{n} = -\infty$ then $\lim\limits_{ n \to \infty }b_{n} -\infty.$
3. If $\lim\limits_{ n \to \infty } a_{n} = A,$ $\lim\limits_{ n \to \infty } b_{n} = B,$ and $a_{n} \leq b_{n}$ for all $n \geq N$ then $A \leq B.$

##### Proof: $\forall n\geq N(a_{n} \leq b_{n}) \land \lim\limits_{ n\to \infty } a_{n} = \infty \implies \lim\limits_{ n\to \infty } b_{n} = \infty$

1\. By definition of divergence to infinity, $\lim\limits_{ n \to \infty } a_{n} = \infty$ means:

$$
\forall k \in \mathbb{R}, \exists N \in \mathbb{N}, \forall n \geq N, a_{n} > k.
$$

2\. Since $a_{n} \leq b_{n}$ for all $n \geq N,$ we have:

$$
k < a_{n} \leq b_{n}, \quad \forall n \geq N.
$$

3\. Since $b_{n}$ is also greater than any arbitrarily large real number $k,$ we conclude:

$$
\lim_{n \to \infty} b_{n} = \infty.
$$

$$
\therefore \boxed{\lim_{n \to \infty} a_{n} = \infty \text{ and } a_{n} \leq b_{n} \implies \lim_{n \to \infty} b_{n} = \infty.}
$$

##### Proof: $\forall N \geq N(a_{n} \geq b_{n}) \land \lim\limits_{ n\to \infty } a_{n} = -\infty \implies \lim\limits_{ n\to \infty } b_{n} = -\infty$

1\. By definition of divergence to negative infinity, $\lim\limits_{ n \to \infty }a_{n} = -\infty$ means:

$$
\forall k \in \mathbb{R}, \exists N \in \mathbb{N}, \forall n \geq N, a_{n} < k.
$$

2\. Since $a_{n} \geq b_{n}$ for all $n \geq N,$ we obtain:

$$
b_{n} \leq a_{n} < k, \quad \forall n \geq N.
$$

3\. Since $b_{n}$ is also smaller than any arbitrarily small real number $k,$ we conclude:

$$
\lim_{n \to \infty} b_{n} = -\infty.
$$

$$
\therefore \boxed{\lim_{n \to \infty} a_{n} = -\infty \text{ and } a_{n} \geq b_{n} \implies \lim_{n \to \infty} b_{n} = -\infty.}
$$

##### Proof: $\forall n\geq N(a_{n} \leq b_{n}) \land \lim\limits_{ n\to \infty } a_{n} = A \land \lim\limits_{ n\to \infty }b_{n} = B \implies A \leq B$

1\. Assume for contradiction that $A > B.$

2\. Define $\varepsilon = \frac{A - B}{3}.$

3\. By definition of limits:
- Since $\lim_{n \to \infty} a_{n} = A,$ there exists $N_{1}$ such that for all $n \geq N_{1},$

$$
|a_{n} - A| < \frac{A - B}{3}.
$$

- Similarly, since $\lim_{n \to \infty} b_{n} = B,$ there exists $N_{2}$ such that for all $n \geq N_{2},$

$$
|b_{n} - B| < \frac{A - B}{3}.
$$

4\. Choose $N = \max(N_{1}, N_{2}),$ ensuring that both inequalities hold for all $n \geq N.$

5\. Then for $n \geq N$:

$$
\begin{gather}
-\frac{A - B}{3} < a_{n} - A < \frac{A - B}{3} \\
A - \frac{A - B}{3} < a_{n} \end{gather}
$$

6\. Similarly,

$$
\begin{gather}
-\frac{A - B}{3} < b_{n} - B < \frac{A - B}{3} \\
b_{n} < B + \frac{A - B}{3}
\end{gather}
$$

7\. Since $a_{n} \leq b_{n},$ we obtain:

$$
A - \frac{A - B}{3} < b_{n} < B + \frac{A - B}{3}.
$$

8\. Rearranging,

$$
\begin{align}
A - \frac{A - B}{3} &\leq B + \frac{A - B}{3} \\
3A - (A - B) &\leq 3B + (A - B) \\
2A + B &\leq A + 2B \\
A &\leq B \end{align}
$$

9\. This contradicts our assumption that $A > B,$ so we conclude:

$$
\therefore A \leq B.
$$

$$
\therefore \boxed{\lim_{n \to \infty} a_{n} = A, \lim_{n \to \infty} b_{n} = B, a_{n} \leq b_{n} \implies A \leq B.}
$$

---

### Sequence Squeeze Theorem (Page 87, 3.23)

If $\langle a_{n} \rangle,$ $\langle b_{n} \rangle,$ and $\langle x_{n} \rangle$ are sequences such that:

$$
\begin{gather}
a_{n} \leq x_{n} \leq b_{n} \quad \text{for all } n \\
\\
\text{and} \\
\\
\lim_{n \to \infty} a_{n} = L \quad \text{and}\quad \lim_{n \to \infty} b_{n} = L,
\end{gather}
$$

then:

$$
\lim_{n \to \infty} x_{n} = L.
$$

#### Proof

Let $\varepsilon > 0.$

1\. **Convergence of $\langle a_{n} \rangle$:** Since $a_{n} \to L,$ there exists some $N_{1}$ such that $n > N_{1}$ implies:

$$
|a_{n} - L| < \varepsilon.
$$

That is:

$$
-\varepsilon < a_{n} - L < \varepsilon.
$$

Or equivalently:

$$
L - \varepsilon < a_{n} < L + \varepsilon. \tag{1}
$$

2\. **Convergence of $\langle b_{n} \rangle$:** Since $b_{n} \to L,$ there exists some $N_{2}$ such that $n > N_{2}$ implies:

$$
|b_{n} - L| < \varepsilon.
$$

That is:

$$
-\varepsilon < b_{n} - L < \varepsilon.
$$

Or equivalently:

$$
L - \varepsilon < b_{n} < L + \varepsilon. \tag{2}
$$

3\. Let $N = \max(N_{1}, N_{2}),$ and consider $n > N.$

4\. Combining the inequality $a_{n} \leq x_{n} \leq b_{n}$ with the left-hand side of $(1)$ and the right-hand side of $(2),$ we get:

$$
L - \varepsilon < a_{n} \leq x_{n} \leq b_{n} < L + \varepsilon.
$$

5\. Simplifying the inequality above:

$$
\begin{gather}
L - \varepsilon < x_{n} < L + \varepsilon \\
-\varepsilon < x_{n} - L < \varepsilon \\
|x_{n} - L| < \varepsilon.
\end{gather}
$$

Therefore, $(x_{n} \to L)$ as required. $\Box$

## 3.6 The Monotone Convergence Theorem

1\. What does it mean for a sequence to be monotone?

2\. How does the monotone convergence theorem guarantee convergence for bounded monotone sequences?

3\. How can the theorem be used to prove the convergence of sequences like $a_{n} = 0.1, 0.12, 0.123, \dots$?

### Key Terms

#### Monotone Convergence Theorem (Page 90, 3.27)

Suppose $\langle x_{n} \rangle$ is monotone. Then $\langle x_{n} \rangle$ converges **if and only if it is bounded**. Moreover:

- **If $\langle x_{n} \rangle$ is increasing**, then either $\langle x_{n} \rangle$ diverges to $+\infty,$ or

$$
\lim_{n \to \infty} x_{n} = \sup(\{x_{n}: n \in \mathbb{N}\}).
$$

- **If $\langle x_{n} \rangle$ is decreasing**, then either $\langle x_{n} \rangle$ diverges to $-\infty,$ or

$$
\lim_{n \to \infty} x_{n} = \inf(\{x_{n}: n \in \mathbb{N}\}).
$$

#### Bounded $S$ Contains Sequences Converging to $\sup(S)$ and $\inf(S)$

- **Supremum Approximation**: Suppose $S \subseteq \mathbb{R}$ is bounded above. Then there exists a sequence $(a_{n})$ where $a_{n} \in S$ for each $n$ and:

$$
\lim_{n \to \infty} a_{n} = \sup(S).
$$

- Infimum Approximation: Suppose $S \subseteq \mathbb{R}$ is bounded below. Then there exists a sequence $\langle b_{n} \rangle$ where $b_{n} \in S$ for each $n$ and:

$$
\lim_{n \to \infty} b_{n} = \inf(S).
$$

##### Proof of Supremum Approximation

1\. Suppose $S \subseteq \mathbb{R}$ is bounded above. By the **Least Upper Bound Property**, $\sup(S)$ exists. Denote it by $\alpha = \sup(S).$

**Supremum Definition**
2\. By definition of the supremum:
- $a \leq \alpha$ for all $a \in S$;
- For any $\varepsilon > 0,$ there exists some $x \in S$ such that $x > \alpha - \varepsilon.$

3\. **Sequence Construction:**
For each $n \in \mathbb{N},$ set $\varepsilon = \frac{1}{n}.$ By a bounded set's supremum property, there exists an $a_{n} \in S$ such that:

$$
a_{n} > \alpha - \frac{1}{n}.
$$

Thus, we have constructed a sequence $(a_{n})$ with the property:

$$
\alpha - \frac{1}{n} < a_{n} \leq \alpha \quad \text{for all } n \in \mathbb{N}.
$$

4\. **Sequence Convergence:** Let $\langle x_{n} \rangle = \{\alpha\}$ and $\{y_{n}\} = \{\frac{1}{n}\}.$ Then, as $n \to \infty$:

$$
x_{n} \to \alpha \quad \text{and} \quad y_{n} \to 0.
$$

5\. **Limit of $\alpha - \frac{1}{n}$:** By the **difference rule for limits of sequences**, as $n \to \infty$:

$$
\alpha - \frac{1}{n} \to \alpha - 0 = \alpha.
$$

6\. **Inequality Relationship:** From step (3), we have:

$$
\alpha - \frac{1}{n} \leq a_{n} \leq \alpha \quad \text{for all } n \in \mathbb{N}.
$$

As $n \to \infty,$ both the lower bound ($\alpha - \frac{1}{n}$) and the upper bound ($\alpha$) converge to $\alpha.$

7\. **Application of the Squeeze Theorem:** By the **Squeeze Theorem**, it follows that:

$$
\lim_{n \to \infty} a_{n} = \alpha.
$$

Therefore, if $S \subseteq \mathbb{R}$ is bounded above, there exists a sequence $(a_{n}) \subseteq S$ such that:

$$
\lim_{n \to \infty} a_{n} = \sup(S).
$$

##### Proof of Infimum Approximation

1\. Suppose $S \subseteq \mathbb{R}$ is bounded below. By the **Greatest Lower Bound Property**, $\inf(S)$ exists. Denote it by $\beta = \inf(S).$

2\. **Infimum Definition:** By definition of the infimum:
- $\beta \leq a$ for all $a \in S$;
- For any $\varepsilon > 0,$ there exists some $x \in S$ such that $x < \beta + \varepsilon.$

3\. **Sequence Construction:** For each $n \in \mathbb{N},$ set $\varepsilon = \frac{1}{n}.$ By a bounded set's infimum property, there exists an $a_{n} \in S$ such that:

$$
a_{n} < \beta + \frac{1}{n}.
$$

Thus, we have constructed a sequence $(a_{n})$ with the property:

$$
\beta + \frac{1}{n} > a_{n} \geq \beta, \quad \text{for all } n \in \mathbb{N}.
$$

4\. **Sequence Convergence:** Let $\langle x_{n} \rangle = \{\beta\}$ and $\{y_{n}\} = \{\frac{1}{n}\}.$ Then, as $n \to \infty$:

$$
x_{n} \to \beta \quad \text{and} \quad y_{n} \to 0.
$$

5\. **Limit of $\beta + \frac{1}{n}$:** By the **sum rule for limits of sequences**, as $n \to \infty$:

$$
\beta + \frac{1}{n} \to \beta + 0 = \beta.
$$

6\. **Inequality Relationship:** From step (3), we have:

$$
\beta + \frac{1}{n} > a_{n} \geq \beta, \quad \text{for all } n \in \mathbb{N}.
$$

As $n \to \infty,$ both the upper bound ($\beta + \frac{1}{n}$) and the lower bound ($\beta$) converge to $\beta.$

7\. **Application of the Squeeze Theorem:** By the **Squeeze Theorem**, it follows that:

$$
\lim_{n \to \infty} a_{n} = \beta.
$$

Therefore, if $S \subseteq \mathbb{R}$ is bounded below, there exists a sequence $(a_{n}) \subseteq S$ such that:

$$
\lim_{n \to \infty} a_{n} = \inf(S).
$$

## 3.7 Subsequences

1\. What is a subsequence, and how is it defined formally?

2\. What does Proposition 3.32 state about the relationship between the convergence of a sequence and its subsequences?

3\. How can different subsequences of a sequence like $a_{n} = (-1)^{n}$ illustrate divergence?

### Key Terms

#### Limits of Sequences and Subsequences Are Equal (Proposition 3.32)

A sequence $\langle x_{n} \rangle$ converges to $L$ if and only if every subsequence of $\langle x_{n} \rangle$ also converges to $L.$

##### Proof

###### Case 1: $\langle x_{n} \rangle \to L \implies \text{every } \langle x_{n_{k}} \rangle \to L$

1\. Assume that a sequence $\langle x_{n} \rangle$ converges to $L$ and $\langle x_{n_{k}} \rangle$ is a subsequence of $\langle x_{n} \rangle.$

2\. Let $\varepsilon > 0.$

3\. By definition of convergence:

$$
\exists N \in \mathbb{N} \bigl[\forall n \in N( n > N \implies |x_{n} - L| < \varepsilon) \bigr].
$$

4\. For the subsequence $\langle x_{n_{k}} \rangle$ to converge to $L$:

$$
\exists N_{1} \in \mathbb{N} \bigl[\forall k \in N( k > N \implies |x_{{n}_{k}} - L| < \varepsilon) \bigr].
$$

5\. By definition of subsequences, $\langle n_{k} \rangle$ is a strictly increasing sequence of natural numbers:

$$
n_{1} < n_{2} < n_{3} < \dots
$$

6\. Thus, for every $n_{k},$ it follows that:

$$
n_{k} \geq k.
$$

7\. If $N_{1} = N$ and $k > N_{1},$ then $n_{k} > N,$ ensuring:

$$
|x_{n_{k}} - L| < \varepsilon.
$$

$$
\therefore \boxed{\langle x_{n_{k}} \rangle \to L}
$$

###### Case 2: $\text{If All } \langle x_{n_{k}} \rangle \to L \implies \langle x_{n} \rangle \to L$

**Approach 1: Squeeze Theorem**

1\. Given: Every subsequence $\langle x_{n_{k}} \rangle$ of the sequence $\langle x_{n} \rangle$ satisfies:

$$
\lim_{k \to \infty} x_{n_{k}} = L.
$$

2\. **Bounding the Sequence**: Define two specific subsequences of $\langle x_{n} \rangle$:
- Let $\langle a_{k} \rangle$ be the subsequence of $\langle x_{n} \rangle$ consisting of its largest terms (maximum subsequence).
- Let $\langle b_{k} \rangle$ be the subsequence of $\langle x_{n} \rangle$ consisting of its smallest terms (minimum subsequence).

3\. By assumption, both $\langle a_{k} \rangle$ and $\langle b_{k} \rangle$ converge to $L:$

$$
\lim_{k \to \infty} a_{k} = L \quad \text{and} \quad \lim_{k \to \infty} b_{k} = L
$$

4\. **Squeezing the Sequence**: Since the original sequence $\langle x_{n} \rangle$ lies between its minimum and maximum subsequences:

$$
b_{k} \leq x_{n} \leq a_{k}, \quad \text{for all } n.
$$

5\. Taking the limit as $n \to \infty,$ we apply the **Squeeze Theorem**:

$$
\lim_{n \to \infty} b_{k} = L, \quad \lim_{n \to \infty} a_{k} = L \implies \lim_{n \to \infty} x_{n} = L
$$

$$
\therefore \boxed{\text{If all } \langle x_{n_{k}} \rangle \to L \implies \langle x_{n} \rangle \to L}
$$

**Approach 2: Proof by Contradiction**

1\. Given: Every subsequence $\langle x_{n_{k}} \rangle$ of the sequence $\langle x_{n} \rangle$ satisfies:

$$
\lim_{k \to \infty} x_{n_{k}} = L
$$

2\. Assume, for the sake of contradiction, that $\langle x_{n} \rangle$ does **not** converge to $L.$

3\. By definition of divergence, there exists $\varepsilon > 0$ such that:

$$
\forall N \in \mathbb{N} \bigl[ \exists n \in N (n > N ~ \land ~ |x_{n} - L| \geq \varepsilon)\bigr].
$$

4\. **Construction of a Divergent Subsequence**: Using the negation of convergence, we can construct a subsequence $\langle x_{n_{k}} \rangle$ of $\langle x_{n} \rangle$ such that:

$$
|x_{n_{k}} - L| \geq \varepsilon, \quad \text{for all } k.
$$

5\. Because $\langle x_{n_{k}} \rangle$ diverges, it does **not** converge to $L.$ This contradicts the assumption that every subsequence of $\langle x_{n} \rangle$ converges to $L.$

$$
\therefore \boxed{\text{If all } \langle x_{n_{k}} \rangle \to L \implies \langle x_{n} \rangle \to L}
$$

**Summary:**
- **Case 1**: If $\langle x_{n} \rangle \to L,$ then all subsequences $\langle x_{n_{k}} \rangle$ converge to $L.$
- **Case 2**: If all subsequences $\langle x_{n_{k}} \rangle$ converge to $L,$ then $\langle x_{n} \rangle$ also converges to $L.$

$$
\boxed{\langle x_{n} \rangle \to L \iff \text{Every subsequence } \langle x_{n_{k}} \rangle \to L}
$$

#### Different Subsequential Limits Implies Sequence Divergence (Page 96, 3.34)

If a sequence $\langle x_{n} \rangle$ has two or more subsequences converging to different limits, then $\langle x_{n} \rangle$ is divergent.

##### Proof

1\. Given: Two or more subsequences of a sequence $\langle x_{n} \rangle$ such that they converge at different limits.

2\. Assume, for a contradiction, that $\langle x_{n} \rangle$ converges to $L.$

3\. Because the limits of sequences and subsequences are equal, then any subsequence of $\langle x_{n} \rangle$ would also converge at $L.$

4\. However, this contradicts the assumption that there are subsequences converging to different limits.

$$
\therefore ~ \boxed{\langle x_{n} \rangle \text{ diverges}}
$$

#### Monotone Sequence with Convergent Subsequence Converges to the Same Limit (Page 97, 3.35)

If a monotone sequence $\langle x_{n} \rangle$ has a convergent subsequence, then $\langle x_{n} \rangle$ converges too, and has the same limit.

##### Proof

1\. Assume that a sequence $\langle x_{n} \rangle$ is monotone increasing.

2\. Let $\langle x_{n_{k}} \rangle$ be a convergent subsequence of $\langle x_{n} \rangle.$

3\. By definition of **subsequences**, the terms in $\langle x_{n_{k}} \rangle$ and their order originate in $\langle x_{n} \rangle.$

4\. Therefore, $\langle x_{n_{k}} \rangle$ is also monotone increasing.

5\. By the **monotone convergence theorem**, if $\langle x_{n_{k}} \rangle$ is monotone increasing and convergent:

$$
\lim_{ n \to \infty } x_{n_{k}} = \sup(\{ x_{n_{k}} \mid k \in \mathbb{N} \})
$$

6\. Because $\langle x_{n_{k}} \rangle$ derives its terms from $\langle x_{n} \rangle$ and $\langle x_{n} \rangle$ is monotone increasing, for any element $x_{n},$ there exists an element $x_{n_{k}}$ such that for some $k,$

$$
x_{n} \leq x_{n_{k}}
$$

7\. Therefore, for all $n,$

$$
x_{n} \leq x_{n_{k}} \leq \sup(\{ x_{n_{k}} \mid k \in \mathbb{N} \})
$$

8\. This shows that $\langle x_{n} \rangle$ is bounded above.

9\. By the **monotone convergence theorem**, because $\langle x_{n} \rangle$ is bounded above, it converges.

10\. By of the **equality of limits of sequences and subsequences**, if $\langle x_{n} \rangle$ is convergent it has the same limit as $\langle x_{n_{k}} \rangle$:

$$
\lim_{ n \to \infty } x_{n} = \lim_{ n \to \infty } x_{n_{k}} = \sup(\{ x_{n_{k}} \mid k \in \mathbb{N} \})
$$

##### Proof II

Without loss of generality, assume that $(a_{n})$ is monotone increasing.

###### Technical Property

- Since $(a_{n})$ is increasing, it follows that if $s < t,$ then $a_{s} \leq a_{t}.$
- For any subsequence $\langle x_{n_{k}} \rangle,$ the indices $n_{k}$ are strictly increasing, and therefore $t \leq n_{t}$ for all $t.$
- Thus, combining these properties, if $s < t$:

$$
a_{s} \leq a_{t} \leq a_{n_{t}}. \tag{1}
$$

###### Main Argument

1\. Suppose that $\langle x_{n_{k}} \rangle$ is a **convergent subsequence** of $(a_{n}).$

2\. By the **Monotone Convergence Theorem** and the fact that $\langle x_{n_{k}} \rangle$ converges, we know that:

$$
\lim_{k \to \infty} a_{n_{k}} = \sup(\{a_{n_{k}}: k \in \mathbb{N}\}).
$$

3\. Let $a = \sup(\{a_{n_{k}}: k \in \mathbb{N}\}).$

4\. By definition of sequence **convergence**, there exists some $M \in \mathbb{N}$ such that:

$$
a - \varepsilon < a_{n_{k}} < a + \varepsilon, \quad \text{for all } k > M.
$$

5\. In particular, this holds when $k = M + 1.$ Thus:

$$
a - \varepsilon < a_{n_{M+1}}.
$$

6\. Since $a$ is the supremum and, hence, an upper bound for $\langle x_{n_{k}} \rangle,$ we also have:

$$
a - \varepsilon < a_{n_{M+1}} \leq a. \tag{2}
$$

###### Bounding $\{a_ \ell\}$

7\. Combining the inequalities from $(1)$ and $(2),$ we observe that for all $\ell > n_{M+1}$:

$$
a - \varepsilon < a_{n_{M+1}} \leq a_{\ell} \leq a_{n_{\ell}} \leq a.
$$

8\. Thus, there exists some $N \in \mathbb{N}$ (in particular, $N = n_{M+1}$) such that for all $\ell > N$:

$$
a - \varepsilon < a_{\ell} \leq a.
$$

(Alternatively, because $a_{\ell}$ is bounded above by $a,$ by the **monotone convergence theorem**, $a_{\ell}$ converges to $a.$)

$$
\therefore ~ \boxed{a_ \ell \to a}
$$

## 3.8 The Bolzano-Weierstrass Theorem

1\. What does the Bolzano-Weierstrass theorem state about bounded sequences?

2\. How can you prove that every bounded sequence has a convergent subsequence?

3\. What are examples of sequences that do or do not have convergent subsequences?

### Key Terms

#### Peak Point Lemma (Page 99, 3.36)

[ProofWiki](https://proofwiki.org/wiki/Category:Peak_{P}oint_{L}emma)

Let $\langle x_{n} \rangle$ be a sequence in $\mathbb{R}.$ Then there exists a subsequence $\langle x_{n_{k}} \rangle$ such that $\langle x_{n_{k}} \rangle$ is monotone. Formally,

For a real sequence $\langle x_{n} \rangle,$ the set of peak-point indices is defined as:

$$
S =\left\{\begin{gather}
n \in \mathbb{N} : \forall m > n, \\
(x_{n} \geq x_{m} \implies x_{n} \text{ is a peak})
\end{gather}\right\}
$$

1. If $S$ is infinite, then $\langle x_{n_{k}} \rangle$ is monotone non-increasing.
2. If $S$ is finite, then $\langle x_{n_{k}} \rangle$ is monotone non-decreasing.

##### Proof

Let $\langle x_{n} \rangle$ be a sequence in $\mathbb{R}.$ Define a term, $x_{n},$ as a *peak* if:

$$
x_{n} \geq x_{m} \quad \forall m > n.
$$

That is, a term is a *peak* if it is larger than every subsequent term.

We consider two cases based on the number of peak points:

1. **Case 1: Infinitely Many Peaks**
   - Assume $\{n \in \mathbb{N}: x_{n} \text{ is a peak}\}$ is infinite.
   - Let $\langle x_{n_{k}} \rangle$ denote the subsequence of all peak points, where $x_{n_{k}}$ is the $k$-th peak.
   - By definition of a peak:

$$
x_{n_{k}} \geq x_{n_{k+1}} \quad \forall k \in \mathbb{N}.
$$

   - Thus, $\langle x_{n_{k}} \rangle$ is a monotone **non-increasing** subsequence.

1. **Case 2: Finitely Many Peaks**
   - Assume $\{n \in \mathbb{N}: x_{n} \text{ is a peak}\}$ is finite.
   - Let $x_{N}$ be the last peak, such that for $n > N,$ no term is a peak.
   - Hence, for every $n > N,$ there exists $m > n$ such that:

$$
x_{m} > x_{n}.
$$

   - Construct a subsequence $\langle x_{n_{k}} \rangle$ inductively:
     - Let $n_{1} > N$ such that $x_{n_{1}} > x_{N}.$
     - For $k \geq 1,$ let $n_{k+1} > n_{k}$ such that $x_{n_{k+1}} > x_{n_{k}}.$
   - Thus, $\langle x_{n_{k}} \rangle$ is a monotone **non-decreasing** subsequence.

In either case (infinitely many peaks or finitely many peaks), we construct a monotone subsequence (either increasing or decreasing).
Thus, every sequence in $\mathbb{R}$ has a monotone subsequence.

$$
\therefore ~ \boxed{\forall \langle x_{n} \rangle \subseteq \mathbb{R}, \exists \langle x_{n_{k}} \rangle \subseteq \langle x_{n} \rangle : \langle x_{n_{k}} \rangle \text{ is monotone.}}
$$

#### Bolzano-Weierstrass Theorem (Page 101, 3.37)

Every bounded sequence has a convergent subsequence.

##### Proof

1\. Assume $\langle x_{n} \rangle$ is a bounded sequence.

2\. Because $\langle x_{n} \rangle$ is bounded, $\langle x_{n_{k}} \rangle$ is also bounded.

3\. By the peak point lemma, for every sequence, there exists a monotone subsequence $\langle x_{n_{k}} \rangle.$

4\. By the **monotone convergence theorem**, since $\langle x_{n_{k}} \rangle$ is both bounded and monotone, it is also convergent.

## The Cauchy Criterion

### Key Terms

#### Cauchy Criterion (Page 104, 3.39)

A real sequence $(a_{n})$ that satisfies the property that for every $\varepsilon > 0$ there exists some $N \in \mathbb{N}$ such that for all $m, n > N,$

$$
|a_{m} - a_{n}| < \varepsilon.
$$

#### Lemma: Cauchy Sequences Are Bounded (Page 106, 3.41)

If a sequence $\langle x_{n} \rangle$ satisfies the Cauchy criterion, then $\langle x_{n} \rangle$ is bounded.

##### Proof

1\. **Definition of a Cauchy Sequence**: Since $\langle x_{n} \rangle$ is Cauchy, for $\varepsilon > 0,$ there exists some $N \in \mathbb{N}$ such that:

$$
|x_{m} - x_{n}| < \varepsilon, \quad \text{for all } m, n > N.
$$

2\. **Special Case for Adjacent Terms**: In particular, for all $n > N,$ we can set $m = n + \varepsilon,$ giving:

$$
|x_{n} - x_{n+1}| < \varepsilon.
$$

3\. **Bounding Terms Beyond $N$**: Since $|x_{n} - x_{n+1}| < \varepsilon$ for $n > N,$ we know the sequence terms beyond $N$ are bounded as:

$$
x_{N+1} - \varepsilon < x_{n} < x_{N+1} + \varepsilon, \quad \text{for all } n > N.
$$

4\. **Bounding the Entire Sequence**: The sequence terms $\langle x_{n} \rangle$ for $n \leq N$ are finitely many, so they are bounded as well.

5\. **General Bounds for $\langle x_{n} \rangle$**: Define the lower bound $L$ and the upper bound $U$ for the entire sequence as:

$$
\begin{align}
L &= \min\{|x_{1}|, |x_{2}|, \dots, |x_{N}|, |x_{N+1} - \varepsilon|\}, \\
U &= \max\{|x_{1}|, |x_{2}|, \dots, |x_{N}|, |x_{N+1} + \varepsilon|\}.
\end{align}
$$

6\. For all $n \in \mathbb{N},$ we have:

$$
L \leq x_{n} \leq U.
$$

$$
\therefore ~ \boxed{\text{The Cauchy sequence $\langle x_{n} \rangle$ is bounded.}}
$$

#### Cauchy Criterion for Convergence (Page 107, 3.42)

A sequence $\langle x_{n} \rangle$ in $\mathbb{R}$ converges to some $L \in \mathbb{R}$ if and only if it satisfies the Cauchy Criterion.

##### Proof

###### Case 1: $x_{n} \to L \implies \langle x_{n} \rangle \text{ is Cauchy}$

1\. **Assume Convergence**: Suppose $\langle x_{n} \rangle$ converges to $L \in \mathbb{R}.$ Let $\varepsilon > 0.$

2\. By definition of sequence **convergence**, there exists $N \in \mathbb{N}$ such that for all $n > N$:

$$
|x_{n} - L| < \frac{\varepsilon}{2}. \tag{1}
$$

3\. For any $n, m > N,$ use the triangle inequality:

$$
\begin{align}
|x_{n} - x_{m}| &= |x_{n} - L + L - x_{m}|  \\
&\leq |x_{n} - L| + |x_{m} - L|.
\end{align}
$$

4\. Substituting the bounds from $(1)$:

$$
|x_{n} - x_{m}| \leq \frac{\varepsilon}{2} + \frac{\varepsilon}{2} = \varepsilon.
$$

5\. Thus, $\langle x_{n} \rangle$ satisfies the Cauchy condition.

###### Case 2: $\langle x_{n} \rangle \text{ is Cauchy} \implies x_{n} \to L$

1\. **Assume Cauchy**: Suppose $\langle x_{n} \rangle$ satisfies the Cauchy Criterion.

2\. By the lemma that Cauchy sequences are bounded, $\langle x_{n} \rangle$ is bounded.

3\. By the **Bolzano-Weierstrass Theorem**, $\langle x_{n} \rangle$ has a convergent subsequence, $(x_{n_{j}}),$ which converges to some $L \in \mathbb{R}.$

4\. Let $\varepsilon > 0.$ Since $\langle x_{n} \rangle$ is Cauchy, there exists $N_{1} \in \mathbb{N}$ such that for all $n, m > N_{1}$:

$$
|x_{n} - x_{m}| < \frac{\varepsilon}{2}. \tag{1}
$$

5\. Since $(x_{n_{j}})$ converges to $L,$ there exists $N_{2} \in \mathbb{N}$ such that for all $j > N_{2}$:

$$
|x_{n_{j}} - L| < \frac{\varepsilon}{2}. \tag{2}
$$

6\. Define $J = \max\{N_{1}, N_{2}\}.$ For $j > J,$ both conditions $(1)$ and $(2)$ hold.

7\. *Subsequence Approximation*: By definition of subsequences, $n_{j} \geq j.$ For $j > J$:

$$
\begin{align}
|x_{j} - L| &= |x_{j} - x_{n_{j + 1}} + x_{n_{j + 1}} - L|  \\
&\leq |x_{j} - x_{n_{j}}| + |x_{n_{j}} - L|.
\end{align}
$$

8\. Substituting $(1)$ and $(2)$:

$$
|x_{j} - L| \leq \frac{\varepsilon}{2} + \frac{\varepsilon}{2} = \varepsilon.
$$

10\. Since $|x_{j} - L| < \varepsilon$ for all $j > J,$ $\langle x_{n} \rangle$ converges to $L.$

###### Conclusion

- $x_{n} \to L \implies \langle x_{n} \rangle \text{ is Cauchy}$
- $\langle x_{n} \rangle \text{ is Cauchy} \implies x_{n} \to L$

$$
\therefore ~ \boxed{\langle x_{n} \rangle \text{ is Cauchy} \iff x_{n} \to L}
$$

#### Cauchy Sequence Examples

##### Example 1: Show that the Sequence $\left(\frac{1}{n} \right)$ is a Cauchy Sequence

###### Scratch Work

1\. **Goal:** Given an arbitrary $\varepsilon > 0,$ find a specific $N \in \mathbb{N}$ such that for all $n, m \geq N,$

$$
|x_{n} - x_{m}| < \varepsilon.
$$

2\. **Find $N$**:

$$
\begin{align}
|x_{n} - x_{m}| &< \varepsilon \\\
\left| \frac{1}{n} - \frac{1}{m} \right| &< \varepsilon \\\
\left| \frac{1}{n} - \frac{1}{m} \right| &= \left| \frac{1}{n} + \frac{-1}{m} \right| \\\
&\leq \left| \frac{1}{n} \right| + \left| \frac{-1}{m} \right| \\\
&= \frac{1}{n} + \frac{1}{m}.
\end{align}
$$

3\. If we want $\frac{1}{n} + \frac{1}{m} < \varepsilon,$ it suffices to make each term less than $\frac{\varepsilon}{2}.$ By the **Archimedean principle**, we can choose $N$ such that $\frac{1}{N} < \frac{\varepsilon}{2},$ and for $n, m \geq N,$ we have $\frac{1}{n} < \frac{\varepsilon}{2}$ and $\frac{1}{m} < \frac{\varepsilon}{2}.$

###### Solution

1\. Fix any $\varepsilon > 0$ and set $N \in \mathbb{N}$ such that $\frac{1}{N} < \frac{\varepsilon}{2}.$
2\. Then for all $n, m \geq N$:

$$
\begin{align}
|x \_{n} - x \_{m}| &= \left| \frac{1}{n} - \frac{1}{m} \right| \\\
& \leq \frac{1}{n} + \frac{1}{m} \\\
&< \frac{\varepsilon}{2} + \frac{\varepsilon}{2} = \varepsilon.
\end{align}
$$

$$
\therefore ~ \boxed{\left( \frac{1}{n} \right) \text{ is a Cauchy Sequence.}}
$$

---

##### Example 2: Show that the Sequence $\left((-1)^{n} \right)$ is not Cauchy

###### Scratch Work

1\. **Goal:** Given an arbitrary $\varepsilon > 0,$ find a specific $N \in \mathbb{N}$ such that for all $n, m \geq N,$

$$
|x \_{n} - x \_{m}| < \varepsilon.
$$

2\. **Observation:** For $x \_{n} = (-1)^{n}$:
- If $n$ is even, $x \_{n} = 1.$
- If $n$ is odd, $x \_{n} = -1.$
- Alternating terms cause $|x \_{n} - x \_{m}| = 2,$ which does not decrease as $n, m$ increase.

3\. To prove $\left((-1)^{n} \right)$ is not Cauchy, choose $\varepsilon = 1.$ Observe that $|x_{n} - x_{m}| \geq 2$ regardless of $N.$

###### Solution

1\. Let $\varepsilon = 1.$ For any $N \in \mathbb{N},$ pick $n = 2k$ (even) and $m = 2k + 1$ (odd), where $k \geq N.$

2\. Then:

$$
x \_{n} = 1, \quad x \_{m} = -1, \quad |x \_{n} - x \_{m}| = |1 - (-1)| = 2.
$$

3\. Since $|x_{n} - x_{m}| \geq \varepsilon = 1,$ $\left((-1)^{n} \right)$ is not a Cauchy sequence.

$$
\therefore ~ \boxed{(-1)^{n} \text{ is not a Cauchy Sequence.}}
$$

---

##### Example 3: Show that the Sequence $\left(\frac{1}{n^{2}} \right)$ is Cauchy

###### Scratch Work

1\. **Goal:** Given an arbitrary $\varepsilon > 0,$ find a specific $N \in \mathbb{N}$ such that for all $n, m \geq N,$

$$
|x_{n} - x_{m}| < \varepsilon.
$$

2\. **Find $N$**:

$$
\begin{align}
|x_{n} - x_{m}| &< \varepsilon \\
\left| \frac{1}{n^{2}} - \frac{1}{m^{2}} \right| &< \varepsilon \\
\left| \frac{1}{n^{2}} - \frac{1}{m^{2}} \right| &= \left| \frac{1}{n^{2}} + \frac{-1}{m^{2}} \right| \\
&\leq \left| \frac{1}{n^{2}} \right| + \left| \frac{-1}{m^{2}} \right| \\
&= \frac{1}{n^{2}} + \frac{1}{m^{2}}.
\end{align}
$$

1. If we want $\frac{1}{n^{2}} + \frac{1}{m^{2}} < \varepsilon,$ it suffices to make each term less than $\frac{\varepsilon}{2}.$ By the **Archimedean principle**, we can choose $N$ such that $\frac{1}{N^{2}} < \frac{\varepsilon}{2},$ and for $n, m \geq N,$ we have $\frac{1}{n^{2}} < \frac{\varepsilon}{2}$ and $\frac{1}{m^{2}} < \frac{\varepsilon}{2}.$

###### Solution

1\. Fix any $\varepsilon > 0$ and set $N \in \mathbb{N}$ such that $\frac{1}{N^{2}} < \frac{\varepsilon}{2}.$

2\. Then for all $n, m \geq N$:

$$
\begin{align}
|x_{n} - x_{m}| &= \left| \frac{1}{n^{2}} - \frac{1}{m^{2}} \right| \\
& \leq \frac{1}{n^{2}} + \frac{1}{m^{2}} \\
&< \frac{\varepsilon}{2} + \frac{\varepsilon}{2} = \varepsilon.
\end{align}
$$

$$
\therefore \boxed{\left( \frac{1}{n^{2}} \right) \text{ is a Cauchy Sequence.}}
$$

---

### Extra Proofs and Theorems

#### Shift Rule for Convergent Sequences

Given a real sequence, $\langle x_{n} \rangle \in \mathbb{R},$ for any $m, n \in \mathbb{N},$

$$
\lim_{n \to \infty} x_{n} = L \iff \lim_{n \to \infty} x_{n + m} = L.
$$

---

##### Proof

###### Case 1: $\lim\limits_{ n\to \infty }x_{n} = L \implies \lim\limits_{ n\to \infty}x_{n + m} = L$

1\. Suppose $\langle x_{n} \rangle$ is a real, convergent sequence, such that:

$$
\lim_{ n \to \infty } x_{n} = L
$$

2\. By the **definition of convergence**, for any $\varepsilon > 0,$ there exists an $N \in \mathbb{N},$ such that for all $n \in \mathbb{N}:$

$$
n \geq N \implies |x_{n} - L| < \varepsilon
$$

3\. Given any $m \in \mathbb{N},$ consider the **shifted sequence** $(x_{n + m})$ and observe since $n \geq N$ implies $m + n \geq N,$ the same bound holds:

$$
|x_{n + m} - L| < \varepsilon.
$$

4\. Thus, by the **definition of convergence**,

$$
\lim_{ n \to \infty }x_{n + m} = L.
$$

$$
\therefore ~ \boxed{\lim_{ n \to \infty }x_{n} = L \implies \lim_{ n \to \infty }x_{n + m} = L}
$$

---

###### Case 2: $\lim\limits_{ n\to \infty }x_{n + m} = L \implies \lim\limits_{ n\to \infty }x_{n} = L$

1\. Suppose $(x_{n + m})$ is a real, shifted, convergent sequence, such that $\forall m \in \mathbb{N}:$

$$
\lim_{ n \to \infty } x_{n + m} = L
$$

2\. By definition of sequence **convergence**, $x_{n + m} \to L$ implies $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies |x_{n + m} - L| < \varepsilon
$$

3\. Define $k = n + m$ and observe that $n = k - m.$

4\. Substitute $k - m$ for $n$ in the inequality:

$$
k - m \geq N \implies |x_{k} - L| < \varepsilon
$$

5\. Then, shifting the sequence back to its unshifted form:

$$
\begin{align}
&n = k \geq N + m > N  \\
& \quad \implies |x_{n} - L| < \varepsilon
\end{align}
$$

6\. Thus, by the **definition of convergence**, $x_{n} \to L,$ since $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies |x_{n} - L| < \varepsilon
$$

$$
\therefore ~ \boxed{\lim_{ n \to \infty }x_{n + m} = L \implies \lim_{ n \to \infty }x_{n} = L}
$$

##### Alternate Proof: [Tail of Convergent Sequence - ProofWiki](https://proofwiki.org/wiki/Tail_of_Convergent_Sequence)

###### **Necessary Condition (Forward Direction)**

**Assumption:** Suppose $x_{n} \to L$.

1\. By the definition of convergence, for every $\varepsilon > 0$, there exists $N \in \mathbb{N}$, for all $n \in \mathbb{N},$ such that:

$$
n \geq N \implies |x_{n} - L| < \varepsilon
$$

2\. Define $N^{*} = \max(1, N - m),$ then for all $n \geq N^{*},$

$$
n \geq N - m \implies n + m \geq N
$$

3\. Since $n + m \geq N$, we apply the assumption:

$$
|x_{n+m} - L| < \varepsilon, \quad \forall n \geq N^{*}
$$

4\. Thus, $x_{n+m} \to L$, proving the necessary condition.

###### **Sufficient Condition (Reverse Direction)**

**Assumption:** Suppose $x_{n+m} \to L$.

1\. By the definition of convergence, for every $\varepsilon > 0$, there exists $N \in \mathbb{N}$, for all $n \in \mathbb{N},$ such that:

$$
n \geq N \implies |x_{n + m} - L| < \varepsilon
$$

2\. Since $n + 1 \geq N$ implies $n \geq N - 1$, define:

$$
N^* = N + 1.
$$

This ensures that for all $n \geq N^*$, we still have:

$$
|x_{n} - L| < \varepsilon
$$

3\. Thus, $x_{n} \to L$, proving the sufficient condition.

---

###### **Conclusion**

Since both the necessary and sufficient conditions hold, we conclude:

$$
a_n \to a \quad \text{if and only if} \quad a_{n+m} \to a.
$$

$$
\boxed{\text{Shifting a sequence by a finite number of terms does not affect its limit.}}
$$

---

#### Shift Rule for Divergent Sequences

Given a real sequence, $\langle x_{n} \rangle \in \mathbb{R},$ for any $m, n \in \mathbb{N},$

$$
\lim_{n \to \infty} x_{n} = \infty \iff \lim_{n \to \infty} x_{n + m} = \infty.
$$

##### Proof

###### Case 1: $\lim\limits_{n \to \infty }x_{n} = \infty \implies \lim\limits_{n \to \infty}x_{n + m} = \infty$

1\. Suppose $\langle x_{n} \rangle$ is a real, divergent sequence, such that:

$$
\lim_{ n \to \infty }  x_{n} = \infty
$$

2\. By definition of **divergence to infinity**, $\forall M > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies x_{n} > M
$$

3\. Given any $m \in \mathbb{N}$, consider the **shifted sequence** $(x_{n + m})$. Since $n \geq N$ implies $m + n \geq N$, the same bound applies:

$$
x_{n + m} > M
$$

4\. Thus, by **definition of divergence**, $x_{n + m} \to \infty$, since $\forall M > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies x_{n + m} > M
$$

$$
\therefore ~ \boxed{\lim\limits_{ n \to \infty }x_{n} = \infty \implies \lim\limits_{ n \to \infty }x_{n + m} = \infty}
$$

---

###### Case 2: $\lim\limits_{n\to \infty }x_{n + m} = \infty \implies \lim\limits_{n \to \infty }x_{n} = \infty$

1\. Suppose $(x_{n + m})$ is a real, shifted sequence divergent to infinity, such that $\forall m \in \mathbb{N}$:

$$
\lim_{ n \to \infty } x_{n + m} = \infty
$$

2\. By definition of sequence **divergence**, $x_{n + m} \to \infty$ implies $\forall M > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies x_{n + m} > M
$$

3\. Define $k = n + m$ and observe that $n = k - m.$

4\. Substitute $k - m$ for $n$ in the inequality:

$$
k - m \geq N \implies x_{k} > M
$$

5\. Then, shifting the sequence back to its unshifted form:

$$
\begin{align}
&n = k \geq N + m > N  \\
& \quad \implies x_{n} > M
\end{align}
$$

6\. Thus, by the definition of sequence **divergence**, $x_{n} \to \infty$, since $\forall M > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies x_{n} > M
$$

$$
\therefore ~ \boxed{\lim_{ n \to \infty }x_{n + m} = \infty \implies \lim_{ n \to \infty }x_{n} = \infty}
$$

##### Conclusion

- $\lim\limits_{ n \to \infty }x_{n} = \infty \implies \lim\limits_{ n \to \infty }x_{n + m} = \infty.$
- $\lim\limits_{ n \to \infty }x_{n + m} = \infty \implies \lim\limits_{ n \to \infty }x_{n} = \infty.$

$$
\therefore ~ \boxed{\lim_{ n \to \infty }x_{n} = \infty \iff \lim_{ n \to \infty }x_{n + m} = \infty}
$$

---

#### Proof: Equivalence of the Root Test and Ratio Test Limits ([Math Stack Exchange](https://math.stackexchange.com/questions/287932/convergence-of-ratio-test-implies-convergence-of-the-root-test?rq=1))

We want to show that if the sequence $(a_n)$ consists of positive terms and the limit

$$
\lim_{n \to \infty} \frac{a_{n+1}}{a_n} = L
$$

exists, then it follows that:

$$
\lim_{n \to \infty} \sqrt[n]{a_n} = L.
$$

##### Step 1: Using the Definition of Limit

By the definition of a limit, for each $\varepsilon > 0$, there exists $N$ such that for all $n > N$:

$$
\left| \frac{a_{n+1}}{a_n} - L \right| < \varepsilon.
$$

Thus, we can bound $\frac{a_{n+1}}{a_n}$ as follows:

$$
L - \varepsilon < \frac{a_{n+1}}{a_n} < L + \varepsilon.
$$

##### Step 2: Expressing $a_n$ as a Product

We express $a_n$ in terms of previous terms:

$$
a_n = \frac{a_n}{a_{n-1}} \cdot \frac{a_{n-1}}{a_{n-2}} \cdots \frac{a_{N+1}}{a_N} \cdot a_N.
$$

Applying the given bound on $\frac{a_{n+1}}{a_n}$, we obtain:

$$
a_n < (L + \varepsilon)^{n-N} a_N.
$$

##### Step 3: Taking the $n$th Root

Taking the $n$th root on both sides:

$$
\sqrt[n]{a_n} < (L + \varepsilon)^{(n-N)/n} \cdot \sqrt[n]{a_N}.
$$

##### Step 4: Taking the Limit

As $n \to \infty$, we observe that:

$$
\lim_{n \to \infty} (L + \varepsilon)^{(n-N)/n} = L + \varepsilon.
$$

Thus, taking limits on both sides:

$$
\lim_{n \to \infty} \sqrt[n]{a_n} \leq L + \varepsilon.
$$

Since $\varepsilon$ is arbitrary, we conclude:

$$
\lim_{n \to \infty} \sqrt[n]{a_n} \leq L.
$$

Similarly, using a lower bound with $L - \varepsilon$, we obtain:

$$
\lim_{n \to \infty} \sqrt[n]{a_n} \geq L.
$$

##### Conclusion

Since we have both upper and lower bounds, we conclude:

$$
\lim_{n \to \infty} \sqrt[n]{a_n} = L.
$$

Thus, we have proven the equivalence:

$$
\lim_{n \to \infty} \sqrt[n]{a_n} = \lim_{n \to \infty} \frac{a_{n+1}}{a_n}.
$$

---

### Theorem: Limit Preservation of Non-Negativity for Real Sequences ([ProofWiki](https://proofwiki.org/wiki/Limit_of_Positive_Real_Sequence_is_Positive))

Given a real, convergent sequence, $\langle x_{n} \rangle \subseteq \mathbb{R},$ such that $\lim\limits_{n \to \infty} x_{n} = L$ and $x_{n} \geq 0,$ for all $n \in \mathbb{N},$ then $L \geq 0.$

#### Proof

1\. Suppose $\langle x_{n} \rangle \subseteq \mathbb{R}$ is a real, convergent sequence, such that:

$$
\forall n \in \mathbb{N}(x_{n} \geq 0) \quad \text{and} \quad \lim_{ n \to \infty } x_{n} = L
$$

2\. By definition of **sequence convergence**, $x_{n} \to L$ implies $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies |x_{n} - L| < \varepsilon
$$

3\. Assume, for **contradiction**, $L < 0.$

4\. Given any $\varepsilon > 0,$ observe $-L > 0$ and define $\varepsilon = -L.$

$$
\begin{gather}
|x_{n} - L| < -L \\
L < x_{n} - L < -L \\
L + L < x_{n} < L - L \\
2L < x_{n} < 0
\end{gather}
$$

5\. However, $x_{n} < 0$ contradicts the premise that $x_{n} > 0.$

6\. Thus, the limit of a non-negative sequence is non-negative.

### Theorem: Reciprocal of a Strictly Increasing Sequence of Positive Terms is Strictly Decreasing

Given a sequence $\langle x_{n} \rangle$ of positive, strictly increasing terms in an ordered field $\mathbb{F},$ the reciprocal sequence $\left\langle \frac{1}{x_{n}} \right\rangle$ is strictly decreasing.

$$
\begin{align}
& \forall \langle x_{n} \rangle \subseteq \mathbb{F}, \\
& \quad ( \forall n \in \mathbb{N}, ~ x_{n} > 0\quad \text{and} \quad x_{n} < x_{n+1} ) \\
& \qquad \implies \left( \forall n \in \mathbb{N}, ~ \frac{1}{x_{n}} > \frac{1}{x_{n+1}} \right)
\end{align}
$$

#### Proof

1\. Suppose $\langle x_{n} \rangle$ is a sequence of **positive**, **strictly increasing** terms in an ordered field $\mathbb{F},$ such that $\forall n \in \mathbb{N}:$

$$
x_{n} > 0 \quad \text{and} \quad x_{n} < x_{n+1}
$$

2\. Since $x_{n} > 0$ for all $n$, each reciprocal $\frac{1}{x_{n}}$ is well-defined and positive.

3\. Then, by the **Ordering of Reciprocals**,

$$
x_{n} < x_{n+1} \implies \frac{1}{x_{n}} > \frac{1}{x_{n+1}}
$$

4\. Thus, the sequence $\left\langle \frac{1}{x_{n}} \right\rangle$ is strictly decreasing.

$$
\boxed{ \left\langle \frac{1}{x_{n}} \right\rangle \text{ is strictly decreasing} }
$$

#### The Ratio Test for Sequence Convergence

Given a real sequence of non-zero numbers, $\langle x_{n} \rangle_{n = 1}^{\infty} \subseteq \mathbb{R} \setminus \{ 0 \},$ such that:

$$
\lim_{n \to \infty} \left| \frac{x_{n+1}}{x_{n}} \right| = L
$$

- If $L < 1$, then the sequence $\langle x_{n} \rangle$ converges to 0.
- If $L > 1,$ then the sequence $\langle x_{n} \rangle$ diverges.
- If $L = 1,$ then the test is inconclusive.

##### Proof: [Math Online Wiki](http://mathonline.wikidot.com/the-ratio-test-for-sequence-convergence)

###### Lemma: Vanishing Bound Lemma

Given real sequences, $\langle a_{n} \rangle_{n = 1}^{\infty}, \langle b_{n} \rangle_{n = 1}^{\infty} \subseteq \mathbb{R},$ such that $a_{n} \geq 0,$ for all $n \in \mathbb{N},$ and a real number, $K \in \mathbb{R}_{> 0},$ if $\lim\limits_{n \to \infty} a_{n} = 0$ and there exists some $N_{1} \in \mathbb{N},$ for any $n \in \mathbb{N},$ such that

$$
n \geq N_{1}  \implies |b_n - L| \leq K \cdot a_{n} \implies \lim_{n \to \infty} b_n = L
$$

###### Proof

1\. Suppose $K$ is a real, positive number and $\langle a_{n} \rangle_{n = 1}^{\infty}$ and $\langle b_{n} \rangle_{n = 1}^{\infty}$ are real sequences, such that $a_{n} \geq 0,$ for all $n \in \mathbb{N}.$

2\. Assume that $\langle a_{n} \rangle$ converges to $0,$ $\lim\limits_{n \to \infty} a_{n} = 0,$ and there exists some $N_{1} \in \mathbb{N},$ for any $n \in \mathbb{N},$ such that

$$
n \geq N_{1}  \implies |b_n - L| \leq K \cdot a_{n}
$$

3\. Given any arbitrary $\varepsilon > 0,$ observe that since $K > 0,$ then $\frac{\varepsilon}{K} > 0.$

4\. By definition of **sequence convergence**, $a_{n} \to 0$ implies $\forall \varepsilon > 0,$ $\exists N_{2} \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N_{2} \implies |a_{n} - 0| < \frac{\varepsilon}{K}
$$

5\. Hence, since $a_{n} \geq 0,$ $\forall n \in \mathbb{N},$ by factoring the quotient:

$$
\begin{align}
& |a_{n} - 0| = |a_{n}| = a_{n} < \frac{\varepsilon}{K}  \\
& \quad  \implies K \cdot a_{n} < \varepsilon
\end{align}
$$

6\. Define $N = \max\{N_{1}, N_{2}\}$ and consider any $n \geq N.$

7\. Substituting $a_{n}$ for with its bound, $\frac{\varepsilon}{K},$ $\forall n \geq N:$

$$
\begin{align}
|b_n - L| &\leq K \cdot a_{n}  \\
&< K \cdot \frac{\varepsilon}{K} = \varepsilon
\end{align}
$$

8\. Thus, by definition of **sequence convergence**, $b_{n} \to L$ since $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies |b_{n} - L| < \varepsilon
$$

###### Proof: Convergence

1\. Suppose $\langle x_{n} \rangle$ is a real, convergent sequence of positive terms, $x_{n} > 0,$ for all $n \in \mathbb{N},$ such that:

$$
\lim_{n \to \infty} \frac{x_{n+1}}{x_{n}} = L
$$

2\. Assume $L < 1$ and let $r$ be a real number, such that by the **limit preservation of non-negativity**:

$$
\begin{align}
& \forall n \in \mathbb{N}(x_{n} > 0)  \\
& \quad \implies 0 \leq L < r < 1
\end{align}
$$

3\. Given any arbitrary $\varepsilon > 0,$ observe that $r > L$ implies $r - L > 0.$

4\. By definition of **sequence convergence**, $\tfrac{x_{n+1}}{x_{n}} \to L$ implies $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that $\forall n \geq N:$

$$
\begin{gather}
\left|\frac{x_{n+1}}{x_{n}} - L \right| < \varepsilon \\
-\varepsilon < \frac{x_{n+1}}{x_{n}} - L  < \varepsilon \\
L - \varepsilon < \frac{x_{n+1}}{x_{n}} < \varepsilon + L
\end{gather}
$$

5\. Hence, substituting $r - L$ for $\varepsilon,$ $\forall n \in \mathbb{N}:$

$$
\begin{align}
\frac{x_{n+1}}{x_{n}} &< \varepsilon + L \\
\frac{x_{n+1}}{x_{n}} &< (r - L) + L \\
\frac{x_{n+1}}{x_{n}} &< r \\
x_{n + 1} &< r \cdot x_{n}
\end{align}
$$

6\. Apply the inequality $x_{n+1} < r \cdot x_{n}$ to subsequent terms and observe that each successive term is bounded above by $r$ multiplied by the previous term, such that $\forall m \in \mathbb{N}:$

$$
\begin{gather}
x_{N+1} < r \cdot x_{N} \\
x_{N+2} < r \cdot x_{N+1} < r^{2} \cdot x_{N} \\
x_{N+3} < r \cdot x_{N+2} < r^{3} \cdot x_{N} \\
\vdots \\
x_{N+m} < r^{m} \cdot x_{N}
\end{gather}
$$

7\. Since $x_{N+m} < r^m \cdot x_{N},$ $\forall m \in \mathbb{N},$ re-index the inequality in terms of $n = N + m,$ such that $\forall n \geq N:$

$$
\begin{align}
x_{n} &< r^{m} \cdot x_{N} \\
x_{N + m} &< r^{n - N} \cdot x_{N}
\end{align}
$$

8\. Define $K = \frac{x_{N}}{r^{N}}$ and recall that $x_{n} > 0,$ $\forall n \in \mathbb{N}:$

$$
\begin{align}
|x_{n}| = x_{n} &< r^{n - N} \cdot x_{N}  \\
&= \left( \frac{x_{N}}{r^{N}} \right) \cdot r^{n} = K \cdot r^{n}
\end{align}
$$

9\. By definition of **geometric sequence convergence** and the **scalar multiplication rule of convergent sequences**, since $r \in (-1, 1]:$

$$
\lim_{n \to \infty } r^{n} = 0 \implies \lim_{n \to \infty } (K \cdot r^{n}) = 0
$$

10\. Thus, by the **Vanishing Bound Lemma**, since $x_{n} < K \cdot r^{n}$ and $K \cdot r^{n} \to 0:$

$$
\lim_{n \to \infty} x_{n} = 0
$$

$$
\therefore ~ \boxed{ \forall \langle x_{n} \rangle \subseteq \mathbb{R}_{> 0} \left[ \lim_{n \to \infty } \left( \frac{x_{n + 1}}{x_{n}} \right) < 1 \implies \lim_{n \to \infty } x_{n} = 0  \right] }
$$

##### Proof: ([Math Stack Exchange](https://math.stackexchange.com/questions/1138836/proof-attempt-to-the-ratio-test-for-sequences))

###### Lemma: Recursive Growth Lemma

Given a sequence of non-zero, real numbers $\langle x_{n} \rangle \subseteq \mathbb{R} \setminus \{0\},$ for some constants $N \in \mathbb{N}$ and $r \in \mathbb{R}$ with $r > 1$, such that:

$$
\forall n \geq N, \quad |x_{n+1}| > r \cdot |x_{n}|
$$

Then for all $m \in \mathbb{N}$, it follows that:

$$
|x_{N + m}| > r^{m} \cdot |x_{N}|
$$

---

###### Proof by Mathematical Induction

We prove the claim by induction on $m \in \mathbb{N}$.

Here, $m$ indexes the number of recursive steps **starting from** index $N$, so $x_{N + m}$ is the $m^{\text{th}}$ recursive term after $x_N$.

***Base Case: $m = 1$***

We want to prove:

$$
|x_{N+1}| > r^{1} \cdot |x_{N}|
$$

This corresponds to evaluating the recursive inequality at $n = N$. Since $N \geq N$, the assumption applies directly.

1\. From the assumption:

$$
|x_{N+1}| > r \cdot |x_{N}|
$$

2\. Observe that:

$$
r \cdot |x_{N}| = r^{1} \cdot |x_{N}|
$$

3\. Therefore:

$$
|x_{N + 1}| > r^1 \cdot |x_N|
$$

> [!Note]
>
> We deliberately apply the inequality at $n = N$, the **first valid index** covered by the assumption. There is no need to consider $n > N$ here.

***Inductive Hypothesis***

Assume that for some $k \in \mathbb{N}$, the following inequality holds:

$$
|x_{N + k}| > r^{k} \cdot |x_{N}|
$$

We aim to prove that this implies:

$$
|x_{N + (k + 1)}| > r^{k + 1} \cdot |x_{N}|
$$

***Inductive Step***

1\. From the inductive hypothesis:

$$
|x_{N + k}| > r^{k} \cdot |x_{N}|
$$

2\. Multiply both sides by $r > 1$ (preserving the inequality):

$$
r \cdot |x_{N + k}| > r \cdot (r^k \cdot |x_N|) = r^{k + 1} \cdot |x_N|
$$

3\. From the lemma's assumption (applied at $n = N + k \geq N$):

$$
|x_{N + (k + 1)}| > r \cdot |x_{N + k}|
$$

4\. Combine steps 3.2 and 3.3 using transitivity of inequality:

$$
|x_{N + (k + 1)}| > r \cdot |x_{N + k}| > r^{k + 1} \cdot |x_N|
$$

***Conclusion***

By the principle of mathematical induction, we conclude:

$$
\boxed{ \therefore ~ \forall m \in \mathbb{N}, \quad |x_{N + m}| > r^{m} \cdot |x_{N}| }
$$

###### Proof: Convergence

1\. Suppose $\langle x_{n} \rangle$ is a real, sequence of non-zero terms, $x_{n} \neq 0,$ for all $n \in \mathbb{N}.$

2\. Assume, for any real number, $L \in (0, 1):$

$$
\lim_{n \to \infty} \left| \frac{x_{n + 1}}{x_{n}} \right| = L
$$

3\. Given any arbitrary $\varepsilon > 0,$ observe that $1 > L$ implies $1 - L > 0$ and then, for any $\varepsilon \in (0, 1 - L),$

$$
r := L + \varepsilon < 1
$$

4\. By definition of **sequence convergence**, $\tfrac{x_{n+1}}{x_{n}} \to L$ implies $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies \left|\frac{x_{n+1}}{x_{n}} - L \right| < \varepsilon
$$

5\. Factoring the inequality

$$
\begin{align}
& \left|\frac{x_{n+1}}{x_{n}} - L \right| < \varepsilon \\
& \left|\frac{x_{n+1}}{x_{n}} \right| < L + \varepsilon \\
& \quad \implies |x_{n + 1}| < (L + \varepsilon) |x_{n}|
\end{align}
$$

6\. Substituting $r$ for $L + \varepsilon$ and applying the inequality recursively $\forall m \in \mathbb{N}:$

$$
\begin{gather}
|x_{N + 1}| < r \cdot |x_{N}| \\
|x_{N + 2}| < r \cdot |x_{N + 1}| < r^{2} \cdot |x_{N}| \\
|x_{N + 3}| < r \cdot |x_{N + 2}| < r^{3} \cdot |x_{N}| \\
\vdots \\
|x_{N + m}| < r^{m} \cdot |x_{N}|
\end{gather}
$$

7\. By definition of **convergence of geometric sequences**, since $r \in (0, 1):$

$$
\lim_{m \to \infty} r^{m} = 0 \implies \lim_{m \to \infty } r^{m} \cdot |x_{N}| = 0
$$

8\. By the **Squeeze Theorem for Real Sequences**, since $0 \leq |x_{n + m}| < r^{m} \cdot |x_{n}|,$ $r^{m} \cdot |x_{n}| \to 0,$ and, trivially, $0 \to 0:$

$$
\lim_{m \to \infty } |x_{N + m}| = 0
$$

9\. By the **Shift Theorem for Real Sequences**,

$$
\lim_{m \to \infty } |x_{N + m}| = 0 \implies \lim_{n \to \infty } |x_{n}| = 0
$$

10\. Thus, by the **Absolute Value Theorem** for real, convergent sequences,

$$
\lim_{n \to \infty} |x_{n}| = 0 \implies \lim_{n \to \infty} x_{n} = 0
$$

$$
\therefore ~ \boxed{\forall \langle x_{n} \rangle \subseteq \mathbb{R} \setminus \{ 0 \} \left[ \forall L \in (0, 1)  \left[ \lim_{n \to \infty } \left|\frac{x_{n + 1}}{x_{n}} \right| = L \implies \lim_{n \to \infty } x_{n} = 0  \right] \right] }
$$

##### Proof: Divergence

1\. Suppose $\langle x_{n} \rangle$ is a real, sequence of non-zero terms, $x_{n} \neq 0,$ for all $n \in \mathbb{N}.$

2\. For any real number, $L \in (0, 1),$ assume:

$$
\lim_{n \to \infty} \left| \frac{x_{n + 1}}{x_{n}} \right| = L
$$

3\. Given any arbitrary $\varepsilon > 0,$ observe that $L > 1$ implies $\frac{L - 1}{2} > 0$ and define $\varepsilon:= \frac{L - 1}{2},$ such that:

$$
r := L - \varepsilon = \frac{L + 1}{2} \in (1, L)
$$

4\. By definition of **sequence convergence**, $\tfrac{x_{n+1}}{x_{n}} \to L$ implies $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies \left|\frac{x_{n+1}}{x_{n}} - L \right| < \varepsilon
$$

5\. Reversing and factoring the inequality

$$
\begin{align}
& \left|\frac{x_{n+1}}{x_{n}} - L \right| < \varepsilon \\
& \quad \implies \left|\frac{x_{n+1}}{x_{n}} \right| > L - \varepsilon \\
& \quad \implies |x_{n + 1}| > (L - \varepsilon) |x_{n}|
\end{align}
$$

6\. Substituting $r$ for $L - \varepsilon$ and applying the inequality recursively $\forall m \in \mathbb{N}:$

$$
\begin{gather}
|x_{N + 1}| > r \cdot |x_{N}| \\
|x_{N + 2}| > r \cdot |x_{N + 1}| > r^{2} \cdot |x_{N}| \\
|x_{N + 3}| > r \cdot |x_{N + 2}| > r^{3} \cdot |x_{N}| \\
\vdots \\
|x_{N + m}| > r^{m} \cdot |x_{N}|
\end{gather}
$$

7\. By definition of **exponential sequence divergence** and the **scalar multiplication rule of convergent sequences**, since $r > 1:$

$$
\lim_{m \to \infty} r^{m} = \infty \implies \lim_{m \to \infty } r^{m} \cdot |x_{N}| = \infty
$$

8\. By the **Comparison Theorem for Real Sequences**, since $|x_{N+m}| > r^{m} \cdot |x_{N}|$ and $r^{m} \cdot |x_{N}| \to \infty,$ then $\forall m \in \mathbb{N}:$

$$
\lim_{m \to \infty } |x_{N + m}| = \infty
$$

9\. By the **Shift Theorem for Real Sequences**,

$$
\lim_{m \to \infty } |x_{N + m}| = \infty \implies \lim_{n \to \infty } |x_{n}| = \infty
$$

10\. Since $|x_{n}| \to \infty,$ it follows that $x_n$ diverges, such that the form of divergence depends on the sign behavior of the sequence:

$$
x_n \to
\begin{cases}
+\infty & \text{if } \exists N,\ \forall n \geq N,\ x_n > 0 \text{ and increasing} \\
-\infty & \text{if } \exists N,\ \forall n \geq N,\ x_n < 0 \text{ and decreasing} \\
\text{does not exist} & \text{if } x_n \text{ oscillates or changes sign infinitely often}
\end{cases}
$$

$$
x_{n} \to
\begin{cases}
+\infty & \text{if } x_{n} > 0 \text{ eventually} \\
-\infty & \text{if } x_{n} < 0 \text{ eventually} \\
\text{does not exist} & \text{if } x_{n} \text{ changes sign infinitely often}
\end{cases}
$$

$$
\therefore ~ \boxed{\forall \langle x_{n} \rangle \subseteq \mathbb{R} \setminus \{ 0 \} \left[ \forall L \geq 1  \left[ \lim_{n \to \infty } \left| \frac{x_{n + 1}}{x_{n}} \right| = L \implies \lim_{n \to \infty } x_{n} ~ \text{Diverges}  \right] \right] }
$$

---

#### Absolute Value Theorem

Given a real sequence, ${\langle x_{n} \rangle}_{n = 1}^{\infty} \subseteq \mathbb{R},$ if the absolute value of the sequence, $|x_{n}|,$ converges to 0, then the sequence converges to 0.

$$
\lim_{n \to \infty}  |x_{n}| = 0 \implies \lim_{n \to \infty} x_{n} = 0
$$

##### Proof

1\. Suppose ${\langle x_{n} \rangle}_{n = 1}^{\infty}$ is a real sequence and $|x_{n}|$ is the absolute value form of the sequence.

2\. Assume the sequence's absolute value form converges to 0:

$$
\lim_{n \to \infty} |x_{n}| = 0
$$

3\. By definition of **sequence convergence**, $|x_{n}| \to 0$ implies $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that if $n \geq N:$

$$
||x_{n}| - 0| < \varepsilon \implies |x_{n}| < \varepsilon
$$

4\. Observe that $|x_{n}| = |x_{n} - 0|,$ such that $\forall n \geq N:$

$$
| x_{n} - 0 | < \varepsilon
$$

5\. Thus, by definition of **sequence convergence**, $x_{n} \to 0,$ since $\forall \varepsilon > 0,$ $\exists N \in \mathbb{N},$ $\forall n \in \mathbb{N},$ such that:

$$
n \geq N \implies |x_{n} - 0| < \varepsilon
$$

$$
\therefore ~ \boxed{ \lim_{n \to \infty} |x_{n}| = 0 \implies \lim_{n \to \infty} x_{n} = 0 }
$$
