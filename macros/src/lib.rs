use std::{collections::HashMap, fs};

use convert_case::{Case, Casing};
use cream_core::{
    Attribute, Mutability, ResourceType, Returned, Schema, SchemaExtension, Type, Uniqueness,
};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use serde::de::DeserializeOwned;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Bracket,
    Ident, Token,
};

#[allow(unused)]
struct DeclareResource {
    path: String,
    as_: Token![as],
    name: Ident,
    bracket_token: Bracket,
    schemas: Punctuated<ReferencedSchema, Token![,]>,
}

struct ReferencedSchema {
    path: String,
}

impl Parse for DeclareResource {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            path: input.parse::<syn::LitStr>()?.value(),
            as_: input.parse::<Token![as]>()?,
            name: input.parse::<Ident>()?,
            bracket_token: bracketed!(content in input),
            schemas: Punctuated::parse_terminated(&content)?,
        })
    }
}

impl Parse for ReferencedSchema {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            path: input.parse::<syn::LitStr>()?.value(),
        })
    }
}

fn load_static_resource<T: DeserializeOwned>(path: &str) -> (T, String) {
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap_or(".".into());
    let path = std::path::Path::new(&root).join(path);
    let content = fs::read_to_string(&path).expect("File not found");
    (
        serde_json::from_str(&content).expect("Failed to parse JSON"),
        content,
    )
}

struct SchemaStruct {
    declaration: TokenStream2,
    ty: Ident,
    create_ty: Ident,
}

const KEYWORDS: &[&str] = &["ref", "type"];

fn sanitize_name(name: &str, casing: Case) -> Ident {
    let converted = name.replace("$", "").to_case(casing);
    if KEYWORDS.contains(&converted.as_str()) {
        format_ident!("{}_", converted)
    } else {
        format_ident!("{}", converted)
    }
}

fn declare_manager_trait(
    manager: Ident,
    ty: Ident,
    create_ty: Ident,
    resource_type_str: &str,
    schemas: &HashMap<String, (Schema, String)>,
) -> TokenStream2 {
    let adapter = format_ident!("{}Adapter", manager);
    let schema_arms = schemas.iter().map(|(schema_id, (_, schema_str))| {
        quote! {
            #schema_id => {
                ::serde_json::from_str(#schema_str).expect(concat!("Failed to deserialize ", #schema_id))
            }
        }
    }).collect::<Vec<_>>();
    quote! {
        #[::cream::hidden::axum::async_trait]
        pub trait #manager: ::std::fmt::Debug + Send + Sync + 'static {
            async fn list(
                &self,
                args: ::cream::ListResourceArgs<'async_trait>,
            ) -> Result<::cream::ListResourceResult<#ty>, ::cream::Error>;
            async fn get(&self, args: ::cream::GetResourceArgs<'async_trait>) -> Result<#ty, ::cream::Error>;
            async fn create(&self, resource: #create_ty) -> Result<String, ::cream::Error>;
            async fn update(&self, args: ::cream::UpdateResourceArgs<'async_trait>) -> Result<(), ::cream::Error>;
            async fn replace(&self, id: &'async_trait str, resource: #create_ty) -> Result<(), ::cream::Error>;
            async fn delete(&self, id: &'async_trait str) -> Result<(), ::cream::Error>;

            fn default_page_size(&self) -> usize {
                50
            }
        }

        #[derive(Debug)]
        pub struct #adapter<T: #manager>(T);

        #[::cream::hidden::axum::async_trait]
        impl<T: #manager> ::cream::GenericResourceManager for #adapter<T> {
            async fn list(
                &self,
                args: ::cream::ListResourceArgs<'async_trait>,
            ) -> Result<::cream::ListResourceResult<::cream::hidden::ijson::IObject>, ::cream::Error> {
                let result = self.0.list(args).await?;
                Ok(::cream::ListResourceResult {
                    resources: result.resources.into_iter().map(|mut resource| {
                        resource.locate();
                        resource.to_object()
                    }).collect(),
                    total_count: result.total_count,
                    items_per_page: result.items_per_page,
                })
            }

            async fn get(&self, args: ::cream::GetResourceArgs<'async_trait>) -> Result<::cream::hidden::ijson::IObject, ::cream::Error> {
                let mut resource = self.0.get(args).await?;
                resource.locate();
                Ok(resource.to_object())
            }

            async fn create(&self, resource: ::cream::hidden::ijson::IObject) -> Result<String, ::cream::Error> {
                let create_resource = #create_ty::from_object(&resource)?;
                self.0.create(create_resource).await
            }

            async fn update(&self, args: ::cream::UpdateResourceArgs<'async_trait>) -> Result<(), ::cream::Error> {
                self.0.update(args).await
            }

            async fn replace(&self, id: &str, resource: ::cream::hidden::ijson::IObject) -> Result<(), ::cream::Error> {
                let create_resource = #create_ty::from_object(&resource)?;
                self.0.replace(id, create_resource).await
            }

            async fn delete(&self, id: &str) -> Result<(), ::cream::Error> {
                self.0.delete(id).await
            }
            fn default_page_size(&self) -> usize {
                self.0.default_page_size()
            }

            fn load_resource_type(&self) -> ::cream::ResourceType {
                ::serde_json::from_str(#resource_type_str).expect(concat!("Failed to deserialize resource type"))
            }

            fn load_schema(&self, id: &str) -> ::cream::Schema {
                match id {
                    #(#schema_arms)*
                    _ => panic!("Unknown schema: {}", id),
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn declare_schema_struct(
    struct_name: Ident,
    attributes: &[Attribute],
    schema_urn: TokenStream2,
    parent_attr_name: Option<&str>,
    manager: Option<Ident>,
    extensions: &[SchemaExtension],
    schemas: &HashMap<String, (Schema, String)>,
    core_resource_type: Option<&ResourceType>,
) -> SchemaStruct {
    let mut fields = Vec::new();
    let mut create_fields = Vec::new();
    let mut other_declarations = Vec::new();
    let mut field_consts = Vec::new();

    let id_attribute = Attribute {
        name: "id".into(),
        type_: Type::String,
        multi_valued: false,
        description: "Unique identifier for the resource".into(),
        required: false,
        canonical_values: None,
        case_exact: true,
        mutability: Mutability::ReadOnly,
        returned: Returned::Always,
        uniqueness: Uniqueness::Server,
        reference_types: None,
        sub_attributes: Vec::new(),
    };

    let external_id_attribute = Attribute {
        name: "externalId".into(),
        type_: Type::String,
        multi_valued: false,
        description: "External identifier for the resource".into(),
        required: false,
        canonical_values: None,
        case_exact: true,
        mutability: Mutability::ReadWrite,
        returned: Returned::Default,
        uniqueness: Uniqueness::None,
        reference_types: None,
        sub_attributes: Vec::new(),
    };

    let extra_attributes = if core_resource_type.is_some() {
        vec![id_attribute, external_id_attribute]
    } else {
        Vec::new()
    };

    for attr in extra_attributes.iter().chain(attributes) {
        let name = sanitize_name(&attr.name, Case::Snake);
        let upper_name = sanitize_name(&attr.name, Case::UpperSnake);
        let pascal_name = sanitize_name(&attr.name, Case::Pascal);
        let attr_name = &attr.name;

        let (mut ty, mut create_ty) = match attr.type_ {
            Type::String | Type::Binary => (quote! { String }, quote! { String }),
            Type::Boolean => (quote! { bool }, quote! { bool }),
            Type::Decimal => (quote! { f64 }, quote! { f64 }),
            Type::Integer => (quote! { i64 }, quote! { i64 }),
            Type::DateTime => (quote! { ::cream::DateTime }, quote! { ::cream::DateTime }),
            Type::Reference => (quote! { ::cream::Reference }, quote! { ::cream::Reference }),
            Type::Complex => {
                let singular_name = if attr.multi_valued {
                    if let Some(prefix) = pascal_name.to_string().strip_suffix("ses") {
                        format_ident!("{}s", prefix)
                    } else if let Some(prefix) = pascal_name.to_string().strip_suffix("s") {
                        format_ident!("{}", prefix)
                    } else {
                        pascal_name
                    }
                } else {
                    pascal_name
                };
                let SchemaStruct {
                    declaration,
                    ty,
                    create_ty,
                } = declare_schema_struct(
                    format_ident!("{}{}", struct_name, singular_name),
                    &attr.sub_attributes,
                    schema_urn.clone(),
                    Some(attr_name),
                    None,
                    &[],
                    schemas,
                    None,
                );
                other_declarations.push(declaration);
                (quote! { #ty }, quote! { #create_ty })
            }
        };

        // Define constants for referencing fields
        if let Some(parent_attr_name) = parent_attr_name {
            field_consts.push(quote! {
                pub const #upper_name: ::cream::AttrPathRef<'static> = ::cream::AttrPathRef {
                    urn: #schema_urn,
                    name: #parent_attr_name,
                    sub_attr: Some(#attr_name),
                };
            });
        } else {
            field_consts.push(quote! {
                pub const #upper_name: ::cream::AttrPathRef<'static> = ::cream::AttrPathRef {
                    urn: #schema_urn,
                    name: #attr_name,
                    sub_attr: None,
                };
            });
        }

        // "read" type
        let is_present = !matches!(attr.returned, Returned::Never)
            && !matches!(attr.mutability, Mutability::WriteOnly);

        if is_present {
            let is_optional = matches!(attr.returned, Returned::Default | Returned::Request);
            let mut serde_attrs = Vec::new();
            serde_attrs.push(quote! { rename = #attr_name });

            if attr.multi_valued {
                ty = quote! { Vec<#ty> };
            }

            if is_optional {
                serde_attrs.push(quote! { skip_serializing_if = "Option::is_none" });
                ty = quote! { Option<#ty> };
            }

            fields.push(quote! {
                #[serde( #(#serde_attrs),* )]
                pub #name: #ty,
            });
        }

        // "create" type
        let create_is_present = !matches!(attr.mutability, Mutability::ReadOnly);

        if create_is_present {
            let create_is_optional = !attr.required;
            let mut serde_attrs = Vec::new();
            serde_attrs.push(quote! { rename = #attr_name });

            if attr.multi_valued {
                create_ty = quote! { Vec<#create_ty> };
            }

            if create_is_optional {
                if attr.multi_valued {
                    serde_attrs.push(quote! { default });
                } else {
                    create_ty = quote! { Option<#create_ty> };
                }
            }

            create_fields.push(quote! {
                #[serde( #(#serde_attrs),* )]
                pub #name: #create_ty,
            });
        }
    }

    for (i, ext) in extensions.iter().enumerate() {
        let name = format_ident!("ext{}", i);
        let schema_id = &ext.schema;
        let SchemaStruct {
            declaration,
            ty,
            create_ty,
        } = declare_schema_struct(
            format_ident!("{}Ext{}", struct_name, i),
            &schemas[&ext.schema].0.attributes,
            quote! {Some(#schema_id)},
            None,
            None,
            &[],
            schemas,
            None,
        );
        let ty = quote! { #ty };
        let mut create_ty = quote! { #create_ty };

        other_declarations.push(declaration);
        let mut serde_attrs = Vec::new();
        serde_attrs.push(quote! { rename = #schema_id });

        fields.push(quote! {
            #[serde( #(#serde_attrs),* )]
            pub #name: #ty,
        });

        let mut serde_attrs = Vec::new();
        serde_attrs.push(quote! { rename = #schema_id });
        if !ext.required {
            create_ty = quote! { Option<#create_ty> };
            serde_attrs.push(quote! { default });
        }

        create_fields.push(quote! {
            #[serde( #(#serde_attrs),* )]
            pub #name: #create_ty,
        });
    }

    let create_struct_name = format_ident!("Create{}", struct_name);

    let mut other_methods = Vec::new();
    if let Some(manager) = manager {
        let adapter = format_ident!("{}Adapter", manager);
        other_methods.push(quote! {
            pub fn manage(manager: impl #manager) -> impl ::cream::GenericResourceManager {
                #adapter(manager)
            }
        });
    };

    if let Some(resource_type) = core_resource_type {
        let resource_name = &resource_type.name;
        let endpoint = &resource_type.endpoint;
        let schema_type_name = format_ident!("{}Schema", struct_name);
        let resource_type_name = format_ident!("{}ResourceType", struct_name);
        other_declarations.push(quote! {
            ::cream::declare_schema!(#schema_type_name = "urn:ietf:params:scim:schemas:core:2.0:Schema");
            ::cream::declare_resource_type!(#resource_type_name = #resource_name);
        });

        other_methods.push(quote! {
            pub fn locate(&mut self) {
                self.meta.location = Some(::cream::Reference::new_relative(&format!(
                    "{}/{}",
                    #endpoint,
                    self.id
                )));
            }
        });

        fields.push(quote! {
            pub meta: ::cream::Meta<#resource_type_name>,
        })
    }

    let declaration = quote! {
        #(#other_declarations)*

        #[derive(Debug, ::cream::hidden::serde::Serialize, Clone)]
        pub struct #struct_name {
            #(
                #fields
            )*
        }

        impl #struct_name {
            #(
                #field_consts
            )*

            #(
                #other_methods
            )*

            pub fn to_object(&self) -> ::cream::hidden::ijson::IObject {
                ::cream::hidden::ijson::to_value(self)
                    .expect("Infallible serialization")
                    .into_object()
                    .expect("Resources must serialize as objects")
            }
        }

        #[derive(Debug, ::cream::hidden::serde::Deserialize, Clone)]
        pub struct #create_struct_name {
            #(
                #create_fields
            )*
        }

        impl #create_struct_name {
            pub fn from_object(object: &::cream::hidden::ijson::IObject) -> Result<Self, ::cream::Error> {
                ::cream::hidden::ijson::from_value(object.as_ref()).map_err(|e| ::cream::Error {
                    status: ::cream::hidden::axum::http::StatusCode::BAD_REQUEST,
                    schemas: Default::default(),
                    scim_type: Some(::cream::ErrorType::InvalidValue),
                    detail: e.to_string(),
                })
            }
        }
    };
    SchemaStruct {
        ty: struct_name,
        create_ty: create_struct_name,
        declaration,
    }
}

#[proc_macro]
pub fn declare_resource(input: TokenStream) -> TokenStream {
    let DeclareResource {
        path,
        name,
        schemas: ref_schemas,
        ..
    } = parse_macro_input!(input as DeclareResource);

    // Load the resource type and referenced schemas
    let (resource_type, _resource_type_str) = load_static_resource::<ResourceType>(&path);
    let mut schemas = HashMap::new();
    for ref_schema in ref_schemas {
        let (schema, schema_str) = load_static_resource::<Schema>(&ref_schema.path);
        schemas.insert(schema.id.clone(), (schema, schema_str));
    }

    let manager = format_ident!("{}Manager", name);

    let SchemaStruct {
        ty,
        create_ty,
        declaration,
    } = declare_schema_struct(
        name.clone(),
        &schemas[&resource_type.schema].0.attributes,
        quote! { None },
        None,
        Some(manager.clone()),
        &resource_type.schema_extensions,
        &schemas,
        Some(&resource_type),
    );

    let mut result = TokenStream2::new();
    result.extend(declaration);
    result.extend(declare_manager_trait(
        manager,
        ty,
        create_ty,
        &serde_json::to_string(&resource_type).unwrap(),
        &schemas,
    ));
    result.into()
}
