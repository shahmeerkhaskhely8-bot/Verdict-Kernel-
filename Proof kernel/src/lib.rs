#![no_std]
#![forbid(unsafe_code)]

/// The three-state verdict type specified by `proof_kernel.v`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Verified,
    Unverified,
    Unknown,
}

impl Verdict {
    /// Algebraic merge corresponding exactly to Coq's `merge_verdict`.
    pub const fn merge(self, other: Self) -> Self {
        use Verdict::*;
        match (self, other) {
            (Unknown, verdict) | (verdict, Unknown) => verdict,
            (Verified, Verified) => Verified,
            (Unverified, Unverified) => Unverified,
            (Verified, Unverified) | (Unverified, Verified) => Verified,
        }
    }
}

/// Evaluates the integer policy corresponding to Coq's `evaluate_policy`.
pub const fn evaluate_policy(value: i32) -> Verdict {
    if value > 0 {
        Verdict::Verified
    } else {
        Verdict::Unverified
    }
}

pub const EVIDENCE_LOG_CAPACITY: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceLog {
    entries: [Verdict; EVIDENCE_LOG_CAPACITY],
    len: usize,
    state: Verdict,
}

impl EvidenceLog {
    pub const fn new() -> Self {
        Self {
            entries: [Verdict::Unknown; EVIDENCE_LOG_CAPACITY],
            len: 0,
            state: Verdict::Unknown,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn capacity(&self) -> usize {
        EVIDENCE_LOG_CAPACITY
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn is_full(&self) -> bool {
        self.len == EVIDENCE_LOG_CAPACITY
    }

    pub const fn state(&self) -> Verdict {
        self.state
    }

    pub fn entries(&self) -> &[Verdict] {
        &self.entries[..self.len]
    }

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundedArray<T: Copy, const CAPACITY: usize> {
    values: [T; CAPACITY],
    len: usize,
}

impl<T: Copy, const CAPACITY: usize> BoundedArray<T, CAPACITY> {
    pub const fn new(fill: T) -> Self {
        Self {
            values: [fill; CAPACITY],
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn is_full(&self) -> bool {
        self.len == CAPACITY
    }

    pub fn as_slice(&self) -> &[T] {
        &self.values[..self.len]
    }

    pub fn get(&self, index: usize) -> Option<T> {
        self.as_slice().get(index).copied()
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.is_full() {
            return Err(value);
        }
        self.values[self.len] = value;
        self.len += 1;
        Ok(())
    }
}

pub type Rule<T> = fn(&T) -> Verdict;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy<T, const MAX_RULES: usize> {
    rules: BoundedArray<Rule<T>, MAX_RULES>,
}

impl<T, const MAX_RULES: usize> Policy<T, MAX_RULES> {
    pub const fn new() -> Self {
        Self {
            rules: BoundedArray::new(Self::empty_rule),
        }
    }

    const fn empty_rule(_: &T) -> Verdict {
        Verdict::Unknown
    }

    pub fn add_rule(&mut self, rule: Rule<T>) -> Result<(), Rule<T>> {
        self.rules.push(rule)
    }

    pub const fn len(&self) -> usize {
        self.rules.len()
    }

    pub const fn capacity(&self) -> usize {
        self.rules.capacity()
    }

    pub const fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub const fn is_full(&self) -> bool {
        self.rules.is_full()
    }

    pub fn evaluate(&self, input: &T) -> Verdict {
        self.rules
            .as_slice()
            .iter()
            .fold(Verdict::Unknown, |state, rule| state.merge(rule(input)))
    }
}

impl<T, const MAX_RULES: usize> Default for Policy<T, MAX_RULES> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PolicyEngine;

impl PolicyEngine {
    pub fn evaluate<T, const MAX_RULES: usize>(
        policy: &Policy<T, MAX_RULES>,
        input: &T,
    ) -> Verdict {
        policy.evaluate(input)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_policy, BoundedArray, EvidenceLog, Policy, PolicyEngine, Verdict,
        EVIDENCE_LOG_CAPACITY,
    };
    use Verdict::{Unknown, Unverified, Verified};

    const ALL: [Verdict; 3] = [Verified, Unverified, Unknown];

    #[test]
    fn merge_is_commutative() {
        for left in ALL {
            for right in ALL {
                assert_eq!(left.merge(right), right.merge(left));
            }
        }
    }

    #[test]
    fn merge_is_associative() {
        for left in ALL {
            for middle in ALL {
                for right in ALL {
                    assert_eq!(
                        left.merge(middle).merge(right),
                        left.merge(middle.merge(right))
                    );
                }
            }
        }
    }

    #[test]
    fn merge_has_unknown_as_identity() {
        for verdict in ALL {
            assert_eq!(Unknown.merge(verdict), verdict);
            assert_eq!(verdict.merge(Unknown), verdict);
        }
    }

    #[test]
    fn verified_takes_precedence_over_unverified() {
        assert_eq!(Verified.merge(Unverified), Verified);
        assert_eq!(Unverified.merge(Verified), Verified);
    }

    #[test]
    fn positive_policy_input_is_verified() {
        assert_eq!(evaluate_policy(1), Verified);
        assert_eq!(evaluate_policy(i32::MAX), Verified);
    }

    #[test]
    fn non_positive_policy_input_is_unverified() {
        assert_eq!(evaluate_policy(0), Unverified);
        assert_eq!(evaluate_policy(-1), Unverified);
        assert_eq!(evaluate_policy(i32::MIN), Unverified);
    }

    fn kernel_rule(value: &i32) -> Verdict {
        evaluate_policy(*value)
    }

    #[test]
    fn policy_engine_matches_kernel_evaluation() {
        let mut policy = Policy::<i32, 1>::new();
        assert_eq!(policy.add_rule(kernel_rule), Ok(()));
        assert_eq!(PolicyEngine::evaluate(&policy, &7), Verified);
        assert_eq!(PolicyEngine::evaluate(&policy, &0), Unverified);
    }

    #[test]
    fn empty_policy_is_unknown() {
        let policy = Policy::<i32, 1>::new();
        assert_eq!(policy.evaluate(&7), Unknown);
    }

    #[test]
    fn policy_merges_rules_using_kernel_operator() {
        fn verified(_: &i32) -> Verdict {
            Verified
        }
        fn unverified(_: &i32) -> Verdict {
            Unverified
        }
        let mut policy = Policy::<i32, 2>::new();
        assert_eq!(policy.add_rule(verified), Ok(()));
        assert_eq!(policy.add_rule(unverified), Ok(()));
        assert_eq!(policy.evaluate(&0), Verified);
    }

    #[test]
    fn bounded_array_respects_capacity() {
        let mut values = BoundedArray::<u8, 2>::new(0);
        assert_eq!(values.push(1), Ok(()));
        assert_eq!(values.push(2), Ok(()));
        assert_eq!(values.push(3), Err(3));
        assert_eq!(values.as_slice(), &[1, 2]);
        assert_eq!(values.get(2), None);
    }

    #[test]
    fn evidence_log_merges_with_unknown_identity() {
        let mut log = EvidenceLog::new();
        assert_eq!(log.append(Verified), Ok(Verified));
        assert_eq!(log.append(Unverified), Ok(Verified));
        assert_eq!(log.state(), Verified);
    }

    #[test]
    fn evidence_log_has_fixed_capacity() {
        let mut log = EvidenceLog::new();
        for _ in 0..EVIDENCE_LOG_CAPACITY {
            assert!(log.append(Verified).is_ok());
        }
        assert!(log.is_full());
        assert_eq!(log.append(Unverified), Err(Unverified));
        assert_eq!(log.len(), EVIDENCE_LOG_CAPACITY);
    }
}
