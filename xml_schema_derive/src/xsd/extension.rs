use crate::xsd::{
  attribute::Attribute, rust_types_mapping::RustTypesMapping, sequence::Sequence, Implementation,
  XsdContext,
};
use proc_macro2::TokenStream;

#[derive(Clone, Default, Debug, PartialEq, YaDeserialize)]
#[yaserde(
  root = "extension",
  prefix = "xs",
  namespace = "xs: http://www.w3.org/2001/XMLSchema"
)]
pub struct Extension {
  #[yaserde(attribute)]
  pub base: String,
  #[yaserde(rename = "attribute")]
  pub attributes: Vec<Attribute>,
  #[yaserde(rename = "sequence")]
  pub sequences: Vec<Sequence>,
}

impl Implementation for Extension {
  fn implement(
    &self,
    namespace_definition: &TokenStream,
    prefix: &Option<String>,
    context: &XsdContext,

    sub_types_name_prefix: &Option<&str>,
  ) -> TokenStream {
    let rust_type = RustTypesMapping::get(context, &self.base);

    let attributes: TokenStream = self
      .attributes
      .iter()
      .map(|attribute| attribute.implement(namespace_definition, prefix, context, sub_types_name_prefix))
      .collect();

    let inner_attribute = if format!("{rust_type}") == "String" {
      quote!(#[yaserde(text)])
    } else {
      TokenStream::new()
    };

    quote!(
      #inner_attribute
      pub content: #rust_type,
      #attributes
    )
  }
}

impl Extension {
  pub fn get_field_implementation(
    &self,
    context: &XsdContext,
    _prefix: &Option<String>,
  ) -> TokenStream {
    let rust_type = RustTypesMapping::get(context, &self.base);
    quote!(pub content : #rust_type)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::str::FromStr;

  #[test]
  fn extension() {
    let st = Extension {
      base: "xs:string".to_string(),
      attributes: vec![],
      sequences: vec![],
    };

    let context =
      XsdContext::new(r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"></xs:schema>"#)
        .unwrap();

    let implementation = st.implement(&TokenStream::new(), &None, &context, &None);

    let expected = TokenStream::from_str(
      r#"
        #[yaserde(text)]
        pub content: String,
      "#,
    )
    .unwrap();

    assert_eq!(implementation.to_string(), expected.to_string());
  }

  #[test]
  fn extension_with_attributes() {
    use crate::xsd::attribute::Required;

    let st = Extension {
      base: "xs:string".to_string(),
      attributes: vec![
        Attribute {
          name: Some("attribute_1".to_string()),
          kind: Some("xs:string".to_string()),
          reference: None,
          required: Required::Required,
          simple_type: None,
        },
        Attribute {
          name: Some("attribute_2".to_string()),
          kind: Some("xs:boolean".to_string()),
          reference: None,
          required: Required::Optional,
          simple_type: None,
        },
      ],
      sequences: vec![],
    };

    let context =
      XsdContext::new(r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"></xs:schema>"#)
        .unwrap();

    let implementation = st.implement(&TokenStream::new(), &None, &context, &None);

    let expected = TokenStream::from_str(
      r#"
        #[yaserde(text)]
        pub content: String,
        #[yaserde(attribute)]
        pub attribute_1: String,
        #[yaserde(attribute)]
        pub attribute_2: Option<bool> ,
      "#,
    )
    .unwrap();

    assert_eq!(implementation.to_string(), expected.to_string());
  }
}
