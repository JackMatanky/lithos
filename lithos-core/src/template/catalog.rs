//! Template lifecycle manager (load → compile → cache → render).

use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use minijinja::{AutoEscape, Environment, UndefinedBehavior};

use crate::template::{
    adapter::{Emitter, FilterRegistry},
    aggregate::{Template, TemplateName},
    error::TemplateError,
    repository::ReadRepository,
};

/// Template catalog: orchestrates loading, compilation, and rendering.
///
/// # Responsibilities
/// - Loads all templates from storage (via `ReadRepository` trait).
/// - Topologically sorts by extends relationships (parents before children).
/// - Compiles templates via `Emitter` + `MiniJinja`.
/// - Caches compiled templates in `Arc<Environment>` (shared across threads).
/// - Provides unified render API.
pub struct TemplateCatalog<R: ReadRepository> {
    /// Compiled templates (shared across threads).
    env: Arc<Environment<'static>>,

    /// Domain metadata storage (for template queries).
    repository: R,
}

impl<R: ReadRepository> TemplateCatalog<R> {
    /// Constructs catalog with storage backend.
    ///
    /// Configures `MiniJinja` Environment:
    /// - Strict undefined behavior (fail on missing inputs)
    /// - Max template depth: 10 (prevent infinite recursion)
    /// - Auto-escape: None (we render Markdown, not HTML)
    /// - Registers custom filters (`validate_length`, etc.)
    ///
    /// # Errors
    /// Returns `TemplateError` if initialization fails.
    #[inline]
    pub fn try_new(repository: R) -> Result<Self, TemplateError> {
        let mut env = Environment::new();
        env.set_undefined_behavior(UndefinedBehavior::Strict);
        env.set_recursion_limit(10);
        env.set_auto_escape_callback(|_| AutoEscape::None);

        FilterRegistry::register_all(&mut env);

        Ok(Self {
            env: Arc::new(env),
            repository,
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
        let templates = self.repository.list_templates()?;

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
            let source = Emitter::emit(template);

            // Leak strings to meet Environment<'static> requirement.
            // Templates are loaded once at startup and are permanent.
            let name_static: &'static str =
                Box::leak(template.name().as_str().to_owned().into_boxed_str());
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
    /// - Render: Undefined input, filter validation failed, or other render
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
        let templates = self.repository.list_templates()?;
        Ok(templates.into_iter().map(|t| t.name().to_string()).collect())
    }

    /// Topologically sorts templates by extends relationships (Kahn's
    /// algorithm).
    ///
    /// # Errors
    /// Returns `CircularComposition` if a cycle is detected.
    fn topological_sort(
        templates: &[Template],
    ) -> Result<Vec<&Template>, TemplateError> {
        let mut graph: HashMap<&TemplateName, Vec<&TemplateName>> =
            HashMap::new();
        let mut in_degree: HashMap<&TemplateName, usize> = HashMap::new();
        let mut template_map: HashMap<&TemplateName, &Template> =
            HashMap::new();

        for template in templates {
            template_map.insert(template.name(), template);
            in_degree.entry(template.name()).or_insert(0);

            if let Some(parent) = template.extends() {
                graph.entry(parent).or_default().push(template.name());
                #[expect(
                    clippy::arithmetic_side_effects,
                    reason = "Template count is finite"
                )]
                {
                    *in_degree.entry(template.name()).or_insert(0) += 1;
                }
            }
        }

        let mut queue: VecDeque<&TemplateName> = in_degree
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

            for child in children {
                let deg = in_degree.get_mut(child).ok_or_else(|| {
                    TemplateError::Storage(format!(
                        "Child {child} not found in degrees"
                    ))
                })?;

                #[expect(
                    clippy::arithmetic_side_effects,
                    reason = "In-degree is positive"
                )]
                {
                    *deg -= 1;
                };

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
        BlockStrategy, TemplateBlock, repository::WriteRepository,
        storage::testing::InMemoryRepository,
    };

    #[test]

    fn loads_and_compiles_all_templates() {
        let storage = InMemoryRepository::new();

        // Create parent template
        let parent_name = TemplateName::try_from("parent").unwrap();
        let parent = Template::try_new(
            &parent_name,
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
        let child_name = TemplateName::try_from("child").unwrap();
        let parent_name_ext = TemplateName::try_from("parent").unwrap();
        let child = Template::try_new(
            &child_name,
            Some(parent_name_ext),
            vec![TemplateBlock::new(
                "title",
                "Custom Title",
                BlockStrategy::Replace,
            )],
            HashMap::new(),
        )
        .unwrap();

        storage.save_template(&parent).unwrap();
        storage.save_template(&child).unwrap();

        // Load all into catalog (automatic topological sort)
        let mut catalog = TemplateCatalog::try_new(storage).unwrap();
        catalog.load_all().unwrap();

        // Render child template
        let output = catalog.render("child", minijinja::context! {}).unwrap();
        assert!(output.contains("Custom Title"));
        assert!(!output.contains("Default Title"));
    }

    #[test]

    fn detects_circular_extends() {
        let storage = InMemoryRepository::new();

        // Create circular dependency: A extends B, B extends A
        let a_name = TemplateName::try_from("a").unwrap();
        let b_name = TemplateName::try_from("b").unwrap();
        let a = Template::try_new(
            &a_name,
            Some(TemplateName::try_from("b").unwrap()),
            vec![],
            HashMap::new(),
        )
        .unwrap();

        let b = Template::try_new(
            &b_name,
            Some(TemplateName::try_from("a").unwrap()),
            vec![],
            HashMap::new(),
        )
        .unwrap();

        storage.save_template(&a).unwrap();
        storage.save_template(&b).unwrap();

        // Load should fail with cycle detection error
        let mut catalog = TemplateCatalog::try_new(storage).unwrap();
        catalog.load_all().unwrap_err();
    }

    #[test]

    fn topological_sort_compiles_parents_before_children() {
        let storage = InMemoryRepository::new();

        // Create 3-level hierarchy: grandparent <- parent <- child
        let gp_name = TemplateName::try_from("grandparent").unwrap();
        let grandparent = Template::try_new(
            &gp_name,
            None,
            vec![TemplateBlock::new(
                "a",
                "Grandparent",
                BlockStrategy::Replace,
            )],
            HashMap::new(),
        )
        .unwrap();

        let p_name = TemplateName::try_from("parent").unwrap();
        let parent = Template::try_new(
            &p_name,
            Some(TemplateName::try_from("grandparent").unwrap()),
            vec![TemplateBlock::new("a", "Parent", BlockStrategy::Replace)],
            HashMap::new(),
        )
        .unwrap();

        let c_name = TemplateName::try_from("child").unwrap();
        let child = Template::try_new(
            &c_name,
            Some(TemplateName::try_from("parent").unwrap()),
            vec![TemplateBlock::new("a", "Child", BlockStrategy::Replace)],
            HashMap::new(),
        )
        .unwrap();

        // Store in WRONG order (child first)
        storage.save_template(&child).unwrap();
        storage.save_template(&grandparent).unwrap();
        storage.save_template(&parent).unwrap();

        // Catalog should sort them correctly
        let mut catalog = TemplateCatalog::try_new(storage).unwrap();
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

    fn list_names_returns_all_templates() {
        let storage = InMemoryRepository::new();

        let t1_name = TemplateName::try_from("t1").unwrap();
        let t1 =
            Template::try_new(&t1_name, None, vec![], HashMap::new()).unwrap();
        let t2_name = TemplateName::try_from("t2").unwrap();
        let t2 =
            Template::try_new(&t2_name, None, vec![], HashMap::new()).unwrap();

        storage.save_template(&t1).unwrap();
        storage.save_template(&t2).unwrap();

        let catalog = TemplateCatalog::try_new(storage).unwrap();
        let names = catalog.list_names().unwrap();

        assert_eq!(names.len(), 2);
        assert!(names.contains(&"t1".to_owned()));
        assert!(names.contains(&"t2".to_owned()));
    }
}
