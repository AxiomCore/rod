pub use super::spec::RodSpec;
use crate::core::validator::RodValidator;

pub fn from_yaml(content: &str) -> Result<Box<dyn RodValidator>, serde_yaml::Error> {
    let spec: RodSpec = serde_yaml::from_str(content)?;
    Ok(spec.build())
}

/// The spec of a YAML validator definition without building it.
pub fn spec_from_yaml(content: &str) -> Result<RodSpec, serde_yaml::Error> {
    serde_yaml::from_str(content)
}
