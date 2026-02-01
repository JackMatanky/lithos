# Tech Spec (Design Doc) Process

This directory contains **Technical Specifications** (Tech Specs). These documents capture the _tactical_ design of features before we write code.

## Why write a Tech Spec?

> "Cheapest place to fix a bug is on the whiteboard."

We write specs to:

1.  **Force "Working Backwards"**: Design the API/Experience before the Implementation.
2.  **Define Boundaries**: Explicitly define the contracts between components to prevent integration hell.
3.  **Catch "Impossible" Requirements**: Identify constraints (Allocations, Threading) early.

## Methodology & Best Practices

The template is a thinking tool. Follow this order of operations to get the most out of it:

### 1. The "Box" (Section 1)

Define what success looks like _before_ you design.

- **Constraint Check**: Do not start designing if you don't know the budget (latency, memory, time).

### 2. The Interface (Section 2 & 3.2)

Write the "User Manual" (Section 2) and the "Component Interface" (Section 3.2) before any internals.

- **Rule**: If the API is hard to explain, the design is wrong.

### 3. The Integration (Section 3.3)

Systems fail at the edges. Explicitly map how your component talks to others.

- **Sequence Diagrams**: Use MermaidJS to visualize the flow.
- **Failure Modes**: What happens if the other component is slow/down?

### 4. The Critique (Section 7)

Design is iterative. Use the Critique Log to challenge your own assumptions.

- **Self-Correction**: It is better to find a flaw in the doc than in the code.

---

## AI-Assisted Workflow

The template is designed to be filled by an AI agent acting as a Principal Engineer.

### The "Design Partner" Prompt

Copy-paste this prompt to your AI agent to start a high-quality spec:

```text
Act as a Principal Software Architect. I need to write a Tech Spec for a new feature: [FEATURE NAME/DESCRIPTION].

Your goal is to help me "Work Backwards" from the user experience.

1.  **Read**: `docs/design/template.md`.
2.  **Interview Me**: Ask 3-5 sharp questions to clarify the "Problem Space" (Constraints, Non-Goals) and the "User Experience" (How should the API feel?).
3.  **Draft**: Once I answer, draft the "Guide-Level Explanation" (Section 2) and "Component Specifications" (Section 3).
4.  **Integration Check**: Explicitly model the "Integration & Data Flow" (Section 3.3). Who calls who? What events are emitted?
5.  **Critique**: After drafting, assume the persona of a hostile SRE. Identify 3 ways this design could fail in production (The "Pre-Mortem").
6.  **Output**: Fill the template with our agreed design.

Do not ask me to fill the sections. You fill them based on my answers.
```

## The Lifecycle

1.  **Draft**: Create `docs/design/NNN-feature-name.md` using the template.
2.  **Critique**: Use the "Critique Log" in the template. Challenge your own assumptions.
3.  **Approve**: Once the "Critique Log" shows resolution, move Status to `Approved`.
4.  **Implement**: Write the code.
5.  **Archive**: Once shipped, mark Status as `Implemented`. You do _not_ need to keep this doc in sync with code forever. It is a historical plan.
