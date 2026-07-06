#![allow(
    dead_code,
    reason = "Shared integration test fixtures are not used by every test"
)]

use std::sync::Arc;

use traces_db::Store;
pub use traces_db::testing::TestStore;
use traces_schema::{
    aggregate::Schema,
    identifier::SchemaName,
    property::{Multiplicity, Optionality, Property, PropertyId, PropertyName},
    property_spec::{BoolSpec, PropertySpec, StringSpec},
    repository::ReadRepository,
    storage::RedbRepository,
};

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
pub type NamedProperty = (PropertyName, Property);

pub trait RepositoryExt {
    fn find_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<Schema>, Box<dyn std::error::Error>>;
}

impl<R> RepositoryExt for R
where
    R: ReadRepository,
{
    fn find_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<Schema>, Box<dyn std::error::Error>> {
        let Some(id) = self.find_schema_id_by_name(name)? else {
            return Ok(None);
        };
        self.find_schema_by_id(id).map_err(Into::into)
    }
}

pub fn setup_repository(store: &Arc<Store>) -> RedbRepository {
    RedbRepository::new(Arc::clone(store))
}

#[derive(Debug)]
pub struct PropertyBuilder {
    name: String,
    optionality: Optionality,
    multiplicity: Multiplicity,
    id: Option<PropertyId>,
}

impl PropertyBuilder {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            optionality: Optionality::default(),
            multiplicity: Multiplicity::default(),
            id: None,
        }
    }

    pub const fn optionality(mut self, optionality: Optionality) -> Self {
        self.optionality = optionality;
        self
    }

    pub const fn multiplicity(mut self, multiplicity: Multiplicity) -> Self {
        self.multiplicity = multiplicity;
        self
    }

    pub const fn id(mut self, id: PropertyId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn build_bool(self) -> TestResult<NamedProperty> {
        self.build_with_spec(PropertySpec::Bool(BoolSpec::default()))
    }

    pub fn build_string_default(self) -> TestResult<NamedProperty> {
        self.build_with_spec(PropertySpec::String(StringSpec::default()))
    }

    pub fn build_with_spec(
        self,
        spec: PropertySpec,
    ) -> TestResult<NamedProperty> {
        let name = PropertyName::try_new(&self.name)?;
        let id = self.id.unwrap_or_default();
        let property =
            Property::new(id, self.optionality, self.multiplicity, spec);
        Ok((name, property))
    }
}

pub fn bool_property(name: &str) -> TestResult<NamedProperty> {
    PropertyBuilder::new(name).build_bool()
}

pub fn string_property(name: &str) -> TestResult<NamedProperty> {
    PropertyBuilder::new(name).build_string_default()
}
