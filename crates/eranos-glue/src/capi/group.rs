//! The bridge's group: every call's synchronous form, which blocks while
//! another call holds the lock, and its asynchronous form, which runs on the
//! group's worker thread and reports its outcome to a callback there; the
//! read handle's form of every read; the cancellation of asynchronous calls;
//! and the outcome of each kind of answer, which a callback receives.
//!
//! A call that takes optional text has a twin of each form that passes none,
//! since C passes no absent text: `set_language` has `clear_language`. The
//! group's constructors and read handle, the read handle's release, the
//! cancellation, and the outcomes that hand over a group or a read handle
//! are templates.

use std::collections::BTreeMap;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::calls::{Access, Call, Kind, Parameter};
use crate::capi::crossing::{Answer, Crossing, parameter_crossing};
use crate::capi::items::Core;
use crate::capi::{INVALID_TEXT, Named, article, doc, eranos, list, words};
use crate::docs::foreign;

/// The failure of a read's synchronous form on wasm, where it cannot wait
/// for the lock.
const READ_BUSY: &str = "[`EranosError`] of the kind `Group` with the failure `GroupBusy` on wasm while a change holds the lock or waits for it.";

/// The failure of a change's synchronous form on wasm, where it cannot wait
/// for the lock.
const WRITE_BUSY: &str = "[`EranosError`] of the kind `Group` with the failure `GroupBusy` on wasm while another call holds the lock.";

/// The failure of a read handle's form once the handle is released.
const RELEASED: &str = "[`EranosError`] of the kind `Released` after `release`.";

/// The items of the group, its read handle, the cancellation, and the
/// outcomes, from the calls of `core`.
///
/// # Errors
///
/// For a call whose parameters or answer the bridge cannot carry, and a call
/// that takes optional text more than once, or without setting it.
pub(crate) fn items(core: &Core) -> syn::Result<Vec<Named>> {
    let mut group = Vec::new();
    let mut reader = Vec::new();
    let mut outcomes = BTreeMap::new();
    for call in &core.calls.calls {
        let answer = Answer::of(&call.answer, core)?;
        let forms = forms(call, core)?;
        let busy = match call.access {
            Access::Read => READ_BUSY,
            Access::Write => WRITE_BUSY,
        };
        group.extend(
            forms
                .iter()
                .map(|form| synchronous(call, &answer, form, &quote!(self.handle), busy)),
        );
        group.extend(forms.iter().map(|form| asynchronous(call, &answer, form)));
        if Access::Read == call.access {
            reader.extend(
                forms
                    .iter()
                    .map(|form| synchronous(call, &answer, form, &quote!(self.0), RELEASED)),
            );
        }
        let outcome = outcome(&answer);
        outcomes.insert(outcome.name.clone(), outcome);
    }
    let mut items = vec![
        cancellation(),
        group_item(&group),
        group_outcome(),
        reader_item(&reader),
        reader_outcome(),
    ];
    items.extend(outcomes.into_values());
    Ok(items)
}

/// A parameter of a form.
#[derive(Clone)]
struct Argument<'call> {
    /// The core's parameter.
    parameter: &'call Parameter,
    /// How it crosses.
    crossing: Crossing,
}

/// A form of a call: its name, and what it passes for each of the core's
/// parameters.
struct Form<'call> {
    /// Its name: the call's, or its clearing twin's.
    name: Ident,
    /// Each of the core's parameters as the form takes it, or none for the
    /// optional text that the clearing twin passes as `None`.
    arguments: Vec<Option<Argument<'call>>>,
}

/// The forms of `call`: the call itself, and when it takes optional text,
/// the twin that passes none, named `clear_` for its `set_`.
///
/// # Errors
///
/// For a parameter the bridge cannot carry, optional text taken more than
/// once, and a call that takes it without a name that starts with `set_`.
fn forms<'call>(call: &'call Call, core: &Core) -> syn::Result<Vec<Form<'call>>> {
    let arguments = call
        .parameters
        .iter()
        .map(|parameter| {
            Ok(Argument {
                parameter,
                crossing: parameter_crossing(&parameter.kind, core)?,
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let full = Form {
        name: call.name.clone(),
        arguments: arguments.iter().cloned().map(Some).collect(),
    };
    let optional = arguments
        .iter()
        .filter(|argument| matches!(argument.parameter.kind, Kind::OptionalText))
        .count();
    match optional {
        0 => Ok(vec![full]),
        1 => {
            let name = call.name.to_string();
            let Some(set) = name.strip_prefix("set_") else {
                return Err(syn::Error::new(
                    call.span,
                    "a group call that takes optional text sets it, and its name starts with `set_`",
                ));
            };
            let clearing = Form {
                name: format_ident!("clear_{set}"),
                arguments: arguments
                    .into_iter()
                    .map(|argument| {
                        (!matches!(argument.parameter.kind, Kind::OptionalText)).then_some(argument)
                    })
                    .collect(),
            };
            Ok(vec![full, clearing])
        }
        _ => Err(syn::Error::new(
            call.span,
            "a group call takes optional text once at most",
        )),
    }
}

/// How a synchronous form hands out its answer.
struct Delivery {
    /// The form's body.
    body: TokenStream,
    /// What its `Result` holds when it succeeds.
    output: TokenStream,
    /// The parameter it writes its text into, for an answer of text.
    write: Option<TokenStream>,
}

/// The synchronous `form` of `call`, which answers `answer`: it calls the
/// core's form on `receiver`, the group's handle or the read handle, which
/// fails as `own` says besides the call's refusal.
fn synchronous(
    call: &Call,
    answer: &Answer,
    form: &Form<'_>,
    receiver: &TokenStream,
    own: &str,
) -> TokenStream {
    let (name, core_name) = (&form.name, &call.name);
    let parameters = form
        .arguments
        .iter()
        .flatten()
        .map(|argument| argument.crossing.parameter(&argument.parameter.name));
    let arguments = form
        .arguments
        .iter()
        .map(|argument| argument.as_ref().map_or_else(|| quote!(None), lent));
    let invocation = quote!(#receiver.#core_name(#(#arguments),*));
    let write = quote!(, write: &mut DiplomatWrite);
    let Delivery {
        body,
        output,
        write,
    } = match answer {
        Answer::Flag => Delivery {
            body: quote!(#invocation.map_err(crate::failed)),
            output: quote!(bool),
            write: None,
        },
        Answer::Found(record) => {
            let object = eranos(record);
            Delivery {
                body: quote! {
                    #invocation
                        .map(|value| value.map(|value| Box::new(#object(value))))
                        .map_err(crate::failed)
                },
                output: quote!(Option<Box<#object>>),
                write: None,
            }
        }
        Answer::List(element) => {
            let list = list(element);
            Delivery {
                body: quote! {
                    #invocation
                        .map(|list| Box::new(#list(list)))
                        .map_err(crate::failed)
                },
                output: quote!(Box<#list>),
                write: None,
            }
        }
        Answer::Nothing => Delivery {
            body: quote!(#invocation.map_err(crate::failed)),
            output: quote!(()),
            write: None,
        },
        Answer::OptionalText => Delivery {
            body: quote! {
                let text = #invocation.map_err(crate::failed)?;
                crate::write_optional(write, text.as_deref());
                Ok(())
            },
            output: quote!(()),
            write: Some(write),
        },
        Answer::Record(record) => {
            let object = eranos(record);
            Delivery {
                body: quote! {
                    #invocation
                        .map(|value| Box::new(#object(value)))
                        .map_err(crate::failed)
                },
                output: quote!(Box<#object>),
                write: None,
            }
        }
        Answer::Text => Delivery {
            body: quote! {
                let text = #invocation.map_err(crate::failed)?;
                crate::write_text(write, &text);
                Ok(())
            },
            output: quote!(()),
            write: Some(write),
        },
    };
    let documentation = synchronous_doc(call, answer, form, own);
    quote! {
        #documentation
        pub fn #name(&self #(, #parameters)* #write) -> Result<#output, Box<EranosError>> {
            #body
        }
    }
}

/// The value the synchronous form lends the core's for `argument`: text
/// checked, a value converted, and a record or a list by reference, in
/// `Some` for optional text.
fn lent(argument: &Argument<'_>) -> TokenStream {
    let name = &argument.parameter.name;
    let lent = match &argument.crossing {
        Crossing::Choice(_) | Crossing::Plain(_) => quote!(#name.into()),
        Crossing::Integer(_) => quote!(#name),
        Crossing::List(_) | Crossing::Object(_) | Crossing::TextList => quote!(&#name.0),
        Crossing::Text => quote!(&crate::text(#name)?),
    };
    if matches!(argument.parameter.kind, Kind::OptionalText) {
        quote!(Some(#lent))
    } else {
        lent
    }
}

/// The documentation of the synchronous `form` of `call`: the call's, then
/// its failures, the call's refusal unless it cannot fail, `own`, and text
/// that is not UTF-8 for a form that takes text.
fn synchronous_doc(call: &Call, answer: &Answer, form: &Form<'_>, own: &str) -> TokenStream {
    let writes = match answer {
        Answer::OptionalText => " Writes it into `write` when there is one, and nothing otherwise.",
        Answer::Text => " Writes it into `write`.",
        Answer::Flag | Answer::Found(_) | Answer::List(_) | Answer::Nothing | Answer::Record(_) => {
            ""
        }
    };
    let summary = if form.name == call.name {
        format!("{}{writes}", foreign(call.docs.summary()))
    } else {
        let absent = form
            .arguments
            .iter()
            .zip(&call.parameters)
            .filter(|(argument, _)| argument.is_none())
            .map(|(_, parameter)| format!("`{}`", parameter.name))
            .collect::<Vec<_>>()
            .join(", ");
        format!("The form of `{}` that passes no {absent}.", call.name)
    };
    let mut errors = Vec::new();
    if !call.infallible {
        errors.push("The library's refusal.");
    }
    errors.push(own);
    if form
        .arguments
        .iter()
        .flatten()
        .any(|argument| Crossing::Text == argument.crossing)
    {
        errors.push(INVALID_TEXT);
    }
    let errors = errors.join(" ");
    let details = call
        .docs
        .details()
        .iter()
        .map(|detail| foreign(detail))
        .collect::<Vec<_>>();
    let mut paragraphs = vec![summary.as_str()];
    paragraphs.extend(details.iter().map(String::as_str));
    paragraphs.extend(["# Errors", errors.as_str()]);
    doc(&paragraphs)
}

/// The asynchronous `form` of `call`, which answers `answer`: it prepares
/// its parameters before it runs, so the caller's objects are not needed
/// after it returns, and reports text that is not UTF-8 to its callback.
fn asynchronous(call: &Call, answer: &Answer, form: &Form<'_>) -> TokenStream {
    let method = format_ident!("{}_async", form.name);
    let core_name = format_ident!("{}_async", call.name);
    let taken = form.arguments.iter().flatten().collect::<Vec<_>>();
    let parameters = taken
        .iter()
        .map(|argument| argument.crossing.parameter(&argument.parameter.name));
    let prepared = taken.iter().map(|argument| {
        let name = &argument.parameter.name;
        let value = match &argument.crossing {
            Crossing::Choice(_) | Crossing::Plain(_) => quote!(Ok(#name.into())),
            Crossing::Integer(_) => quote!(Ok(#name)),
            Crossing::List(_) | Crossing::Object(_) | Crossing::TextList => {
                quote!(Ok(#name.0.clone()))
            }
            Crossing::Text => quote!(crate::decoded(#name)),
        };
        quote!(let #name = #value;)
    });
    let arguments = form.arguments.iter().map(|argument| {
        argument.as_ref().map_or_else(
            || quote!(None),
            |argument| {
                let name = &argument.parameter.name;
                if matches!(argument.parameter.kind, Kind::OptionalText) {
                    quote!(Some(#name?))
                } else {
                    quote!(#name?)
                }
            },
        )
    });
    let some = matches!(answer, Answer::Record(_)).then(|| quote!(.map(Some)));
    let outcome = outcome_name(answer);
    let documentation = doc(&[&format!(
        "The asynchronous form of `{}`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind `Cancelled`, leaving the group as it was.",
        form.name
    )]);
    quote! {
        #documentation
        pub fn #method(
            &self,
            #(#parameters,)*
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<#outcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            #(#prepared)*
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .#core_name(#(#arguments,)* cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                        #some
                },
                callback,
                #outcome,
            );
        }
    }
}

/// The name of the outcome of a call that answers `answer`.
fn outcome_name(answer: &Answer) -> Ident {
    match answer {
        Answer::Flag => format_ident!("EranosFlagOutcome"),
        Answer::Found(record) | Answer::Record(record) => format_ident!("Eranos{record}Outcome"),
        Answer::List(element) => format_ident!("Eranos{element}ListOutcome"),
        Answer::Nothing => format_ident!("EranosUnitOutcome"),
        Answer::OptionalText => format_ident!("EranosOptionalTextOutcome"),
        Answer::Text => format_ident!("EranosTextOutcome"),
    }
}

/// The outcome of a call that answers `answer`, which holds the call's
/// failure or a copy of its answer: a record's in an `Option`, since a call
/// that answers a record and one that may find none share it.
fn outcome(answer: &Answer) -> Named {
    let name = outcome_name(answer);
    let Holding {
        answer,
        returns,
        value,
    } = holding(answer);
    let summary = doc(&[&format!(
        "The outcome of an asynchronous call that returns {returns}, which its callback receives and destroys."
    )]);
    Named {
        name: name.to_string(),
        tokens: quote! {
            #summary
            #[diplomat::opaque]
            #[derive(Debug)]
            pub struct #name(pub(crate) Result<#answer, crate::Failure>);

            impl #name {
                #[doc = "Whether the call succeeded."]
                #[must_use]
                pub const fn is_ok(&self) -> bool {
                    self.0.is_ok()
                }

                #[doc = "A copy of the call's failure, absent when it succeeded."]
                #[must_use]
                pub fn error(&self) -> Option<Box<EranosError>> {
                    self.0
                        .as_ref()
                        .err()
                        .map(|failure| Box::new(EranosError::from(failure.clone())))
                }

                #value
            }
        },
    }
}

/// What an outcome holds of a call's answer.
struct Holding {
    /// The type of the answer it holds.
    answer: TokenStream,
    /// What the call returns, in words.
    returns: String,
    /// Its method that hands the answer over, if it has one.
    value: TokenStream,
}

/// What the outcome of a call that answers `answer` holds, and its method
/// that hands a copy over: none for nothing, the flag, the text written
/// into a `DiplomatWrite`, and a record or a list in an opaque object.
fn holding(answer: &Answer) -> Holding {
    match answer {
        Answer::Flag => Holding {
            answer: quote!(bool),
            returns: "an answer".to_owned(),
            value: quote! {
                #[doc = "The answer, false when the call failed."]
                #[must_use]
                pub const fn value(&self) -> bool {
                    matches!(self.0, Ok(true))
                }
            },
        },
        Answer::Found(record) | Answer::Record(record) => {
            let object = eranos(record);
            let copy = format!(
                "A copy of the {} the call returned, if it succeeded and found one.",
                words(record)
            );
            Holding {
                answer: quote!(Option<eranos_bindings::#record>),
                returns: article(&words(record)),
                value: quote! {
                    #[doc = #copy]
                    #[must_use]
                    pub fn value(&self) -> Option<Box<#object>> {
                        self.0
                            .as_ref()
                            .ok()
                            .and_then(Option::as_ref)
                            .map(|value| Box::new(#object(value.clone())))
                    }
                },
            }
        }
        Answer::List(element) => {
            let list = list(element);
            Holding {
                answer: quote!(Vec<eranos_bindings::#element>),
                returns: format!("a list of {} records", words(element)),
                value: quote! {
                    #[doc = "A copy of the list the call returned, if it succeeded."]
                    #[must_use]
                    pub fn value(&self) -> Option<Box<#list>> {
                        self.0
                            .as_ref()
                            .ok()
                            .map(|list| Box::new(#list(list.clone())))
                    }
                },
            }
        }
        Answer::Nothing => Holding {
            answer: quote!(()),
            returns: "nothing".to_owned(),
            value: TokenStream::new(),
        },
        Answer::OptionalText => Holding {
            answer: quote!(Option<String>),
            returns: "text it may not have".to_owned(),
            value: quote! {
                #[doc = "Writes the text the call returned, if it succeeded and found some; absent otherwise."]
                pub fn value(&self, write: &mut DiplomatWrite) -> Option<()> {
                    crate::write_optional(write, self.0.as_ref().ok().and_then(Option::as_deref))
                }
            },
        },
        Answer::Text => Holding {
            answer: quote!(String),
            returns: "text".to_owned(),
            value: quote! {
                #[doc = "Writes the text the call returned, if it succeeded; absent otherwise."]
                pub fn value(&self, write: &mut DiplomatWrite) -> Option<()> {
                    crate::write_optional(write, self.0.as_deref().ok())
                }
            },
        },
    }
}

/// The cancellation of asynchronous calls.
fn cancellation() -> Named {
    Named {
        name: "EranosCancellation".to_owned(),
        tokens: quote! {
            /// A cancellation of asynchronous calls, which one cancellation can be given to several of.
            ///
            /// Cancelling it stops the calls it was given at their next yield, or at once while they
            /// wait for the lock, and they report the kind `Cancelled`, leaving the group as it was.
            #[diplomat::opaque]
            #[derive(Debug)]
            pub struct EranosCancellation(pub(crate) Arc<eranos_bindings::Cancellation>);

            impl EranosCancellation {
                /// A cancellation not yet cancelled.
                #[must_use]
                pub fn create() -> Box<Self> {
                    Box::new(Self(Arc::default()))
                }

                /// Cancels every call it was given.
                pub fn cancel(&self) {
                    self.0.cancel();
                }

                /// Whether it was cancelled.
                #[must_use]
                pub fn is_cancelled(&self) -> bool {
                    self.0.is_cancelled()
                }
            }
        },
    }
}

/// The group, with its constructors and read handle around `methods`, the
/// forms of every call.
fn group_item(methods: &[TokenStream]) -> Named {
    Named {
        name: "EranosGroup".to_owned(),
        tokens: quote! {
            /// A group: its details, members, categories, expenses, transfers, totals, and
            /// settlements.
            ///
            /// Every call has a synchronous form, which blocks the calling thread while another
            /// call holds the lock, and an asynchronous one, which runs on the group's worker
            /// thread, started by the first, in the order of the calls, and reports to its callback
            /// there, so a callback must be safe to call from another thread. Destroying the group
            /// lets the queued calls finish and their callbacks run.
            #[diplomat::opaque]
            #[derive(Debug)]
            pub struct EranosGroup {
                pub(crate) handle: eranos_bindings::GroupHandle,
                pub(crate) jobs: OnceLock<mpsc::Sender<crate::Job>>,
            }

            impl EranosGroup {
                /// A new group, with the details `details` holds.
                ///
                /// # Errors
                ///
                /// [`EranosError`] of the kind `Money` for a currency the library does not list, of
                /// the kind `Group` for a blank name, and of the kind `Language` for a language
                /// that is not a tag.
                pub fn create(details: &EranosNewGroup) -> Result<Box<Self>, Box<EranosError>> {
                    eranos_bindings::GroupHandle::new(&details.0)
                        .map(|handle| Box::new(Self::from(handle)))
                        .map_err(crate::failed)
                }

                /// The group saved as `json`, which `save` wrote.
                ///
                /// # Errors
                ///
                /// [`EranosError`] of the kind `InvalidJson` for text that is not a saved group,
                /// of the kind `Group` for a format this release cannot read, of the module whose
                /// rule an entry breaks, and of the kind `InvalidText` for text that is not UTF-8.
                pub fn load(json: &DiplomatStr) -> Result<Box<Self>, Box<EranosError>> {
                    eranos_bindings::GroupHandle::load(&crate::text(json)?)
                        .map(|handle| Box::new(Self::from(handle)))
                        .map_err(crate::failed)
                }

                /// The asynchronous form of `load`: it runs on a thread of its own and reports its
                /// outcome to `callback` there.
                pub fn load_async(
                    json: &DiplomatStr,
                    cancellation: Option<&EranosCancellation>,
                    callback: impl Fn(Box<EranosGroupOutcome>) + 'static,
                ) {
                    let json = crate::decoded(json);
                    let cancellation = cancellation.map(EranosCancellation::shared);
                    crate::run(
                        async move {
                            eranos_bindings::GroupHandle::load_async(json?, cancellation)
                                .await
                                .map(Self::from)
                                .map_err(crate::Failure::Library)
                        },
                        callback,
                        |outcome| EranosGroupOutcome(outcome.map(Some)),
                    );
                }

                #(#methods)*

                /// A read handle, which holds the shared lock until it is released or dropped, so
                /// that its reads see no change in between.
                ///
                /// # Errors
                ///
                /// [`EranosError`] of the kind `Group` with the failure `GroupBusy` on wasm while a
                /// change holds the lock or waits for it.
                pub fn read(&self) -> Result<Box<EranosGroupReader>, Box<EranosError>> {
                    self.handle
                        .read()
                        .map(|reader| Box::new(EranosGroupReader(reader)))
                        .map_err(crate::failed)
                }

                /// The asynchronous form of `read`: it runs on the group's worker thread, in the
                /// order of the calls, and reports its outcome to `callback` there. Cancelling
                /// `cancellation` stops the call at its next yield, or at once while it waits for
                /// the lock, and reports the kind `Cancelled`, leaving the group as it was.
                pub fn read_async(
                    &self,
                    cancellation: Option<&EranosCancellation>,
                    callback: impl Fn(Box<EranosGroupReaderOutcome>) + 'static,
                ) {
                    let handle = self.handle.clone();
                    let cancellation = cancellation.map(EranosCancellation::shared);
                    crate::spawn(
                        &self.jobs,
                        async move {
                            handle
                                .read_async(cancellation)
                                .await
                                .map_err(crate::Failure::Library)
                                .map(Some)
                        },
                        callback,
                        EranosGroupReaderOutcome,
                    );
                }
            }
        },
    }
}

/// The outcome of the asynchronous load, which hands the group over once.
fn group_outcome() -> Named {
    Named {
        name: "EranosGroupOutcome".to_owned(),
        tokens: quote! {
            /// The outcome of the asynchronous load, which its callback receives and destroys.
            #[diplomat::opaque_mut]
            #[derive(Debug)]
            pub struct EranosGroupOutcome(pub(crate) Result<Option<EranosGroup>, crate::Failure>);

            impl EranosGroupOutcome {
                /// Whether the load succeeded.
                #[must_use]
                pub const fn is_ok(&self) -> bool {
                    self.0.is_ok()
                }

                /// A copy of the load's failure, absent when it succeeded.
                #[must_use]
                pub fn error(&self) -> Option<Box<EranosError>> {
                    self.0
                        .as_ref()
                        .err()
                        .map(|failure| Box::new(EranosError::from(failure.clone())))
                }

                /// Takes the group, if the load succeeded and it was not taken before.
                pub fn take(&mut self) -> Option<Box<EranosGroup>> {
                    self.0.as_mut().ok().and_then(Option::take).map(Box::new)
                }
            }
        },
    }
}

/// The read handle, with its release before `methods`, the forms of every
/// read.
fn reader_item(methods: &[TokenStream]) -> Named {
    Named {
        name: "EranosGroupReader".to_owned(),
        tokens: quote! {
            /// A read handle: it holds the group's shared lock until it is released or destroyed,
            /// so the reads of one report see no change in between; a change waits for it.
            #[diplomat::opaque]
            #[derive(Debug)]
            pub struct EranosGroupReader(pub(crate) eranos_bindings::GroupReader);

            impl EranosGroupReader {
                /// Releases the lock at once; later reads report the kind `Released`.
                pub fn release(&self) {
                    self.0.release();
                }

                #(#methods)*
            }
        },
    }
}

/// The outcome of the asynchronous read handle, which hands the handle over
/// once.
fn reader_outcome() -> Named {
    Named {
        name: "EranosGroupReaderOutcome".to_owned(),
        tokens: quote! {
            /// The outcome of an asynchronous call that returns a read handle, which its callback
            /// receives and destroys.
            #[diplomat::opaque_mut]
            #[derive(Debug)]
            pub struct EranosGroupReaderOutcome(
                pub(crate) Result<Option<eranos_bindings::GroupReader>, crate::Failure>,
            );

            impl EranosGroupReaderOutcome {
                /// Whether the call succeeded.
                #[must_use]
                pub const fn is_ok(&self) -> bool {
                    self.0.is_ok()
                }

                /// A copy of the call's failure, absent when it succeeded.
                #[must_use]
                pub fn error(&self) -> Option<Box<EranosError>> {
                    self.0
                        .as_ref()
                        .err()
                        .map(|failure| Box::new(EranosError::from(failure.clone())))
                }

                /// Takes the read handle, if the call succeeded and it was not taken before.
                pub fn take(&mut self) -> Option<Box<EranosGroupReader>> {
                    self.0
                        .as_mut()
                        .ok()
                        .and_then(Option::take)
                        .map(|reader| Box::new(EranosGroupReader(reader)))
                }
            }
        },
    }
}

#[cfg(test)]
mod tests;
