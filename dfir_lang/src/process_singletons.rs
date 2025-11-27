//! Utility methods for processing singleton references: `#my_var`.

use itertools::Itertools;
use proc_macro2::{Group, Ident, TokenStream, TokenTree};
use quote::quote_spanned;
use syn::punctuated::Punctuated;
use syn::{Expr, Token};

use crate::parse::parse_terminated;

/// Finds all the singleton references `#my_var` and appends them to `found_idents`. Returns the
/// `TokenStream` but with the hashes removed from the varnames.
///
/// The returned tokens are used for "preflight" parsing, to check that the rest of the syntax is
/// OK. However the returned tokens are not used in the codegen as we need to use [`postprocess_singletons`]
/// later to substitute-in the context referencing code for each singleton
pub fn preprocess_singletons(tokens: TokenStream, found_idents: &mut Vec<Ident>) -> TokenStream {
    process_singletons(tokens, &mut |singleton_ident| {
        found_idents.push(singleton_ident.clone());
        TokenTree::Ident(singleton_ident)
    })
}

/// Replaces singleton references `#my_var` with the code needed to actually get the value inside.
///
/// * `tokens` - The tokens to update singleton references within.
/// * `resolved_idents` - The context `StateHandle` varnames that correspond 1:1 and in the same
///   order as the singleton references within `tokens` (found in-order via [`preprocess_singletons`]).
///
/// Generates borrowing code ([`std::cell::RefCell::borrow_mut`]). Use
/// [`postprocess_singletons_handles`] for just the `StateHandle`s.
pub fn postprocess_singletons(
    tokens: TokenStream,
    resolved_idents: impl IntoIterator<Item = Ident>,
    context: &Ident,
) -> Punctuated<Expr, Token![,]> {
    let mut resolved_idents_iter = resolved_idents.into_iter();
    let processed = process_singletons(tokens, &mut |singleton_ident| {
        let span = singleton_ident.span();
        let context = Ident::new(&context.to_string(), span.resolved_at(context.span()));
        let mut resolved_ident = resolved_idents_iter.next().unwrap();
        resolved_ident.set_span(span);
        let mut group = Group::new(
            proc_macro2::Delimiter::Parenthesis,
            quote_spanned! {span=>
                *(unsafe {
                    // SAFETY: `handle` is from this instance.
                    #context.state_ref_unchecked(#resolved_ident)
                }.borrow_mut())
            },
        );
        group.set_span(singleton_ident.span());
        TokenTree::Group(group)
    });
    parse_terminated(processed).unwrap()
}

/// Same as [`postprocess_singletons`] but generates just the `StateHandle` ident rather than full
/// `RefCell` borrowing code.
pub fn postprocess_singletons_handles(
    tokens: TokenStream,
    resolved_idents: impl IntoIterator<Item = Ident>,
) -> Punctuated<Expr, Token![,]> {
    let mut resolved_idents_iter = resolved_idents.into_iter();
    let processed = process_singletons(tokens, &mut |singleton_ident| {
        let mut resolved_ident = resolved_idents_iter.next().unwrap();
        resolved_ident.set_span(singleton_ident.span().resolved_at(resolved_ident.span()));
        TokenTree::Ident(resolved_ident)
    });
    parse_terminated(processed).unwrap()
}

/// Traverse the token stream, applying the `map_singleton_fn` whenever a singleton is found,
/// returning the transformed token stream.
fn process_singletons(
    tokens: TokenStream,
    map_singleton_fn: &mut impl FnMut(Ident) -> TokenTree,
) -> TokenStream {
    tokens
        .into_iter()
        .peekable()
        .batching(|iter| {
            let out = match iter.next()? {
                TokenTree::Group(group) => {
                    let mut new_group = Group::new(
                        group.delimiter(),
                        process_singletons(group.stream(), map_singleton_fn),
                    );
                    new_group.set_span(group.span());
                    TokenTree::Group(new_group)
                }
                TokenTree::Ident(ident) => TokenTree::Ident(ident),
                TokenTree::Punct(punct) => {
                    if '#' == punct.as_char() && matches!(iter.peek(), Some(TokenTree::Ident(_))) {
                        // Found a singleton.
                        let Some(TokenTree::Ident(mut singleton_ident)) = iter.next() else {
                            unreachable!()
                        };
                        {
                            // Include the `#` in the span.
                            let span = singleton_ident
                                .span()
                                .join(punct.span())
                                .unwrap_or(singleton_ident.span());
                            singleton_ident.set_span(span.resolved_at(singleton_ident.span()));
                        }
                        (map_singleton_fn)(singleton_ident)
                    } else {
                        TokenTree::Punct(punct)
                    }
                }
                TokenTree::Literal(lit) => TokenTree::Literal(lit),
            };
            Some(out)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::{TokenStream, TokenTree};
    use quote::quote;

    #[test]
    fn test_preprocess_singletons_simple() {
        let tokens = quote! { #my_var };
        let mut found_idents = Vec::new();
        
        let result = preprocess_singletons(tokens, &mut found_idents);
        
        assert_eq!(found_idents.len(), 1);
        assert_eq!(found_idents[0].to_string(), "my_var");
        
        // Result should have the hash removed
        let result_str = result.to_string();
        assert!(!result_str.contains('#'));
        assert!(result_str.contains("my_var"));
    }

    #[test]
    fn test_preprocess_singletons_multiple() {
        let tokens = quote! { #var1 + #var2 };
        let mut found_idents = Vec::new();
        
        let _result = preprocess_singletons(tokens, &mut found_idents);
        
        assert_eq!(found_idents.len(), 2);
        assert_eq!(found_idents[0].to_string(), "var1");
        assert_eq!(found_idents[1].to_string(), "var2");
    }

    #[test]
    fn test_preprocess_singletons_nested() {
        let tokens = quote! { { #outer { #inner } } };
        let mut found_idents = Vec::new();
        
        let _result = preprocess_singletons(tokens, &mut found_idents);
        
        assert_eq!(found_idents.len(), 2);
        assert_eq!(found_idents[0].to_string(), "outer");
        assert_eq!(found_idents[1].to_string(), "inner");
    }

    #[test]
    fn test_preprocess_singletons_no_singletons() {
        let tokens = quote! { regular_var + 123 };
        let mut found_idents = Vec::new();
        
        let result = preprocess_singletons(tokens.clone(), &mut found_idents);
        
        assert_eq!(found_idents.len(), 0);
        // Should be unchanged
        assert_eq!(result.to_string(), tokens.to_string());
    }

    #[test]
    fn test_preprocess_singletons_mixed_content() {
        let tokens = quote! { 
            let x = #singleton_val;
            let y = regular_val;
            #another_singleton
        };
        let mut found_idents = Vec::new();
        
        let _result = preprocess_singletons(tokens, &mut found_idents);
        
        assert_eq!(found_idents.len(), 2);
        assert_eq!(found_idents[0].to_string(), "singleton_val");
        assert_eq!(found_idents[1].to_string(), "another_singleton");
    }

    #[test]
    fn test_process_singletons_identity_transform() {
        let tokens = quote! { #test };
        let mut called = false;
        
        let result = process_singletons(tokens.clone(), &mut |ident| {
            called = true;
            assert_eq!(ident.to_string(), "test");
            TokenTree::Ident(ident)
        });
        
        assert!(called);
        assert!(!result.to_string().contains('#'));
    }

    #[test]
    fn test_process_singletons_in_parentheses() {
        let tokens = quote! { (#var) };
        let mut found_idents = Vec::new();
        
        let _result = preprocess_singletons(tokens, &mut found_idents);
        
        assert_eq!(found_idents.len(), 1);
        assert_eq!(found_idents[0].to_string(), "var");
    }

    #[test]
    fn test_process_singletons_in_brackets() {
        let tokens = quote! { [#arr_elem] };
        let mut found_idents = Vec::new();
        
        let _result = preprocess_singletons(tokens, &mut found_idents);
        
        assert_eq!(found_idents.len(), 1);
        assert_eq!(found_idents[0].to_string(), "arr_elem");
    }

    #[test]
    fn test_process_singletons_in_braces() {
        let tokens = quote! { { #block_var } };
        let mut found_idents = Vec::new();
        
        let _result = preprocess_singletons(tokens, &mut found_idents);
        
        assert_eq!(found_idents.len(), 1);
        assert_eq!(found_idents[0].to_string(), "block_var");
    }

    #[test]
    fn test_preprocess_singletons_preserves_other_hashes() {
        // Test that standalone # not followed by ident is preserved
        let tokens: TokenStream = quote! { ## };
        let mut found_idents = Vec::new();
        
        let result = preprocess_singletons(tokens, &mut found_idents);
        
        assert_eq!(found_idents.len(), 0);
        // Both hashes should be preserved
        assert_eq!(result.to_string().matches('#').count(), 2);
    }

    #[test]
    fn test_postprocess_singletons_handles() {
        let tokens = quote! { #state };
        let resolved_idents = vec![Ident::new("handle_state", proc_macro2::Span::call_site())];
        
        let result = postprocess_singletons_handles(tokens, resolved_idents);
        
        assert_eq!(result.len(), 1);
        let first = result.first().unwrap();
        assert!(quote!(#first).to_string().contains("handle_state"));
    }

    #[test]
    fn test_postprocess_multiple_handles() {
        let tokens = quote! { #var1, #var2 };
        let resolved_idents = vec![
            Ident::new("handle1", proc_macro2::Span::call_site()),
            Ident::new("handle2", proc_macro2::Span::call_site()),
        ];
        
        let result = postprocess_singletons_handles(tokens, resolved_idents);
        
        assert_eq!(result.len(), 2);
    }
}
