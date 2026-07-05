# Template Context Glossary

## Core Terms

- **Extension** (or **Template Extension**): A first-party, Traces-provided built-in function, filter, or test injected into the template engine. This is *not* a user-defined plugin or script. Extensions provide domain-specific capabilities (like date formatting or path manipulation) directly to template authors.
- **Pure Extension**: An extension that transforms data without side effects or external state (e.g., string formatting, date math, path manipulation).
- **Effectful Extension**: An extension that performs I/O or side effects during rendering (e.g., reading external files, prompting the user for input). These capabilities are provided via **State Injection**, passing a capability context into MiniJinja's render state.
- **Function**: An extension that generates data or performs side effects and is called directly (e.g., `{{ date.now() }}`).
- **Filter**: An extension that transforms data and is called using the pipe operator (e.g., `{{ my_string | str.slugify }}`).
- **Test**: An extension that evaluates a condition and returns a boolean (e.g., `{% if my_path is path.is_file %}`).
- **ExtensionRegistry**: A component built by the application layer that aggregates and registers all pure and effectful domain extensions into the underlying template engine.
- **State Temps**: An implementation mechanism in MiniJinja where temporary objects are attached to the rendering state. Used to securely pass effectful capabilities (like an I/O host) into extensions and to extract side-channel output (like output paths) post-render.
