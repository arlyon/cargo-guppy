// Copyright (c) The cargo-guppy Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{errors::TripleParseError, Platform};
use cfg_expr::{
    expr::TargetMatcher,
    target_lexicon,
    targets::{get_builtin_target_by_triple, TargetInfo},
    TargetPredicate,
};
use std::{borrow::Cow, cmp::Ordering, hash, str::FromStr};

/// A single, specific target, uniquely identified by a triple.
///
/// A `Triple` may be constructed through `new` or the `FromStr` implementation.
///
/// Every [`Platform`](crate::Platform) has one of these, and an evaluation
/// [`TargetSpec`](crate::TargetSpec) may be backed by one of these as well.
///
/// # Examples
///
/// ```
/// use target_spec::Triple;
///
/// // Parse a simple target.
/// let target = Triple::new("x86_64-unknown-linux-gnu").unwrap();
/// // This is not a valid triple.
/// let err = Triple::new("cannot-be-known").unwrap_err();
/// ```
#[derive(Clone, Debug)]
pub struct Triple {
    inner: TripleInner,
}

impl Triple {
    /// Creates a new `Triple` from a triple string.
    pub fn new(triple_str: impl Into<Cow<'static, str>>) -> Result<Self, TripleParseError> {
        let inner = TripleInner::new(triple_str.into())?;
        Ok(Self { inner })
    }

    /// Returns the string corresponding to this triple.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    /// Evaluates this triple against the given platform.
    ///
    /// This simply compares `self` against the `Triple` the platform is based on, ignoring
    /// target features and flags.
    #[inline]
    pub fn eval(&self, platform: &Platform) -> bool {
        self == platform.triple()
    }

    // Use cfg-expr's target matcher.
    #[inline]
    pub(crate) fn matches(&self, tp: &TargetPredicate) -> bool {
        self.inner.matches(tp)
    }
}

impl FromStr for Triple {
    type Err = TripleParseError;

    fn from_str(triple_str: &str) -> Result<Self, Self::Err> {
        let inner = TripleInner::from_borrowed_str(triple_str)?;
        Ok(Self { inner })
    }
}

/// Inner representation of a triple.
#[derive(Clone, Debug)]
enum TripleInner {
    /// Prefer the builtin representation as it's more accurate.
    Builtin(&'static TargetInfo),
    /// Fall back to the lexicon representation.
    Lexicon {
        triple_str: Cow<'static, str>,
        lexicon_triple: target_lexicon::Triple,
    },
}

impl TripleInner {
    fn new(triple_str: Cow<'static, str>) -> Result<Self, TripleParseError> {
        // First try getting the builtin.
        if let Some(target_info) = get_builtin_target_by_triple(&triple_str) {
            return Ok(TripleInner::Builtin(target_info));
        }

        // Next, try getting the lexicon representation.
        match triple_str.parse::<target_lexicon::Triple>() {
            Ok(lexicon_triple) => Ok(TripleInner::Lexicon {
                triple_str,
                lexicon_triple,
            }),
            Err(lexicon_err) => Err(TripleParseError::new(triple_str, lexicon_err)),
        }
    }

    fn from_borrowed_str(triple_str: &str) -> Result<Self, TripleParseError> {
        // First try getting the builtin.
        if let Some(target_info) = get_builtin_target_by_triple(triple_str) {
            return Ok(TripleInner::Builtin(target_info));
        }

        // Next, try getting the lexicon representation.
        match triple_str.parse::<target_lexicon::Triple>() {
            Ok(lexicon_triple) => Ok(TripleInner::Lexicon {
                triple_str: triple_str.to_owned().into(),
                lexicon_triple,
            }),
            Err(lexicon_err) => Err(TripleParseError::new(
                triple_str.to_owned().into(),
                lexicon_err,
            )),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            TripleInner::Builtin(target_info) => target_info.triple.as_str(),
            TripleInner::Lexicon { triple_str, .. } => triple_str,
        }
    }

    fn matches(&self, tp: &TargetPredicate) -> bool {
        match self {
            TripleInner::Builtin(target_info) => target_info.matches(tp),
            TripleInner::Lexicon { lexicon_triple, .. } => lexicon_triple.matches(tp),
        }
    }
}

// ---
// Trait impls
//
// These impls only use the `triple_str`, which is valid because the triple is a pure
// function of the `triple_str`.
// ---

impl PartialEq for Triple {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl Eq for Triple {}

impl PartialOrd for Triple {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Ord for Triple {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl hash::Hash for Triple {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        hash::Hash::hash(self.as_str(), state);
    }
}

#[cfg(test)]
mod tests {
    use self::target_lexicon::*;
    use super::*;

    #[test]
    fn test_parse() {
        let target =
            super::Triple::new("x86_64-pc-darwin").expect("this triple is known to target-lexicon");

        let expected_triple = target_lexicon::Triple {
            architecture: Architecture::X86_64,
            vendor: Vendor::Pc,
            operating_system: OperatingSystem::Darwin,
            environment: Environment::Unknown,
            binary_format: BinaryFormat::Macho,
        };

        let actual_triple = match target.inner {
            TripleInner::Lexicon { lexicon_triple, .. } => lexicon_triple,
            TripleInner::Builtin(_) => {
                panic!("should not have been able to parse x86_64-pc-darwin as a builtin");
            }
        };
        assert_eq!(
            actual_triple, expected_triple,
            "lexicon triple matched correctly"
        );
    }
}
