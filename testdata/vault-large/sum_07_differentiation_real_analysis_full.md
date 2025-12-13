---
title: sum_07_differentiation_real_analysis_full
uuid: 76dbcdae-22d8-450a-9033-1e03b8e99f15
aliases:
  - "Full Summary of Real Analysis: Differentiation"
  - "full summary of real analysis: differentiation"
  - full_summary_of_real_analysis_differentiation
  - sum_07_differentiation_real_analysis_full
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
  - "[[07_differentiation_real_analysis|Real Analysis: Differentiation]]"
about: |-
 Chapter 7 of Cummings’s _Real Analysis_ develops the concept of the derivative as a limit of difference quotients, defining differentiability and establishing that it implies continuity. It presents core rules for differentiating sums, products, quotients, compositions, and reciprocals, culminating in the Chain Rule. The chapter then introduces the Mean Value Theorem and Rolle’s Theorem, foundational results connecting average and instantaneous rates of change. Key consequences include criteria for constant, increasing, or decreasing behavior of functions based on their derivatives. Finally, it illustrates that not all continuous functions are differentiable, highlighting examples like the absolute value function where sharp points prevent differentiability.
url:
status: develop
type: summary
file_class: pkm_zettel
date_created: 2025-02-14T12:35
date_modified: 2025-10-05T17:48
tags:
---
# Full Summary of Real Analysis: Differentiation

> [!Summary]
>
> - **Resource**: `dv: this.file.frontmatter.library[0]`
>
> - **Source**:: [[Cummings_2019_Real Analysis_07_Differentiation.pdf|Real Analysis: Differentiation, by Jay Cummings]]
>
> - **Parent**:: [[sum_07_differentiation_real_analysis_full|Full Summary of Real Analysis: Differentiation]]

---

## Section 7.1: The Derivative

### Guiding Questions

- What is the formal definition of a derivative?
- How is differentiability related to continuity?
- Can a function be continuous but not differentiable?
- What are examples of functions that are not differentiable at some points?

### Key Terms

#### Derivative at a Point (Page 132, Def 7.1.1)

Let $f: A \to \mathbb{R}$ and let $a \in A$ be an interior point. The **derivative of $f$ at $a$**, denoted $f'(a)$, exists if the following limit exists:

$$
f'(a) = \lim_{x \to a} \frac{f(x) - f(a)}{x - a}
$$

> [!Note]
> This is called the **difference quotient**.

#### Differentiable Function (Page 132, Def 7.1.2)

A function $f: A \to \mathbb{R}$ is **differentiable** on $A$ if $f'(a)$ exists for all interior points $a \in A$.

#### Differentiability Implies Continuity (Page 133, Thm 7.1.3)

If $f$ is differentiable at $a$, then $f$ is continuous at $a$.

##### Proof of Theorem 7.1.3

**Theorem:** If $f$ is differentiable at $a$, then it is continuous at $a$.

**Proof:**

1\. By Def 7.1.1, since $f'(a)$ exists, the limit

$$
\lim_{x \to a} \frac{f(x) - f(a)}{x - a}
$$

exists.

2\. Let

$$
f(x) - f(a) = (x - a) \cdot \frac{f(x) - f(a)}{x - a}
$$

Then as $x \to a$:

- $x - a \to 0$
- $\frac{f(x) - f(a)}{x - a} \to f'(a)$

Hence $f(x) - f(a) \to 0$ and so $f(x) \to f(a)$.

---

## Section 7.2: Derivative Rules

### Guiding Questions

- How does the derivative interact with addition, multiplication, and composition?
- What is the chain rule and why is it important?
- Can we differentiate the reciprocal or quotient of two functions?

### Key Terms

#### Derivative Sum, Product, Constant, and Multiple Rules (Page 134, Thm 7.2.1)

Let $f$ and $g$ be differentiable at $a$, and $c \in \mathbb{R}$. Then:

1. $(f + g)'(a) = f'(a) + g'(a)$
2. $(cf)'(a) = c \cdot f'(a)$
3. $(f \cdot g)'(a) = f(a) \cdot g'(a) + f'(a) \cdot g(a)$

> [!Note]
> These are derived using algebraic manipulation and limit properties.

#### Derivative of the Reciprocal (Page 135, Thm 7.2.2)

If $f$ is differentiable at $a$ and $f(a) \neq 0$, then

$$
\left(\frac{1}{f}\right)'(a) = -\frac{f'(a)}{f(a)^2}
$$

#### Quotient Rule (Page 135, Cor 7.2.3)

If $f$ and $g$ are differentiable at $a$ and $g(a) \neq 0$, then

$$
\left(\frac{f}{g}\right)'(a) = \frac{f'(a) \cdot g(a) - f(a) \cdot g'(a)}{g(a)^2}
$$

#### Chain Rule (Page 136, Thm 7.2.4)

Let $f$ be differentiable at $a$, and let $g$ be differentiable at $f(a)$. Then $g \circ f$ is differentiable at $a$ and:

$$
(g \circ f)'(a) = g'(f(a)) \cdot f'(a)
$$

##### Proof of Theorem 7.2.4

**Theorem:** If $f$ is differentiable at $a$ and $g$ is differentiable at $f(a)$, then $(g \circ f)'(a) = g'(f(a)) \cdot f'(a)$.

**Proof:**

1\. Let $h = g \circ f$ and consider the difference quotient:

$$
\frac{h(x) - h(a)}{x - a} = \frac{g(f(x)) - g(f(a))}{x - a}
$$

2\. Multiply and divide by $f(x) - f(a)$:

$$
= \frac{g(f(x)) - g(f(a))}{f(x) - f(a)} \cdot \frac{f(x) - f(a)}{x - a}
$$

3\. As $x \to a$:

- $\frac{g(f(x)) - g(f(a))}{f(x) - f(a)} \to g'(f(a))$
- $\frac{f(x) - f(a)}{x - a} \to f'(a)$

Hence:

$$
(g \circ f)'(a) = g'(f(a)) \cdot f'(a)
$$

---

## Section 7.3: The Mean Value Theorem

### Guiding Questions

- What does the Mean Value Theorem say about the behavior of differentiable functions?
- How does it generalize the idea of instantaneous rate of change?
- How is Rolle's Theorem a special case of the Mean Value Theorem?

### Key Terms

#### Rolle's Theorem (Page 137, Thm 7.3.1)

Let $f: [a, b] \to \mathbb{R}$ be continuous on $[a,b]$ and differentiable on $(a,b)$. If $f(a) = f(b)$, then there exists $c \in (a, b)$ such that $f'(c) = 0$.

#### Mean Value Theorem (Page 137, Thm 7.3.2)

Let $f: [a, b] \to \mathbb{R}$ be continuous on $[a,b]$ and differentiable on $(a,b)$. Then there exists $c \in (a, b)$ such that

$$
f'(c) = \frac{f(b) - f(a)}{b - a}
$$

> [!Note]
> This tells us there's a point where the instantaneous rate equals the average rate over $[a, b]$.

##### Proof of Theorem 7.3.2: Mean Value Theorem

**Proof:**

1\. Define $g(x) = f(x) - \frac{f(b) - f(a)}{b - a}(x - a)$.

2\. Then $g(a) = f(a)$ and

$$
g(b) = f(b) - \frac{f(b) - f(a)}{b - a}(b - a) = f(a)
$$

So $g(a) = g(b)$.

3\. $g$ is continuous on $[a,b]$ and differentiable on $(a,b)$, so by Rolle's Theorem, there exists $c \in (a,b)$ such that $g'(c) = 0$.

4\. Compute:

$$
g'(x) = f'(x) - \frac{f(b) - f(a)}{b - a}
$$

So $g'(c) = 0$ implies:

$$
f'(c) = \frac{f(b) - f(a)}{b - a}
$$

---

## Section 7.4: Consequences of the Mean Value Theorem

### Guiding Questions

- What can the derivative tell us about function monotonicity?
- What is the converse of the Mean Value Theorem?
- Can two functions with the same derivative differ?

### Key Terms

#### Derivative Zero Implies Constant (Page 139, Cor 7.4.1)

If $f'(x) = 0$ for all $x \in (a,b)$, then $f$ is constant on $(a,b)$.

#### Increasing/Decreasing Functions (Page 139, Cor 7.4.2)

Let $f$ be differentiable on $(a,b)$.

- If $f'(x) > 0$ for all $x \in (a,b)$, then $f$ is strictly increasing.
- If $f'(x) < 0$ for all $x \in (a,b)$, then $f$ is strictly decreasing.

#### Equality of Derivatives Implies Equality of Functions (Page 140, Cor 7.4.3)

If $f'(x) = g'(x)$ on $(a,b)$, then $f(x) = g(x) + c$ for some constant $c$.

---

## Section 7.5: Differentiability and Continuity

### Guiding Questions

- What are examples of continuous but non-differentiable functions?
- Why does differentiability fail at cusps or corners?
- Is the absolute value function differentiable?

### Key Terms

#### Non-Differentiable Example (Page 141, Ex 7.5.1)

The function $f(x) = |x|$ is continuous everywhere but not differentiable at $x = 0$.

> [!Note]
> The one-sided difference quotients yield different limits at 0:
> - From the left: $\frac{|x| - 0}{x} = \frac{-x}{x} = -1$
> - From the right: $\frac{|x| - 0}{x} = \frac{x}{x} = 1$
> So the limit does not exist at 0.
