//! # neuralbev-macros
//!
//! Proc-macro crate для NeuralBev.  
//! Экспортирует единственный макрос `#[neuralbev_component]`.
//!
//! ## AI-HINT
//! Этот крейт содержит только proc-макросы. **Не добавляй сюда runtime-код.**
//! Если нужно добавить новый макрос — создай отдельную функцию вида `pub fn my_macro_impl`.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

// ──────────────────────────────────────────────────────────────────────────────
// MACRO: neuralbev_component
// PURPOSE: Автоматически добавляет Bevy-трейты к компоненту и регистрирует его
//          в TypeRegistry для редактора и сериализации.
// USAGE:   #[neuralbev_component] перед pub struct MyComp { ... }
// AI-HINT: Применяй ко ВСЕМ компонентам проекта — это заменяет ручной derive.
// ──────────────────────────────────────────────────────────────────────────────

/// WHAT: Процедурный макрос для объявления NeuralBev-компонентов.  
/// WHY:  Гарантирует, что каждый компонент имеет нужные трейты и попадает
///       в TypeRegistry (необходимо для Inspector и сериализации сцен).  
/// HOW:  ```rust
///       #[neuralbev_component]
///       pub struct Health {
///           pub current: f32,
///           pub max_health: f32,
///       }
///       ```
/// AI-HINT: Не дублируй `#[derive(Component, Reflect)]` вручную —
///           этот макрос уже добавляет их.
#[proc_macro_attribute]
pub fn neuralbev_component(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();

    // Генерируем блок: добавляем derive + impl ComponentMeta
    let expanded = quote! {
        // Автодобавление всех нужных трейтов.
        #[derive(
            bevy::prelude::Component,
            bevy::reflect::Reflect,
            Default,
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
        )]
        // Регистрация компонента через Reflect.
        #[reflect(Component, Default)]
        #input

        // Реализуем вспомогательный трейт, который сообщает имя компонента
        // редактору и скриптовому слою.
        impl crate::components::ComponentMeta for #name {
            /// Возвращает строковое имя компонента (используется в Inspector и YAML).
            fn component_name() -> &'static str {
                #name_str
            }
        }
    };

    TokenStream::from(expanded)
}
