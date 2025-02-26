use crate::xsd::{
  annotation::Annotation, attribute::Attribute, complex_content::ComplexContent,
  sequence::Sequence, simple_content::SimpleContent, Implementation, XsdContext,
};
use heck::CamelCase;
use proc_macro2::{Span, TokenStream};
use syn::Ident;

#[derive(Clone, Default, Debug, PartialEq, YaDeserialize)]
#[yaserde(
  rename = "complexType"
  prefix = "xs",
  namespace = "xs: http://www.w3.org/2001/XMLSchema"
)]
pub struct ComplexType {
  #[yaserde(attribute)]
  pub name: String,
  #[yaserde(rename = "attribute")]
  pub attributes: Vec<Attribute>,
  pub sequence: Option<Sequence>,
  #[yaserde(rename = "simpleContent")]
  pub simple_content: Option<SimpleContent>,
  #[yaserde(rename = "complexContent")]
  pub complex_content: Option<ComplexContent>,
  #[yaserde(rename = "annotation")]
  pub annotation: Option<Annotation>,
}

impl Implementation for ComplexType {
  fn implement(
    &self,
    namespace_definition: &TokenStream,
    prefix: &Option<String>,
    context: &XsdContext,

    sub_types_name_prefix: &Option<&str>,
  ) -> TokenStream {
    let struct_name = Ident::new(
      &self.name.replace('.', "_").to_camel_case(),
      Span::call_site(),
    );

    let my_sub_types_name_prefix = match *sub_types_name_prefix {
      Some(prefix) => prefix.to_owned() + self.name.as_str(),
      None => self.name.to_owned(),
    };

    let sequence = self
      .sequence
      .as_ref()
      .map(|sequence| sequence.implement(namespace_definition, prefix, context, &Some(&my_sub_types_name_prefix)))
      .unwrap_or_else(TokenStream::new);

    let simple_content = self
      .simple_content
      .as_ref()
      .map(|simple_content| simple_content.implement(namespace_definition, prefix, context, sub_types_name_prefix))
      .unwrap_or_else(TokenStream::new);

    let complex_content = self
      .complex_content
      .as_ref()
      .map(|complex_content| {
        let complex_content_type = complex_content.get_field_implementation(context, prefix);
        quote!(
          #[yaserde(flatten)]
          #complex_content_type,
        )
      })
      .unwrap_or_else(TokenStream::new);

    let attributes: TokenStream = self
      .attributes
      .iter()
      .map(|attribute| attribute.implement(namespace_definition, prefix, context, sub_types_name_prefix))
      .collect();

    let sub_types_implementation = self
      .sequence
      .as_ref()
      .map(|sequence| sequence.get_sub_types_implementation(context, namespace_definition, prefix, &Some(&my_sub_types_name_prefix)))
      .unwrap_or_else(TokenStream::new);

    let docs = self
      .annotation
      .as_ref()
      .map(|annotation| annotation.implement(namespace_definition, prefix, context, sub_types_name_prefix))
      .unwrap_or_else(TokenStream::new);

    quote! {
      #docs

      #[derive(Clone, Debug, Default, PartialEq, yaserde_derive::YaDeserialize, yaserde_derive::YaSerialize)]
      #namespace_definition
      pub struct #struct_name {
        #sequence
        #simple_content
        #complex_content
        #attributes
      }

      #sub_types_implementation
    }
  }
}

impl ComplexType {
  pub fn get_field_implementation(
    &self,
    context: &XsdContext,
    prefix: &Option<String>,

    sub_type_name_prefix: &Option<&str>
  ) -> TokenStream {
    if self.sequence.is_some() {
      self
        .sequence
        .as_ref()
        .map(|sequence| sequence.get_field_implementation(context, prefix, sub_type_name_prefix))
        .unwrap_or_else(TokenStream::new)
    } else {
      self
        .simple_content
        .as_ref()
        .map(|simple_content| simple_content.get_field_implementation(context, prefix))
        .unwrap_or_else(TokenStream::new)
    }
  }

  pub fn get_integrated_implementation(
    &self,
    parent_name: &str,

    sub_type_name_prefix: &Option<&str>
  ) -> TokenStream {
    if self.simple_content.is_some() {
      return quote!(String);
    }

    if self.sequence.is_some() {
      let name = match *sub_type_name_prefix {
        Some(name) => name.to_owned() + parent_name,
        None => parent_name.to_owned(),
      };

      log::debug!("Get integrated implementation for {name} - {parent_name} - {sub_type_name_prefix}", name = self.name, sub_type_name_prefix = sub_type_name_prefix.unwrap_or("None"));

      let list_wrapper = Ident::new(&name, Span::call_site());
      return quote!(#list_wrapper);
    }

    quote!(String)
  }
}
