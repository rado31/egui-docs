//! Translations.
//!
//! Every piece of text a reader sees lives in `locales/<language>/*.ftl`
//! ([Fluent](https://projectfluent.org) files), never in Rust. `build.rs`
//! embeds every folder it finds there, so a new language is a new folder.
//!
//! English (`en`) is the source language: every other language is checked
//! against it by the tests, and it is what a lookup falls back to when a
//! message is missing, so a half-finished translation still runs.
//!
//! Lessons receive a [`Tr`] — a view of one language — in `demo`, `controls`
//! and `code`. Both the demo and its generated snippet read their text through
//! it, which keeps the two in sync in every language.

use std::cell::RefCell;
use std::collections::BTreeSet;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

include!(concat!(env!("OUT_DIR"), "/locales.rs"));

/// The language the others are translated from, and fall back to.
pub const SOURCE_LANGUAGE: &str = "en";

struct Language {
    code: &'static str,
    bundle: FluentBundle<FluentResource>,
}

pub struct I18n {
    /// The source language first, the rest in alphabetical order.
    languages: Vec<Language>,
    current: usize,
    /// Syntax errors and duplicate ids found while loading.
    problems: Vec<String>,
    /// Every lookup the asked-for language could not answer by itself. Only
    /// ever read by the tests: in the app, falling back is the right behaviour.
    missing: RefCell<BTreeSet<String>>,
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}

impl I18n {
    /// Every embedded language, starting in the source language.
    pub fn new() -> Self {
        let mut problems = Vec::new();

        let mut languages: Vec<Language> = LOCALES
            .iter()
            .map(|(code, files)| {
                let id: LanguageIdentifier = code.parse().unwrap_or_else(|_| {
                    problems.push(format!("locales/{code}: not a language code"));
                    LanguageIdentifier::default()
                });
                let mut bundle = FluentBundle::new(vec![id]);
                // By default Fluent wraps every `{ $variable }` in invisible
                // Unicode direction marks. egui's fonts have no glyph for them,
                // so each one would be drawn as a box.
                bundle.set_use_isolating(false);

                for (name, source) in *files {
                    let resource = FluentResource::try_new((*source).to_owned()).unwrap_or_else(
                        |(resource, errors)| {
                            for error in errors {
                                problems.push(format!("locales/{code}/{name}: {error:?}"));
                            }
                            resource
                        },
                    );
                    if let Err(errors) = bundle.add_resource(resource) {
                        for error in errors {
                            problems.push(format!("locales/{code}/{name}: {error:?}"));
                        }
                    }
                }
                Language { code, bundle }
            })
            .collect();

        // Stable, so the others keep the alphabetical order `build.rs` gave them.
        languages.sort_by_key(|language| language.code != SOURCE_LANGUAGE);
        assert!(
            languages.first().is_some_and(|l| l.code == SOURCE_LANGUAGE),
            "locales/{SOURCE_LANGUAGE}/ is missing"
        );

        for problem in &problems {
            log::error!("{problem}");
        }

        Self {
            languages,
            current: 0,
            problems,
            missing: RefCell::default(),
        }
    }

    /// The current language.
    pub fn tr(&self) -> Tr<'_> {
        Tr {
            i18n: self,
            language: self.current,
        }
    }

    /// Codes of every available language, source language first.
    pub fn codes(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.languages.iter().map(|language| language.code)
    }

    /// Switch language. Returns `false`, and changes nothing, for an unknown code.
    pub fn set_language(&mut self, code: &str) -> bool {
        match self.index_of(code) {
            Some(index) => {
                self.current = index;
                true
            }
            None => false,
        }
    }

    /// A view of any language, not only the current one.
    pub fn tr_in(&self, code: &str) -> Option<Tr<'_>> {
        self.index_of(code).map(|language| Tr {
            i18n: self,
            language,
        })
    }

    /// The best available match for a locale like `ru-RU`, `pt_BR` or `en`:
    /// the exact tag if there is a folder for it, else its language part.
    pub fn best_match(&self, locale: &str) -> Option<&'static str> {
        let locale = locale.replace('_', "-");
        let primary = locale.split('-').next().unwrap_or_default();
        [locale.as_str(), primary]
            .into_iter()
            .find_map(|wanted| {
                self.languages
                    .iter()
                    .find(|language| language.code.eq_ignore_ascii_case(wanted))
            })
            .map(|language| language.code)
    }

    /// Problems found in the `.ftl` files while loading. Empty when all is well.
    pub fn problems(&self) -> &[String] {
        &self.problems
    }

    /// The lookups that fell back or failed since the last call, as
    /// `language: message-id`.
    pub fn take_missing(&self) -> BTreeSet<String> {
        self.missing.take()
    }

    /// Every id a language defines: `message` for each message with a value,
    /// `message.attribute` for each attribute.
    pub fn ids(&self, code: &str) -> BTreeSet<String> {
        use fluent_syntax::ast::Entry;

        let Some(index) = self.index_of(code) else {
            return BTreeSet::new();
        };
        let mut ids = BTreeSet::new();
        for (_, source) in LOCALES
            .iter()
            .filter(|(c, _)| *c == self.languages[index].code)
            .flat_map(|(_, files)| files.iter())
        {
            let resource = match FluentResource::try_new((*source).to_owned()) {
                Ok(resource) | Err((resource, _)) => resource,
            };
            for entry in resource.entries() {
                if let Entry::Message(message) = entry {
                    let name = message.id.name;
                    if message.value.is_some() {
                        ids.insert(name.to_owned());
                    }
                    for attribute in &message.attributes {
                        ids.insert(format!("{name}.{}", attribute.id.name));
                    }
                }
            }
        }
        ids
    }

    fn index_of(&self, code: &str) -> Option<usize> {
        self.languages
            .iter()
            .position(|language| language.code == code)
    }
}

/// One language's text. Cheap to copy: it is two words.
#[derive(Clone, Copy)]
pub struct Tr<'a> {
    i18n: &'a I18n,
    language: usize,
}

impl Tr<'_> {
    /// The language code, e.g. `"ru"`.
    pub fn language(&self) -> &'static str {
        self.i18n.languages[self.language].code
    }

    /// A message by id. `message.attribute` reads one of its attributes.
    pub fn get(&self, id: &str) -> String {
        self.fmt(id, &[])
    }

    /// A message with variables, e.g.
    /// `tr.fmt("id-distinct", &[("ids", ids.len().into())])`.
    ///
    /// Pass numbers as numbers, not preformatted strings: Fluent picks the
    /// plural form (Russian has three) from the value.
    pub fn fmt(&self, id: &str, args: &[(&str, FluentValue<'_>)]) -> String {
        let mut fluent_args = FluentArgs::new();
        for (name, value) in args {
            fluent_args.set(*name, value.clone());
        }

        let (message, attribute) = match id.split_once('.') {
            Some((message, attribute)) => (message, Some(attribute)),
            None => (id, None),
        };

        for index in [self.language, 0] {
            let language = &self.i18n.languages[index];
            let bundle = &language.bundle;
            let pattern = bundle
                .get_message(message)
                .and_then(|message| match attribute {
                    Some(attribute) => message.get_attribute(attribute).map(|a| a.value()),
                    None => message.value(),
                });

            let Some(pattern) = pattern else {
                self.i18n
                    .missing
                    .borrow_mut()
                    .insert(format!("{}: {id}", language.code));
                continue;
            };

            let mut errors = Vec::new();
            let text = bundle.format_pattern(pattern, Some(&fluent_args), &mut errors);
            for error in errors {
                self.i18n
                    .missing
                    .borrow_mut()
                    .insert(format!("{}: {id}: {error:?}", language.code));
            }
            return text.into_owned();
        }

        // Not even the source language has it. Showing the id makes the bug
        // visible on screen instead of rendering nothing.
        id.to_owned()
    }
}
