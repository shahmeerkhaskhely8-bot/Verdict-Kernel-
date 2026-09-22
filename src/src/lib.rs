//! Epistemic verdict algebra matching `Verdict.v`.
//!
//! The five constructors mirror Rocq `Inductive Verdict` in order:
//! `Unverified`, `PartiallyVerified`, `Verified`, `Unknown`, `Contradicted`.
//!
//! Diamond lattice (join = [`Verdict::merge`]):
//!
//! ```text
//!              Contradicted
//!             /            \
//!        Verified        Unverified
//!             \            /
//!          PartiallyVerified
//!                  |
//!               Unknown
//! ```
//!
//! Rocq (`Verdict.v`) proves that [`Verdict::merge`] is a commutative and
//! associative join: theorems `merge_comm` and `merge_assoc`. The unit tests
//! below exhaustively re-check those same laws in Rust, including when verdicts
//! are folded through [`EvidenceLog::append`].

#![no_std]
#![forbid(unsafe_code)]

/// Five-state epistemic verdict.
///
/// Constructors correspond 1:1 with Rocq `Verdict` in `Verdict.v`:
/// `Unverified`, `PartiallyVerified`, `Verified`, `Unknown`, `Contradicted`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Unverified,
    PartiallyVerified,
    Verified,
    Unknown,
    Contradicted,
}

impl Verdict {
    /// Join of two verdicts; identical to Rocq `merge` in `Verdict.v`.
    ///
    /// Proven there as commutative (`merge_comm`) and associative
    /// (`merge_assoc`); see also the `merge_comm` / `merge_assoc` unit tests.
    pub const fn merge(self, other: Self) -> Self {
        use Verdict::*;
        match (self, other) {
            (Contradicted, _) | (_, Contradicted) => Contradicted,
            (Unknown, x) | (x, Unknown) => x,
            (Verified, Verified) => Verified,
            (Unverified, Unverified) => Unverified,
            (PartiallyVerified, PartiallyVerified) => PartiallyVerified,
            (Verified, PartiallyVerified) | (PartiallyVerified, Verified) => Verified,
            (Unverified, PartiallyVerified) | (PartiallyVerified, Unverified) => Unverified,
            (Verified, Unverified) | (Unverified, Verified) => Contradicted,
        }
    }
}

/// Maximum number of verdicts an [`EvidenceLog`] can record without allocation.
pub const EVIDENCE_LOG_CAPACITY: usize = 16;

/// Fixed-capacity log of verdicts with a running join state.
///
/// Stores up to [`EVIDENCE_LOG_CAPACITY`] entries in a stack array. The running
/// state starts at [`Verdict::Unknown`] (the merge identity) and is updated on
/// each successful [`EvidenceLog::append`] via [`Verdict::merge`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceLog {
    entries: [Verdict; EVIDENCE_LOG_CAPACITY],
    len: usize,
    state: Verdict,
}

/// A fixed-capacity, stack-backed array.
///
/// `BoundedArray` never allocates. The backing array is initialized when the
/// value is created, so its implementation does not require `unsafe` code.
/// The `len` field identifies the initialized prefix that is exposed by
/// [`BoundedArray::as_slice`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundedArray<T: Copy, const CAPACITY: usize> {
    values: [T; CAPACITY],
    len: usize,
}

impl<T: Copy, const CAPACITY: usize> BoundedArray<T, CAPACITY> {
    /// Creates an empty array whose unused slots contain `fill`.
    pub const fn new(fill: T) -> Self {
        Self {
            values: [fill; CAPACITY],
            len: 0,
        }
    }

    /// Creates a full bounded array from an ordinary fixed-size array.
    pub const fn from_array(values: [T; CAPACITY]) -> Self {
        Self {
            values,
            len: CAPACITY,
        }
    }

    /// Number of elements currently stored.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Maximum number of elements this array can store.
    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    /// Whether the array contains no elements.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Whether no further elements can be appended.
    pub const fn is_full(&self) -> bool {
        self.len == CAPACITY
    }

    /// Returns the stored prefix as a slice.
    pub fn as_slice(&self) -> &[T] {
        &self.values[..self.len]
    }

    /// Returns an element if `index` is in the stored prefix.
    pub fn get(&self, index: usize) -> Option<T> {
        if index < self.len {
            Some(self.values[index])
        } else {
            None
        }
    }

    /// Appends an element, returning it unchanged if the array is full.
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.is_full() {
            return Err(value);
        }

        self.values[self.len] = value;
        self.len += 1;
        Ok(())
    }
}

/// Allocation-free policy evaluation primitives.
pub mod policy {
    use super::{BoundedArray, Verdict};

    /// A policy rule evaluates an input without owning or allocating data.
    pub type Rule<T> = fn(&T) -> Verdict;

    /// A bounded collection of rules evaluated in insertion order.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Policy<T, const MAX_RULES: usize> {
        rules: BoundedArray<Rule<T>, MAX_RULES>,
    }

    impl<T, const MAX_RULES: usize> Policy<T, MAX_RULES> {
        /// Creates an empty policy.
        pub const fn new() -> Self {
            Self {
                rules: BoundedArray::new(Self::empty_rule),
            }
        }

        const fn empty_rule(_: &T) -> Verdict {
            Verdict::Unknown
        }

        /// Creates a policy containing every rule in `rules`.
        pub const fn from_rules(rules: BoundedArray<Rule<T>, MAX_RULES>) -> Self {
            Self { rules }
        }

        /// Adds a rule, returning the rule if the policy is already full.
        pub fn add_rule(&mut self, rule: Rule<T>) -> Result<(), Rule<T>> {
            self.rules.push(rule)
        }

        /// Number of rules in the policy.
        pub const fn len(&self) -> usize {
            self.rules.len()
        }

        /// Maximum number of rules supported by this policy.
        pub const fn capacity(&self) -> usize {
            self.rules.capacity()
        }

        /// Whether the policy has no rules.
        pub const fn is_empty(&self) -> bool {
            self.rules.is_empty()
        }

        /// Whether the policy cannot accept another rule.
        pub const fn is_full(&self) -> bool {
            self.rules.is_full()
        }

        /// Rules in insertion order.
        pub fn rules(&self) -> &[Rule<T>] {
            self.rules.as_slice()
        }

        /// Evaluates all rules and joins their verdicts.
        ///
        /// The identity is [`Verdict::Unknown`], so an empty policy is
        /// conservative and produces `Unknown`. A `Contradicted` result is
        /// retained by `Verdict::merge` and cannot be hidden by later rules.
        pub fn evaluate(&self, input: &T) -> Verdict {
            let mut result = Verdict::Unknown;
            for &rule in self.rules.as_slice() {
                result = result.merge(rule(input));
            }
            result
        }
    }

    impl<T, const MAX_RULES: usize> Default for Policy<T, MAX_RULES> {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Stateless policy evaluator suitable for embedded and other `no_std`
    /// environments.
    pub struct PolicyEngine;

    impl PolicyEngine {
        /// Evaluates a policy without allocating or retaining input state.
        pub fn evaluate<T, const MAX_RULES: usize>(
            policy: &Policy<T, MAX_RULES>,
            input: &T,
        ) -> Verdict {
            policy.evaluate(input)
        }
    }
}

pub use policy::{Policy, PolicyEngine, Rule};

impl EvidenceLog {
    /// Empty log; running state is [`Verdict::Unknown`].
    pub const fn new() -> Self {
        Self {
            entries: [Verdict::Unknown; EVIDENCE_LOG_CAPACITY],
            len: 0,
            state: Verdict::Unknown,
        }
    }

    /// Number of recorded verdicts.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Fixed buffer capacity (always [`EVIDENCE_LOG_CAPACITY`]).
    pub const fn capacity(&self) -> usize {
        EVIDENCE_LOG_CAPACITY
    }

    /// Whether no verdicts have been appended.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Whether the fixed buffer is full.
    pub const fn is_full(&self) -> bool {
        self.len == EVIDENCE_LOG_CAPACITY
    }

    /// Running join of all appended verdicts.
    pub const fn state(&self) -> Verdict {
        self.state
    }

    /// Recorded verdicts in append order (prefix of the internal buffer).
    pub fn entries(&self) -> &[Verdict] {
        &self.entries[..self.len]
    }

    /// Append `verdict`, merging it into the running state with [`Verdict::merge`].
    ///
    /// Returns `Ok(state)` on success, or `Err(verdict)` if the buffer is full
    /// (nothing is mutated).
    pub fn append(&mut self, verdict: Verdict) -> Result<Verdict, Verdict> {
        if self.is_full() {
            return Err(verdict);
        }
        self.entries[self.len] = verdict;
        self.len += 1;
        self.state = self.state.merge(verdict);
        Ok(self.state)
    }
}

impl Default for EvidenceLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BoundedArray, EvidenceLog, Policy, PolicyEngine, Verdict, EVIDENCE_LOG_CAPACITY,
        Verdict::*,
    };

    const ALL: [Verdict; 5] = [
        Unverified,
        PartiallyVerified,
        Verified,
        Unknown,
        Contradicted,
    ];

    /// Exhaustive check of Rocq theorem `merge_comm` from `Verdict.v`.
    #[test]
    fn merge_comm() {
        for &a in &ALL {
            for &b in &ALL {
                assert_eq!(a.merge(b), b.merge(a), "merge_comm({a:?}, {b:?})");
            }
        }
    }

    /// Exhaustive check of Rocq theorem `merge_assoc` from `Verdict.v`.
    #[test]
    fn merge_assoc() {
        for &a in &ALL {
            for &b in &ALL {
                for &c in &ALL {
                    assert_eq!(
                        a.merge(b.merge(c)),
                        a.merge(b).merge(c),
                        "merge_assoc({a:?}, {b:?}, {c:?})"
                    );
                }
            }
        }
    }

    /// Sequential append left-folds with `merge`; final state matches any
    /// parenthesization (Rocq `merge_assoc`).
    #[test]
    fn append_preserves_assoc() {
        for &a in &ALL {
            for &b in &ALL {
                for &c in &ALL {
                    let mut log = EvidenceLog::new();
                    assert!(log.is_empty());
                    assert_eq!(log.capacity(), EVIDENCE_LOG_CAPACITY);
                    assert_eq!(log.state(), Unknown);

                    assert_eq!(log.append(a), Ok(a));
                    assert_eq!(log.append(b), Ok(a.merge(b)));
                    assert_eq!(log.append(c), Ok(a.merge(b).merge(c)));

                    let left_assoc = a.merge(b).merge(c);
                    let right_assoc = a.merge(b.merge(c));
                    assert_eq!(left_assoc, right_assoc, "assoc({a:?},{b:?},{c:?})");
                    assert_eq!(
                        log.state(),
                        left_assoc,
                        "append log state vs assoc({a:?},{b:?},{c:?})"
                    );
                    assert_eq!(log.entries(), &[a, b, c]);
                    assert!(!log.is_empty());
                    assert!(!log.is_full());
                }
            }
        }
    }

    /// Append order does not change the running join (Rocq `merge_comm` +
    /// `merge_assoc`): every permutation of a pair and of a triple yields the
    /// same final state.
    #[test]
    fn append_preserves_comm() {
        for &a in &ALL {
            for &b in &ALL {
                let mut ab = EvidenceLog::new();
                ab.append(a).unwrap();
                ab.append(b).unwrap();

                let mut ba = EvidenceLog::new();
                ba.append(b).unwrap();
                ba.append(a).unwrap();

                assert_eq!(ab.state(), ba.state(), "append_comm pair ({a:?}, {b:?})");
                assert_eq!(ab.state(), a.merge(b));
            }
        }

        for &a in &ALL {
            for &b in &ALL {
                for &c in &ALL {
                    let perms = [
                        [a, b, c],
                        [a, c, b],
                        [b, a, c],
                        [b, c, a],
                        [c, a, b],
                        [c, b, a],
                    ];
                    let expected = a.merge(b).merge(c);
                    for perm in perms {
                        let mut log = EvidenceLog::new();
                        for v in perm {
                            log.append(v).unwrap();
                        }
                        assert_eq!(log.state(), expected, "append_comm triple perm {perm:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn append_respects_fixed_capacity() {
        let mut log = EvidenceLog::new();
        assert!(log.is_empty());
        assert_eq!(log.capacity(), EVIDENCE_LOG_CAPACITY);
        assert_eq!(log.state(), Unknown);

        for i in 0..log.capacity() {
            assert!(log.append(Verified).is_ok(), "slot {i}");
        }
        assert!(log.is_full());
        assert!(!log.is_empty());
        assert_eq!(log.append(Unverified), Err(Unverified));
        assert_eq!(log.len(), log.capacity());
        assert_eq!(log.state(), Verified);
    }

    fn allow_positive(value: &i32) -> Verdict {
        if *value > 0 {
            Verified
        } else {
            Unverified
        }
    }

    fn deny_negative(value: &i32) -> Verdict {
        if *value < 0 {
            Contradicted
        } else {
            Unknown
        }
    }

    fn partially_verified(_: &i32) -> Verdict {
        PartiallyVerified
    }

    #[test]
    fn bounded_array_never_exceeds_capacity() {
        let mut values = BoundedArray::<u8, 2>::new(0);
        assert_eq!(values.capacity(), 2);
        assert!(values.is_empty());
        assert_eq!(values.push(7), Ok(()));
        assert_eq!(values.push(9), Ok(()));
        assert!(values.is_full());
        assert_eq!(values.push(11), Err(11));
        assert_eq!(values.as_slice(), &[7, 9]);
        assert_eq!(values.get(0), Some(7));
        assert_eq!(values.get(2), None);
    }

    #[test]
    fn empty_policy_is_unknown() {
        let policy = Policy::<i32, 2>::new();
        assert!(policy.is_empty());
        assert_eq!(policy.evaluate(&42), Unknown);
        assert_eq!(PolicyEngine::evaluate(&policy, &42), Unknown);
    }

    #[test]
    fn policy_evaluation_merges_all_rule_verdicts() {
        let mut policy = Policy::<i32, 3>::new();
        assert_eq!(policy.add_rule(allow_positive), Ok(()));
        assert_eq!(policy.add_rule(partially_verified), Ok(()));
        assert_eq!(policy.len(), 2);
        assert_eq!(policy.evaluate(&10), Verified);
        assert_eq!(policy.evaluate(&0), Unverified);
    }

    #[test]
    fn contradiction_is_not_hidden_by_later_rules() {
        let mut policy = Policy::<i32, 3>::new();
        assert_eq!(policy.add_rule(deny_negative), Ok(()));
        assert_eq!(policy.add_rule(allow_positive), Ok(()));
        assert_eq!(policy.evaluate(&-4), Contradicted);
    }

    #[test]
    fn policy_capacity_failure_does_not_mutate_policy() {
        let mut policy = Policy::<i32, 1>::new();
        assert_eq!(policy.add_rule(allow_positive), Ok(()));
        assert!(policy.is_full());
        assert!(policy.add_rule(deny_negative).is_err());
        assert_eq!(policy.len(), 1);
        assert_eq!(policy.evaluate(&-1), Unverified);
    }
}
