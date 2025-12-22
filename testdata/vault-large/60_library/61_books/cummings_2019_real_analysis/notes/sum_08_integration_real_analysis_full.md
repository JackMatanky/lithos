---
title: sum_08_integration_real_analysis_full
uuid: c24a5fe1-44c9-44bd-a419-808f055df5bf
aliases:
  - "Full Summary of Real Analysis: Integration"
  - "full summary of real analysis: integration"
  - full_summary_of_real_analysis_integration
  - sum_08_integration_real_analysis_full
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
  - "[[08_integration_real_analysis|Real Analysis: Integration]]"
about: |-
 Chapter 8 of Cummings’s _Real Analysis_ introduces Riemann integration by defining upper and lower sums and integrals, establishing that a function is integrable if these agree. It proves that continuous functions on closed intervals are always integrable, using uniform continuity to control oscillation over small intervals. The chapter develops essential properties of the integral—linearity, additivity over intervals, and comparison—before culminating in the Fundamental Theorem of Calculus, which connects integration and differentiation. The theorem asserts that the integral of a function defines an antiderivative and that integration can be computed via antiderivatives when they exist.
url:
status: develop
type: summary
file_class: pkm_zettel
date_created: 2025-02-14T12:44
date_modified: 2025-10-05T17:48
tags:
---
# Full Summary of Real Analysis: Integration

> [!Summary]
>
> - **Resource**: `dv: this.file.frontmatter.library[0]`
>
> - **Source**:: [[Cummings_2019_Real Analysis_08_Integration.pdf|Real Analysis: Integration, by Jay Cummings]]
>
> - **Parent**:: [[sum_08_integration_real_analysis|Summary of Real Analysis: Integration]]

---

## Section 8.1: Integrability

### Guiding Questions

- What does it mean for a function to be Riemann integrable?
- How do upper and lower sums approximate the integral of a function?
- What is the connection between continuity and integrability?
- Can a discontinuous function be integrable?

### Key Terms

#### Lower and Upper Sums (Page 237, Def 8.1.1)

Let $f: [a, b] \to \mathbb{R}$ be bounded.

- The **lower sum** $L(f, P)$ for a partition $P = \{x_0, x_1, \ldots, x_n\}$ is:

$$
L(f, P) = \sum_{i=1}^n m_i (x_i - x_{i-1})
$$

where $m_i = \inf \{ f(x): x \in [x_{i-1}, x_i] \}$

- The **upper sum** $U(f, P)$ is:

$$
U(f, P) = \sum_{i=1}^n M_i (x_i - x_{i-1})
$$

where $M_i = \sup \{ f(x): x \in [x_{i-1}, x_i] \}$

#### Lower and Upper Integrals (Page 238, Def 8.1.2)

- The **lower integral** of $f$ over $[a, b]$ is:

$$
\underline{\int_a^b} f = \sup \{ L(f, P) : P \text{ a partition of } [a, b] \}
$$

- The **upper integral** is:

$$
\overline{\int_a^b} f = \inf \{ U(f, P) : P \text{ a partition of } [a, b] \}
$$

#### Riemann Integrability (Page 238, Def 8.1.3)

$f$ is **Riemann integrable** on $[a, b]$ if:

$$
\underline{\int_a^b} f = \overline{\int_a^b} f
$$

In this case, the **integral** $\int_a^b f$ is defined as their common value.

> [!Note]
> Integrability depends on the behavior of $f$, not just its values at finitely many points. Functions with discontinuities can still be integrable.

#### Example: Dirichlet Function (Page 238, Ex 8.1.4)

The function:

$$
f(x) = \begin{cases}
1 & \text{if } x \in \mathbb{Q} \cap [0,1] \\
0 & \text{if } x \notin \mathbb{Q} \cap [0,1]
\end{cases}
$$

is not Riemann integrable, because the lower sum is always 0, and the upper sum is always 1 for any partition.

#### Characterization of Integrability (Page 239, Thm 8.1.5)

Let $f: [a, b] \to \mathbb{R}$ be bounded. Then $f$ is Riemann integrable if and only if:

$$
\forall \varepsilon > 0, \ \exists \text{ a partition } P \text{ of } [a, b] \text{ such that } U(f, P) - L(f, P) < \varepsilon
$$

**Proof**:

1\. From the definitions of upper and lower integrals, the infimum and supremum can be approximated within any $\varepsilon > 0$.

2\. If $f$ is integrable, then the difference between the upper and lower integrals is zero, so for any $\varepsilon > 0$, a partition exists where the difference is less than $\varepsilon$.

3\. Conversely, if such partitions exist, then the upper and lower integrals must be equal.

---

## Section 8.2: Continuous Functions Are Integrable

### Guiding Questions

- Why does continuity guarantee integrability?
- How does uniform continuity on closed intervals help with integration?
- What role does the oscillation of a function play in determining integrability?

### Key Terms

#### Theorem: Continuous Functions Are Integrable (Page 242, Thm 8.2.1)

If $f: [a, b] \to \mathbb{R}$ is continuous, then $f$ is Riemann integrable.

**Proof**:

1\. A continuous function on a closed interval is uniformly continuous.

2\. For a given $\varepsilon > 0$, choose $\delta > 0$ so that:

$$
|x - y| < \delta \Rightarrow |f(x) - f(y)| < \frac{\varepsilon}{b - a}
$$

3\. Partition $[a, b]$ into intervals of length less than $\delta$. Over each interval, the oscillation $M_i - m_i < \frac{\varepsilon}{b - a}$.

4\. Then the difference $U(f, P) - L(f, P) < \varepsilon$, so $f$ is integrable by Theorem 8.1.5.

---

## Section 8.3: Properties of the Integral

### Guiding Questions

- How does the integral behave with respect to linear combinations of functions?
- What does additivity over intervals mean in the context of integration?
- What are the comparison and absolute value properties of integrals?

### Key Terms

#### Linearity (Page 243, Thm 8.3.1)

If $f, g$ are integrable on $[a, b]$, and $\alpha, \beta \in \mathbb{R}$, then:

$$
\int_a^b (\alpha f + \beta g) = \alpha \int_a^b f + \beta \int_a^b g
$$

#### Additivity over Intervals (Page 243, Thm 8.3.2)

If $f$ is integrable on $[a, c]$, and $b \in (a, c)$, then:

$$
\int_a^c f = \int_a^b f + \int_b^c f
$$

#### Comparison Theorem (Page 243, Thm 8.3.3)

If $f, g$ are integrable and $f(x) \leq g(x)$ for all $x \in [a, b]$, then:

$$
\int_a^b f \leq \int_a^b g
$$

#### Absolute Value (Page 243, Thm 8.3.4)

If $f$ is integrable on $[a, b]$, then $|f|$ is integrable, and:

$$
\left| \int_a^b f \right| \leq \int_a^b |f|
$$

---

## Section 8.4: The Fundamental Theorem of Calculus

### Guiding Questions

- What is the relationship between integration and differentiation?
- How does the antiderivative of a function relate to its integral?
- Can every integrable function be integrated by finding an antiderivative?

### Key Terms

#### Integral Function (Page 245, Def 8.4.1)

Given $f$ integrable on $[a, b]$, define:

$$
F(x) = \int_a^x f(t)\,dt
$$

Then $F$ is called the **integral function** associated to $f$.

#### FTC Part I (Page 246, Thm 8.4.2)

If $f$ is continuous on $[a, b]$, and $F(x) = \int_a^x f(t)\,dt$, then $F$ is differentiable on $(a, b)$, and:

$$
F'(x) = f(x)
$$

**Proof**:

1\. From the definition of the derivative:

$$
F'(x) = \lim_{h \to 0} \frac{1}{h} \int_x^{x+h} f(t)\,dt
$$

2\. Since $f$ is continuous, it is uniformly continuous on $[a, b]$, so the integral approximates $f(x) \cdot h$, and the limit becomes $f(x)$.

#### FTC Part II (Page 247, Thm 8.4.3)

If $f$ is integrable on $[a, b]$, and $F$ is any function such that $F'(x) = f(x)$ on $(a, b)$, then:

$$
\int_a^b f(x)\,dx = F(b) - F(a)
$$

**Proof**:

1\. Apply the Mean Value Theorem to relate changes in $F$ to values of $f$.

2\. Approximate the integral over partitions and use the derivative condition $F'(x) = f(x)$ to obtain the result as a telescoping sum.

---
