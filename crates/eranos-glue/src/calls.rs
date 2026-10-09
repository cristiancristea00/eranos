//! The group calls as the binding core's trait lists them: each call's
//! name, whether it reads or changes the group, how it takes its
//! parameters, what it answers, whether it can fail, its documentation, and
//! its body.
//!
//! A call is a function with a body that takes the group first, as
//! `&ExpenseGroup` to read it or `&mut ExpenseGroup` to change it, then its
//! parameters; it can fail when it answers with a `Result` of its answer and
//! `EranosError`, and cannot when it answers with any other type or with
//! nothing.

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Attribute, Expr, FnArg, FnModifiers, GenericArgument, Ident, ItemFn, ItemTrait, Pat, Path,
    PathArguments, ReturnType, Safety, TraitItem, TraitItemFn, Type, Visibility, parse_quote,
};

use crate::docs::Docs;

/// Whether a call reads the group or changes it, which decides the lock it
/// takes: the group taken as `&ExpenseGroup` is read, and as
/// `&mut ExpenseGroup` changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Access {
    /// A read, under the shared lock.
    Read,
    /// A change, under the exclusive lock.
    Write,
}

/// How a call takes a parameter, which decides the parameter's owned form,
/// the one an asynchronous call moves into its future.
pub(crate) enum Kind {
    /// A list, `&[T]`, owned as a `Vec<T>`.
    List(Type),
    /// Optional text, `Option<&str>`, owned as an `Option<String>`.
    OptionalText,
    /// A record by reference, `&T`, owned as itself.
    Record(Type),
    /// Text, `&str`, owned as a `String`.
    Text,
    /// A value taken whole, such as a strategy, owned as itself.
    Value(Type),
}

/// A parameter of a call.
pub(crate) struct Parameter {
    /// How the call takes it.
    pub(crate) kind: Kind,
    /// Its name.
    pub(crate) name: Ident,
    /// Its type, as the call's synchronous form takes it.
    pub(crate) written: Type,
}

impl Parameter {
    /// The parameter `name` of the type `written`.
    ///
    /// # Errors
    ///
    /// For a type that is neither text, optional text, a record by
    /// reference, a list by slice, nor a value by path.
    fn parse(name: Ident, written: Type) -> syn::Result<Self> {
        let kind = if let Type::Reference(reference) = &written {
            if reference.mutability.is_some() {
                return Err(syn::Error::new_spanned(
                    &written,
                    "a group call takes no parameter by mutable reference",
                ));
            }
            if named(&reference.elem, "str") {
                Kind::Text
            } else if let Type::Slice(slice) = &*reference.elem {
                Kind::List((*slice.elem).clone())
            } else if let Type::Path(_) = &*reference.elem {
                Kind::Record((*reference.elem).clone())
            } else {
                return Err(unsupported(&written));
            }
        } else if let Type::Path(_) = &written {
            if argument(&written, "Option").is_some_and(is_text) {
                Kind::OptionalText
            } else {
                Kind::Value(written.clone())
            }
        } else {
            return Err(unsupported(&written));
        };
        Ok(Self {
            kind,
            name,
            written,
        })
    }

    /// The type of the parameter's owned form.
    pub(crate) fn owned(&self) -> TokenStream {
        match &self.kind {
            Kind::List(element) => quote!(::std::vec::Vec<#element>),
            Kind::OptionalText => {
                quote!(::std::option::Option<::std::string::String>)
            }
            Kind::Record(record) => record.to_token_stream(),
            Kind::Text => quote!(::std::string::String),
            Kind::Value(value) => value.to_token_stream(),
        }
    }

    /// The owned form lent as the synchronous form takes the parameter.
    pub(crate) fn borrowed(&self) -> TokenStream {
        let name = &self.name;
        match &self.kind {
            Kind::List(_) | Kind::Record(_) | Kind::Text => quote!(&#name),
            Kind::OptionalText => quote!(#name.as_deref()),
            Kind::Value(_) => quote!(#name),
        }
    }

    /// The type a foreign binding's synchronous form takes: the core's, but
    /// for optional text, which `uniffi` passes owned.
    pub(crate) fn foreign(&self) -> TokenStream {
        match &self.kind {
            Kind::OptionalText => self.owned(),
            Kind::List(_) | Kind::Record(_) | Kind::Text | Kind::Value(_) => {
                self.written.to_token_stream()
            }
        }
    }

    /// The foreign form lent as the core's synchronous form takes the
    /// parameter.
    pub(crate) fn lent(&self) -> TokenStream {
        let name = &self.name;
        match &self.kind {
            Kind::OptionalText => quote!(#name.as_deref()),
            Kind::List(_) | Kind::Record(_) | Kind::Text | Kind::Value(_) => quote!(#name),
        }
    }
}

/// The failure of a parameter whose type no rule of [`Kind`] covers.
fn unsupported(written: &Type) -> syn::Error {
    syn::Error::new_spanned(
        written,
        "a group call takes text, optional text, a record by reference, a list by slice, or a value",
    )
}

/// Whether `written` is the type `name` alone, a path of one segment
/// without arguments, such as `str` or `bool`.
pub(crate) fn named(written: &Type, name: &str) -> bool {
    matches!(written, Type::Path(path) if path.qself.is_none() && path.path.is_ident(name))
}

/// Whether `written` is text by reference, `&str`.
fn is_text(written: &Type) -> bool {
    matches!(written, Type::Reference(reference) if reference.mutability.is_none() && named(&reference.elem, "str"))
}

/// The type argument of `written` when it is the generic `name` of one
/// argument, such as `Option<T>` or `Vec<T>`.
pub(crate) fn argument<'written>(written: &'written Type, name: &str) -> Option<&'written Type> {
    if let Type::Path(path) = written
        && let Some(segment) = path.path.segments.last()
        && segment.ident == name
        && let PathArguments::AngleBracketed(arguments) = &segment.arguments
        && 1 == arguments.args.len()
        && let Some(GenericArgument::Type(argument)) = arguments.args.first()
    {
        Some(argument)
    } else {
        None
    }
}

/// Whether `path` ends in the segment `name`, whatever its arguments, as
/// `std::result::Result<T, E>` ends in `Result`.
fn ends_in(path: &Path, name: &str) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == name)
}

/// A group call.
pub(crate) struct Call {
    /// Whether it reads the group or changes it.
    pub(crate) access: Access,
    /// What it answers when it succeeds: its `Result`'s first type, or what
    /// it returns when it cannot fail.
    pub(crate) answer: Type,
    /// Its documentation.
    pub(crate) docs: Docs,
    /// The function every form runs under the lock: the call as the trait
    /// writes it, with its documentation and body, private to the module
    /// that lists the calls.
    pub(crate) function: ItemFn,
    /// Its name, which its function shares.
    pub(crate) name: Ident,
    /// Its parameters, in order.
    pub(crate) parameters: Vec<Parameter>,
    /// Where the trait declares it, which the code written for it points at.
    pub(crate) span: Span,
    /// The function its asynchronous form runs instead of its own under the
    /// lock, when that form cannot be derived.
    pub(crate) asynchronous: Option<Path>,
    /// Whether it cannot fail, so the forms wrap its answer.
    pub(crate) infallible: bool,
}

impl Call {
    /// The call `function` declares.
    ///
    /// # Errors
    ///
    /// For a function that is not a documented plain function with a body,
    /// taking the group first by reference, with no attribute but
    /// `#[async_body = path]` besides its documentation, and one whose
    /// `Result` is not of an answer and `EranosError`.
    fn parse(function: &TraitItemFn) -> syn::Result<Self> {
        let mut asynchronous = None;
        for attribute in function
            .attrs
            .iter()
            .filter(|attribute| !attribute.path().is_ident("doc"))
        {
            if attribute.path().is_ident("async_body") {
                asynchronous = Some(asynchronous_body(attribute)?);
            } else {
                return Err(syn::Error::new_spanned(
                    attribute,
                    "a group call takes `#[async_body = path]` alone besides its documentation",
                ));
            }
        }
        let signature = &function.sig;
        if signature.constness.is_some()
            || signature.asyncness.is_some()
            || signature.abi.is_some()
            || signature.variadic.is_some()
            || !matches!(signature.safety, Safety::Default)
            || !signature.generics.params.is_empty()
            || signature.generics.where_clause.is_some()
        {
            return Err(syn::Error::new_spanned(
                signature,
                "a group call is a plain function",
            ));
        }
        let Some(block) = &function.default else {
            return Err(syn::Error::new_spanned(
                signature,
                "a group call has a body, which its forms run under the lock",
            ));
        };
        let mut inputs = signature.inputs.iter();
        let access = inputs.next().and_then(access).ok_or_else(|| {
            syn::Error::new_spanned(
                signature,
                "a group call takes the group first, as `&ExpenseGroup` to read it or `&mut ExpenseGroup` to change it",
            )
        })?;
        let parameters = inputs.map(parameter).collect::<syn::Result<Vec<_>>>()?;
        let Outcome { answer, infallible } = outcome(&signature.output)?;
        let documentation = function
            .attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident("doc"))
            .cloned()
            .collect();
        Ok(Self {
            access,
            answer,
            docs: Docs::parse(&function.attrs, &signature.ident)?,
            function: ItemFn {
                attrs: documentation,
                vis: Visibility::Inherited,
                modifiers: FnModifiers::default(),
                sig: signature.clone(),
                block: Box::new(block.clone()),
            },
            name: signature.ident.clone(),
            parameters,
            span: signature.ident.span(),
            asynchronous,
            infallible,
        })
    }

    /// Whether the call answers with nothing, `()`.
    pub(crate) fn answers_nothing(&self) -> bool {
        matches!(&self.answer, Type::Tuple(tuple) if tuple.elems.is_empty())
    }
}

/// The function that `#[async_body = path]` names.
///
/// # Errors
///
/// For an attribute that does not name a path.
fn asynchronous_body(attribute: &Attribute) -> syn::Result<Path> {
    let value = &attribute.meta.require_name_value()?.value;
    if let Expr::Path(path) = value {
        Ok(path.path.clone())
    } else {
        Err(syn::Error::new_spanned(
            value,
            "`#[async_body = path]` names the function the asynchronous form runs",
        ))
    }
}

/// The access of a call whose first parameter is `input`: a read of the
/// group taken as `&ExpenseGroup`, a change of the group taken as
/// `&mut ExpenseGroup`, and none for any other parameter.
fn access(input: &FnArg) -> Option<Access> {
    if let FnArg::Typed(typed) = input
        && let Type::Reference(reference) = &*typed.ty
        && let Type::Path(path) = &*reference.elem
        && ends_in(&path.path, "ExpenseGroup")
    {
        Some(if reference.mutability.is_some() {
            Access::Write
        } else {
            Access::Read
        })
    } else {
        None
    }
}

/// The parameter that `input` declares.
///
/// # Errors
///
/// For a receiver, a pattern that is not a plain name, and as
/// [`Parameter::parse`].
fn parameter(input: &FnArg) -> syn::Result<Parameter> {
    let FnArg::Typed(typed) = input else {
        return Err(syn::Error::new_spanned(
            input,
            "a group call takes no `self`",
        ));
    };
    let Pat::Ident(pattern) = &*typed.pat else {
        return Err(syn::Error::new_spanned(
            &typed.pat,
            "a group call names each parameter plainly",
        ));
    };
    if pattern.by_ref.is_some() || pattern.mutability.is_some() || pattern.subpat.is_some() {
        return Err(syn::Error::new_spanned(
            &typed.pat,
            "a group call names each parameter plainly",
        ));
    }
    Parameter::parse(pattern.ident.clone(), (*typed.ty).clone())
}

/// What a call answers with, and whether it can fail.
struct Outcome {
    /// What it answers when it succeeds.
    answer: Type,
    /// Whether it cannot fail.
    infallible: bool,
}

/// What a call whose return type is `output` answers with: the first type
/// of a `Result` of an answer and `EranosError`, which can fail; any other
/// type, which cannot; or nothing, `()`, which cannot either.
///
/// # Errors
///
/// For a `Result` of anything but an answer and `EranosError`.
fn outcome(output: &ReturnType) -> syn::Result<Outcome> {
    let ReturnType::Type(_, written) = output else {
        return Ok(Outcome {
            answer: parse_quote!(()),
            infallible: true,
        });
    };
    let Type::Path(path) = &**written else {
        return Ok(Outcome {
            answer: (**written).clone(),
            infallible: true,
        });
    };
    if !ends_in(&path.path, "Result") {
        return Ok(Outcome {
            answer: (**written).clone(),
            infallible: true,
        });
    }
    let refused = || {
        syn::Error::new_spanned(
            written,
            "a group call that can fail answers with a `Result` of its answer and `EranosError`",
        )
    };
    let Some(PathArguments::AngleBracketed(arguments)) =
        path.path.segments.last().map(|segment| &segment.arguments)
    else {
        return Err(refused());
    };
    match arguments.args.iter().collect::<Vec<_>>().as_slice() {
        [
            GenericArgument::Type(answer),
            GenericArgument::Type(Type::Path(error)),
        ] if ends_in(&error.path, "EranosError") => Ok(Outcome {
            answer: answer.clone(),
            infallible: false,
        }),
        _ => Err(refused()),
    }
}

/// The calls of a trait, in its order.
pub(crate) struct Calls {
    /// The calls.
    pub(crate) calls: Vec<Call>,
    /// The trait's name, whose snake case names the table of its calls.
    pub(crate) name: Ident,
}

impl Calls {
    /// The calls `item` lists.
    ///
    /// # Errors
    ///
    /// For a trait with generics, bounds, or an item that is not a
    /// function, and as [`Call::parse`] for each function.
    pub(crate) fn parse(item: &ItemTrait) -> syn::Result<Self> {
        if !item.generics.params.is_empty()
            || item.generics.where_clause.is_some()
            || !item.supertraits.is_empty()
            || item.unsafety.is_some()
        {
            return Err(syn::Error::new_spanned(
                &item.ident,
                "the group calls' trait is a plain trait of functions",
            ));
        }
        let calls = item
            .items
            .iter()
            .map(|item| {
                if let TraitItem::Fn(function) = item {
                    Call::parse(function)
                } else {
                    Err(syn::Error::new_spanned(
                        item,
                        "the group calls' trait holds functions alone",
                    ))
                }
            })
            .collect::<syn::Result<Vec<_>>>()?;
        Ok(Self {
            calls,
            name: item.ident.clone(),
        })
    }
}

#[cfg(test)]
mod tests;
