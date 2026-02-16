//! Template lifecycle manager (load → compile → cache → render).

use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use minijinja::{AutoEscape, Environment, UndefinedBehavior};

use crate::template::{
    Template,
    adapter::{FilterRegistry, SourceGenerator},
    error::TemplateError,
    ports::Query,
};

/// Template catalog: orchestrates loading, compilation, and rendering.
///
/// # Responsibilities
/// - Loads all templates from storage (via `TemplateQueryPort`).
/// - Topologically sorts by extends relationships (parents before children).
/// - Compiles templates via `SourceGenerator` + `MiniJinja`.
/// - Caches compiled templates in `Arc<Environment>` (shared across threads).
/// - Provides unified render API.
pub struct TemplateCatalog {
    /// Compiled templates (shared across threads).
    env: Arc<Environment<'static>>,

    /// Domain metadata storage (for template queries).
    metadata: Box<dyn Query>,
}

impl TemplateCatalog {
    /// Constructs catalog with storage backend.
    ///
    /// Configures `MiniJinja` Environment:
    /// - Strict undefined behavior (fail on {{ undefined }})
    /// - Max template depth: 10 (prevent infinite recursion)
    /// - Auto-escape: None (we render Markdown, not HTML)
    /// - Registers custom filters (`validate_length`, etc.)
    ///
    /// # Errors
    /// Returns `TemplateError` if initialization fails.
    #[inline]
    pub fn new(metadata: Box<dyn Query>) -> Result<Self, TemplateError> {
        let mut env = Environment::new();
        env.set_undefined_behavior(UndefinedBehavior::Strict);
        env.set_recursion_limit(10);
        env.set_auto_escape_callback(|_| AutoEscape::None);

        FilterRegistry::register_all(&mut env);

        Ok(Self {
            env: Arc::new(env),
            metadata,
        })
    }

    /// Loads and compiles ALL templates from storage.
    ///
    /// # Algorithm
    /// 1. Load all template metadata from storage
    /// 2. Build dependency graph (who extends whom)
    /// 3. Topologically sort (Kahn's algorithm)
    /// 4. For each template in sorted order: a. Generate `MiniJinja` source b.
    ///    Compile with `Environment.add_template()`
    ///
    /// # Performance
    /// O(N) templates × compilation cost. Call ONCE at startup.
    ///
    /// # Errors
    /// - Storage: Database read failed
    /// - `CircularComposition`: Cycle detected in extends
    /// - Syntax: Generated `MiniJinja` source invalid or compilation failed
    ///
    /// # Panics
    /// Panics if the internal `Environment` is not exclusively owned during
    /// loading.
    #[inline]
    pub fn load_all(&mut self) -> Result<(), TemplateError> {
        // 1. Load all templates from storage
        let templates = self.metadata.list()?;

        // 2. Topologically sort by extends (parents before children)
        let sorted = Self::topological_sort(&templates)?;

        // 3. Compile in dependency order
        #[expect(
            clippy::expect_used,
            reason = "Environment is exclusively owned during load phase"
        )]
        let env = Arc::get_mut(&mut self.env).expect(
            "Environment should be exclusively owned during load phase",
        );

        for template in sorted {
            let source = SourceGenerator::generate(template);

            // Leak strings to meet Environment<'static> requirement.
            // Templates are loaded once at startup and are permanent.
            let name_static: &'static str =
                Box::leak(template.name().to_owned().into_boxed_str());
            let source_static: &'static str =
                Box::leak(source.into_boxed_str());

            env.add_template(name_static, source_static).map_err(|e| {
                TemplateError::Syntax(format!(
                    "Failed to compile template '{}': {}",
                    template.name(),
                    e
                ))
            })?;
        }

        Ok(())
    }

    /// Renders a compiled template with context.
    ///
    /// # Performance
    /// O(1) lookup + O(AST size) execution. This is the FAST PATH (no I/O, no
    /// parsing).
    ///
    /// # Errors
    /// - `NotFound`: Template not compiled (did you call `load_all()`?)
    /// - Render: Undefined variable, filter validation failed, or other render
    ///   error
    #[inline]
    pub fn render<S: serde::Serialize>(
        &self,
        name: &str,
        context: S,
    ) -> Result<String, TemplateError> {
        let tmpl = self
            .env
            .get_template(name)
            .map_err(|_e| TemplateError::NotFound(name.into()))?;

        tmpl.render(context)
            .map_err(|e: minijinja::Error| TemplateError::Render(e.to_string()))
    }

    /// Lists all template names (for discovery).
    ///
    /// # Errors
    /// Returns `TemplateError` if storage fails.
    #[inline]
    pub fn list_names(&self) -> Result<Vec<String>, TemplateError> {
        let templates = self.metadata.list()?;
        Ok(templates.into_iter().map(|t| t.name().into()).collect())
    }

    /// Topologically sorts templates by extends relationships (Kahn's
    /// algorithm).
    ///
    /// # Errors
    /// Returns `CircularComposition` if a cycle is detected.
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Checked by graph construction"
    )]
    fn topological_sort(
        templates: &[Template],
    ) -> Result<Vec<&Template>, TemplateError> {
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut template_map: HashMap<&str, &Template> = HashMap::new();

        for template in templates {
            template_map.insert(template.name(), template);
            in_degree.entry(template.name()).or_insert(0);

            if let Some(parent) = template.extends() {
                graph.entry(parent).or_default().push(template.name());
                *in_degree.entry(template.name()).or_insert(0) += 1;
            }
        }

        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&name, _)| name)
            .collect();

        let mut sorted = Vec::new();

        while let Some(current) = queue.pop_front() {
            #[expect(
                clippy::indexing_slicing,
                reason = "Key existence guaranteed by graph construction"
            )]
            sorted.push(template_map[current]);

            let Some(children) = graph.get(current) else {
                continue;
            };

            for &child in children {
                let deg = in_degree.get_mut(child).ok_or_else(|| {
                    TemplateError::Storage(format!(
                        "Child {child} not found in degrees"
                    ))
                })?;

                *deg -= 1;

                if *deg == 0 {
                    queue.push_back(child);
                }
            }
        }

        if sorted.len() != templates.len() {
            return Err(TemplateError::CircularComposition(
                "Cycle detected in template extends relationships".into(),
            ));
        }

        Ok(sorted)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::template::{
        BlockStrategy, TemplateBlock,
        ports::{Command as _, FakeTemplateStorage},
    };

    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Tests use unwrap for concise setup"
    )]
    fn loads_and_compiles_all_templates() {
        let storage = FakeTemplateStorage::new();

        // Create parent template
        let parent = Template::new(
            "parent",
            None,
            vec![TemplateBlock::new(
                "title",
                "Default Title",
                BlockStrategy::Replace,
            )],
            HashMap::new(),
        )
        .unwrap();

        // Create child template
        let child = Template::new(
            "child",
            Some("parent"),
            vec![TemplateBlock::new(
                "title",
                "Custom Title",
                BlockStrategy::Replace,
            )],
            HashMap::new(),
        )
        .unwrap();

        storage.create(&parent).unwrap();
        storage.create(&child).unwrap();

        // Load all into catalog (automatic topological sort)
        let mut catalog = TemplateCatalog::new(Box::new(storage)).unwrap();
        catalog.load_all().unwrap();

        // Render child template
        let output = catalog.render("child", minijinja::context! {}).unwrap();
        assert!(output.contains("Custom Title"));
        assert!(!output.contains("Default Title"));
    }

    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Tests use unwrap for concise setup"
    )]
    fn detects_circular_extends() {
        let storage = FakeTemplateStorage::new();

        // Create circular dependency: A extends B, B extends A
        let a = Template::new("a", Some("b"), vec![], HashMap::new()).unwrap();

        let b = Template::new("b", Some("a"), vec![], HashMap::new()).unwrap();

        storage.create(&a).unwrap();
        storage.create(&b).unwrap();

        // Load should fail with cycle detection error
        let mut catalog = TemplateCatalog::new(Box::new(storage)).unwrap();
        let result = catalog.load_all();

        assert!(matches!(result, Err(TemplateError::CircularComposition(_))));
    }

    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Tests use unwrap for concise setup"
    )]
    fn topological_sort_compiles_parents_before_children() {
        let storage = FakeTemplateStorage::new();

        // Create 3-level hierarchy: grandparent <- parent <- child
        let grandparent = Template::new(
            "grandparent",
            None,
            vec![TemplateBlock::new(
                "a",
                "Grandparent",
                BlockStrategy::Replace,
            )],
            HashMap::new(),
        )
        .unwrap();

        let parent = Template::new(
            "parent",
            Some("grandparent"),
            vec![TemplateBlock::new("a", "Parent", BlockStrategy::Replace)],
            HashMap::new(),
        )
        .unwrap();

        let child = Template::new(
            "child",
            Some("parent"),
            vec![TemplateBlock::new("a", "Child", BlockStrategy::Replace)],
            HashMap::new(),
        )
        .unwrap();

        // Store in WRONG order (child first)
        storage.create(&child).unwrap();
        storage.create(&grandparent).unwrap();
        storage.create(&parent).unwrap();

        // Catalog should sort them correctly
        let mut catalog = TemplateCatalog::new(Box::new(storage)).unwrap();
        catalog.load_all().unwrap();

        // All three should render correctly
        #[expect(
            clippy::assertions_on_result_states,
            reason = "Verifying success"
        )]
        {
            assert!(
                catalog.render("grandparent", minijinja::context! {}).is_ok()
            );
            assert!(catalog.render("parent", minijinja::context! {}).is_ok());
            assert!(catalog.render("child", minijinja::context! {}).is_ok());
        }
    }

    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Tests use unwrap for concise setup"
    )]
    fn list_names_returns_all_templates() {
        let storage = FakeTemplateStorage::new();

        let t1 = Template::new("t1", None, vec![], HashMap::new()).unwrap();
        let t2 = Template::new("t2", None, vec![], HashMap::new()).unwrap();

        storage.create(&t1).unwrap();
        storage.create(&t2).unwrap();

        let catalog = TemplateCatalog::new(Box::new(storage)).unwrap();
        let names = catalog.list_names().unwrap();

        assert_eq!(names.len(), 2);
        assert!(names.contains(&"t1".to_owned()));
        assert!(names.contains(&"t2".to_owned()));
    }
}
