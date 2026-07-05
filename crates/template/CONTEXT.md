# Template Context Glossary

## Core Terms

- **Extension** (or **Template Extension**): A first-party, Traces-provided built-in function, filter, or test injected into the template engine. This is *not* a user-defined plugin or script. Extensions provide domain-specific capabilities (like date formatting or path manipulation) directly to template authors.
- **Pure Extension**: An extension that transforms data without side effects or external state (e.g., string formatting, date math, path manipulation).
- **Effectful Extension**: An extension that performs I/O or side effects during rendering (e.g., reading external files, prompting the user for input).
