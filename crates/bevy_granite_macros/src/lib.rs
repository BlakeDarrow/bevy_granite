use once_cell::sync::Lazy;
use proc_macro::TokenStream;
use quote::quote;
use std::sync::Mutex;
use syn::{parse_macro_input, DeriveInput,};

static REGISTERED_COMPONENTS: Lazy<Mutex<Vec<(String, bool)>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

use std::sync::atomic::{AtomicBool, Ordering};

static IMPORTS_ADDED: AtomicBool = AtomicBool::new(false);

#[proc_macro_attribute]
pub fn granite_component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();
    println!("MACRO: Registering component: {}", name_str);
    let attr_str = attr.to_string();
    let include_default = attr_str.contains("default");
    let is_hidden = attr_str.contains("ui_hidden");
    REGISTERED_COMPONENTS
        .lock()
        .unwrap()
        .push((name_str.clone(), is_hidden));
    let derives = if include_default {
        quote! {
            #[derive(Reflect, Serialize, Deserialize, Debug, Clone, Component, PartialEq)]
        }
    } else {
        quote! {
            #[derive(Reflect, Serialize, Deserialize, Debug, Clone, Component, Default, PartialEq)]
        }
    };
    // Only add imports on the first use
    let needs_imports = !IMPORTS_ADDED.swap(true, Ordering::Relaxed);
    let imports = if needs_imports {
        quote! {
            #[warn(unused_imports)]
            use bevy::prelude::{Component,ReflectFromReflect, ReflectDefault, ReflectDeserialize, ReflectSerialize, ReflectComponent};
            #[warn(unused_imports)]
            use bevy::reflect::{Reflect, FromReflect};
            #[warn(unused_imports)]
            use serde::{Serialize, Deserialize};
        }
    } else {
        quote! {}
    };
    let expanded = quote! {
        #imports
        #derives
        #[reflect(Component, Serialize, Deserialize, Default, FromReflect)]
        #input
    };
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn register_editor_components(input: TokenStream) -> TokenStream {
    let app_name = if input.is_empty() {
        quote!(app)
    } else {
        let parsed = parse_macro_input!(input as syn::Ident);
        quote!(#parsed)
    };

    let components = REGISTERED_COMPONENTS.lock().unwrap();
    let tokens = components.iter().map(|(name, is_hidden)| {
        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());

        if *is_hidden {
            quote! {
                #app_name.register_type::<#ident>();
            }
        } else {
            quote! {
                #app_name.register_type::<#ident>();
                #app_name.register_type_data::<#ident, bevy_granite::prelude::BridgeTag>();
            }
        }
    });

    let expanded = quote! {
        {
            #(#tokens)*
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn ui_callable_events(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();

    // Extract field names and types from the struct
    let (field_names, field_types): (Vec<_>, Vec<_>) = if let syn::Data::Struct(ref data_struct) = input.data {
        if let syn::Fields::Named(ref fields_named) = data_struct.fields {
            fields_named.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap().to_string();
                let field_type = &field.ty;
                (field_name, field_type.clone())
            }).unzip()
        } else {
            (Vec::new(), Vec::new())
        }
    } else {
        (Vec::new(), Vec::new())
    };

    // Generate event sender closures
    let event_senders = field_types.iter().map(|field_type| {
        quote! {
            Box::new(|world: &mut bevy::prelude::World| {
                world.send_event(#field_type::default());
            }) as Box<dyn Fn(&mut bevy::prelude::World) + Send + Sync>
        }
    });

    let expanded = quote! {
        #input
        
        impl bevy_granite_core::UICallableEventMarker for #name {}
        
        impl bevy_granite_core::UICallableEventProvider for #name {
            fn get_event_names() -> &'static [&'static str] {
                &[#(#field_names),*]
            }
            
            fn get_struct_name() -> &'static str {
                #name_str
            }
        }
        
        impl #name {
            pub fn get_event_types() -> &'static [&'static str] {
                &[#(stringify!(#field_types)),*]
            }
            
            pub fn register_ui() {
                let event_senders = vec![#(#event_senders),*];
                let event_names: &'static [&'static str] = &[#(#field_names),*];
                
                // Use the registration function - this will be provided by the user's import
                bevy_granite::prelude::register_ui_callable_events_with_senders(
                    #name_str,
                    event_names,
                    event_senders,
                );
            }
        }
    };
    
    TokenStream::from(expanded)
}

