use crate::xsd::{
  attribute, attribute_group, complex_type, element, import, qualification, simple_type,
  Implementation, XsdContext,
};
use proc_macro2::TokenStream;

#[derive(Clone, Default, Debug, PartialEq, YaDeserialize)]
#[yaserde(
  root="schema"
  prefix="xs",
  namespace="xs: http://www.w3.org/2001/XMLSchema",
)]
pub struct Schema {
  #[yaserde(rename = "targetNamespace", attribute)]
  pub target_namespace: Option<String>,
  #[yaserde(rename = "elementFormDefault", attribute)]
  pub element_form_default: qualification::Qualification,
  #[yaserde(rename = "attributeFormDefault", attribute)]
  pub attribute_form_default: qualification::Qualification,
  #[yaserde(rename = "import")]
  pub imports: Vec<import::Import>,
  #[yaserde(rename = "element")]
  pub elements: Vec<element::Element>,
  #[yaserde(rename = "simpleType")]
  pub simple_type: Vec<simple_type::SimpleType>,
  #[yaserde(rename = "complexType")]
  pub complex_type: Vec<complex_type::ComplexType>,
  #[yaserde(rename = "attribute")]
  pub attributes: Vec<attribute::Attribute>,
  #[yaserde(rename = "attributeGroup")]
  pub attribute_group: Vec<attribute_group::AttributeGroup>,
}

impl Implementation for Schema {
  fn implement(
    &self,
    _namespace_definition: &TokenStream,
    target_prefix: &Option<String>,
    context: &XsdContext,

    sub_types_name_prefix: &Option<&str>,
  ) -> TokenStream {
    let namespace_definition = generate_namespace_definition(target_prefix, &self.target_namespace);

    log::info!("Generate elements");
    let elements: TokenStream = self
      .elements
      .iter()
      .map(|element| element.implement(&namespace_definition, target_prefix, context, sub_types_name_prefix))
      .collect();

    log::info!("Generate simple types");
    for simple_type in self.simple_type.iter() {
      log::debug!("Simple type {name}", name = simple_type.name);
    }
    let simple_types: TokenStream = self
      .simple_type
      .iter()
      .map(|simple_type| simple_type.implement(&namespace_definition, target_prefix, context, sub_types_name_prefix))
      .collect();

    log::info!("Generate complex types");
    for complex_type in self.complex_type.iter() {
      log::debug!("Complex type {name}", name = complex_type.name);
    }

    let complex_types: TokenStream = self
      .complex_type
      .iter()
      .map(|complex_type| complex_type.implement(&namespace_definition, target_prefix, context, sub_types_name_prefix))
      .collect();

    quote!(
      pub mod types {
        #simple_types
        #complex_types
      }

      #elements
    )
  }
}

fn generate_namespace_definition(
  target_prefix: &Option<String>,
  target_namespace: &Option<String>,
) -> TokenStream {
  match (target_prefix, target_namespace) {
    (None, None) => quote!(),
    (None, Some(_target_namespace)) => {
      panic!("undefined prefix attribute, a target namespace is defined")
    }
    (Some(_prefix), None) => panic!(
      "a prefix attribute, but no target namespace is defined, please remove the prefix parameter"
    ),
    (Some(prefix), Some(target_namespace)) => {
      let namespace = format!("{prefix}: {target_namespace}");
      quote!(#[yaserde(prefix=#prefix, namespace=#namespace)])
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_schema_implementation() {
    let schema = Schema::default();

    let context =
      XsdContext::new(r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"></xs:schema>"#)
        .unwrap();

    let implementation = format!("{}", schema.implement(&TokenStream::new(), &None, &context, &None));
    assert_eq!(implementation, "pub mod types { }");
  }

  #[test]
  #[should_panic]
  fn missing_prefix() {
    let schema = Schema {
      target_namespace: Some("http://example.com".to_string()),
      ..Default::default()
    };

    let context =
      XsdContext::new(r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"></xs:schema>"#)
        .unwrap();

    schema.implement(&TokenStream::new(), &None, &context, &None);
  }

  #[test]
  #[should_panic]
  fn missing_target_namespace() {
    let schema = Schema::default();

    let context =
      XsdContext::new(r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"></xs:schema>"#)
        .unwrap();

    schema.implement(&TokenStream::new(), &Some("ex".to_string()), &context, &None);
  }

  #[test]
  fn generate_namespace() {
    let definition = generate_namespace_definition(
      &Some("prefix".to_string()),
      &Some("http://example.com".to_string()),
    );

    let implementation = format!("{definition}");

    assert_eq!(
      implementation,
      r#"# [yaserde (prefix = "prefix" , namespace = "prefix: http://example.com")]"#
    );
  }
}
