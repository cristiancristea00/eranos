//! The group calls as the binding core's trait lists them: each call's
//! name, whether it reads or changes the group, how it takes its
//! parameters, what it answers, and its documentation.

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::spanned::Spanned;
use syn::{
    Attribute, Expr, FnArg, GenericArgument, Ident, ItemTrait, Pat, Path, PathArguments,
    ReceiverKind, ReturnType, Safety, TraitItem, TraitItemFn, Type,
};

use crate::docs::Docs;

/// Whether a call reads the group or changes it, which decides the lock it
/// takes.
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
            if "str" == reference.elem.to_token_stream().to_string() {
                Kind::Text
            } else if let Type::Slice(slice) = &*reference.elem {
                Kind::List((*slice.elem).clone())
            } else if let Type::Path(_) = &*reference.elem {
                Kind::Record((*reference.elem).clone())
            } else {
                return Err(unsupported(&written));
            }
        } else if let Type::Path(_) = &written {
            if "Option < & str >" == written.to_token_stream().to_string() {
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

/// A group call.
pub(crate) struct Call {
    /// Whether it reads the group or changes it.
    pub(crate) access: Access,
    /// What it answers when it succeeds.
    pub(crate) answer: Type,
    /// Its documentation.
    pub(crate) docs: Docs,
    /// The error it fails with.
    pub(crate) error: Type,
    /// Its name, which its operation shares.
    pub(crate) name: Ident,
    /// Its parameters, in order.
    pub(crate) parameters: Vec<Parameter>,
    /// Where the trait declares it, which the code written for it points at.
    pub(crate) span: Span,
    /// The function its asynchronous form runs instead of the operation
    /// under the lock, when that form cannot be derived.
    pub(crate) asynchronous: Option<Path>,
    /// Whether its operation cannot fail, so the forms wrap its answer.
    pub(crate) infallible: bool,
}

impl Call {
    /// The call `function` declares.
    ///
    /// # Errors
    ///
    /// For a function that is not a documented `&self` method answering
    /// with a `Result`, with one of `#[read]` and `#[write]` and no attribute
    /// but `#[infallible]` and `#[async_body = path]` besides.
    fn parse(function: &TraitItemFn) -> syn::Result<Self> {
        let mut access = None;
        let mut asynchronous = None;
        let mut infallible = false;
        for attribute in function
            .attrs
            .iter()
            .filter(|attribute| !attribute.path().is_ident("doc"))
        {
            let path = attribute.path();
            if path.is_ident("read") || path.is_ident("write") {
                attribute.meta.require_path_only()?;
                if access.is_some() {
                    return Err(syn::Error::new_spanned(
                        attribute,
                        "a group call either reads or writes the group",
                    ));
                }
                access = Some(if path.is_ident("read") {
                    Access::Read
                } else {
                    Access::Write
                });
            } else if path.is_ident("infallible") {
                attribute.meta.require_path_only()?;
                infallible = true;
            } else if path.is_ident("async_body") {
                asynchronous = Some(body(attribute)?);
            } else {
                return Err(syn::Error::new_spanned(
                    attribute,
                    "a group call takes `#[read]` or `#[write]`, `#[infallible]`, and `#[async_body = path]`",
                ));
            }
        }
        let access = access.ok_or_else(|| {
            syn::Error::new_spanned(
                &function.sig.ident,
                "a group call says whether it reads or writes the group, with `#[read]` or `#[write]`",
            )
        })?;
        let signature = &function.sig;
        if function.default.is_some()
            || signature.constness.is_some()
            || signature.asyncness.is_some()
            || signature.abi.is_some()
            || signature.variadic.is_some()
            || !matches!(signature.safety, Safety::Default)
            || !signature.generics.params.is_empty()
            || signature.generics.where_clause.is_some()
        {
            return Err(syn::Error::new_spanned(
                signature,
                "a group call is a plain method, declared without a body",
            ));
        }
        let mut inputs = signature.inputs.iter();
        let receiver = inputs.next().and_then(|input| {
            if let FnArg::Receiver(receiver) = input {
                Some(receiver)
            } else {
                None
            }
        });
        if !receiver
            .is_some_and(|receiver| matches!(receiver.kind, ReceiverKind::Reference(_, None, None)))
        {
            return Err(syn::Error::new_spanned(
                signature,
                "a group call takes `&self`",
            ));
        }
        let parameters = inputs.map(parameter).collect::<syn::Result<Vec<_>>>()?;
        let (answer, error) = outcome(&signature.output)?;
        Ok(Self {
            access,
            answer,
            docs: Docs::parse(&function.attrs, &signature.ident)?,
            error,
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
fn body(attribute: &Attribute) -> syn::Result<Path> {
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

/// The parameter that `input` declares.
///
/// # Errors
///
/// For a pattern that is not a plain name, and as [`Parameter::parse`].
fn parameter(input: &FnArg) -> syn::Result<Parameter> {
    let FnArg::Typed(typed) = input else {
        return Err(syn::Error::new_spanned(
            input,
            "a group call takes `&self` first and once",
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

/// The answer and the error of `output`, a `Result<T, E>`.
///
/// # Errors
///
/// For any other return type.
fn outcome(output: &ReturnType) -> syn::Result<(Type, Type)> {
    let refused = || {
        syn::Error::new(
            output.span(),
            "a group call answers with a `Result` of its answer and its error",
        )
    };
    let ReturnType::Type(_, written) = output else {
        return Err(refused());
    };
    let Type::Path(path) = &**written else {
        return Err(refused());
    };
    let segment = path.path.segments.last().ok_or_else(refused)?;
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(refused());
    };
    let types = arguments
        .args
        .iter()
        .map(|argument| {
            if let GenericArgument::Type(generic) = argument {
                Some(generic)
            } else {
                None
            }
        })
        .collect::<Option<Vec<_>>>()
        .ok_or_else(refused)?;
    match (segment.ident.to_string().as_str(), types.as_slice()) {
        ("Result", [answer, error]) => Ok(((*answer).clone(), (*error).clone())),
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
    /// For a trait with generics, bounds, or an item that is not a method,
    /// and as [`Call::parse`] for each method.
    pub(crate) fn parse(item: &ItemTrait) -> syn::Result<Self> {
        if !item.generics.params.is_empty()
            || item.generics.where_clause.is_some()
            || !item.supertraits.is_empty()
            || item.unsafety.is_some()
        {
            return Err(syn::Error::new_spanned(
                &item.ident,
                "the group calls' trait is a plain trait of methods",
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
                        "the group calls' trait holds methods alone",
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
