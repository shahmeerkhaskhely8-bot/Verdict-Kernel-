#![no_std]
#![forbid(unsafe_code)]

/// The verdict algebra specified and proved in `proof_kernel.v`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Verified,
    Unverified,
    Unknown,
}

impl Verdict {
    /// Merges two verdicts using the proven kernel algebra.
    ///
    /// `Unknown` is the identity and `Unverified` is fail-safe and absorbing.
    pub const fn merge(self, other: Self) -> Self {
        use Verdict::*;
        match (self, other) {
            (Unknown, verdict) | (verdict, Unknown) => verdict,
            (Verified, Verified) => Verified,
            (Verified, Unverified) | (Unverified, Verified) | (Unverified, Unverified) => {
                Unverified
            }
        }
    }
}

/// Evaluates the integer policy specified by `evaluate_policy` in Coq.
pub const fn evaluate_policy(value: i32) -> Verdict {
    if value > 0 {
        Verdict::Verified
    } else {
        Verdict::Unverified
    }
}

/// A zero-dependency, fixed-capacity policy engine.
pub struct PolicyEngine<T, const MAX_RULES: usize> {
    rules: [Option<fn(&T) -> Verdict>; MAX_RULES],
    len: usize,
}

impl<T, const MAX_RULES: usize> PolicyEngine<T, MAX_RULES> {
    /// Creates an empty policy engine.
    pub const fn new() -> Self {
        Self {
            rules: [None; MAX_RULES],
            len: 0,
        }
    }

    /// Adds one pure policy rule without allocating.
    pub fn add_rule(&mut self, rule: fn(&T) -> Verdict) -> Result<(), fn(&T) -> Verdict> {
        if self.len == MAX_RULES {
            return Err(rule);
        }
        self.rules[self.len] = Some(rule);
        self.len += 1;
        Ok(())
    }

    /// Returns the number of installed rules.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns the maximum number of rules.
    pub const fn capacity(&self) -> usize {
        MAX_RULES
    }

    /// Returns whether no rules are installed.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Evaluates every rule and folds the results with [`Verdict::merge`].
    pub fn evaluate(&self, input: &T) -> Verdict {
        let mut result = Verdict::Unknown;
        for rule in self.rules[..self.len].iter().flatten() {
            result = result.merge(rule(input));
            if result == Verdict::Unverified {
                return Verdict::Unverified;
            }
        }
        result
    }
}

impl<T, const MAX_RULES: usize> Default for PolicyEngine<T, MAX_RULES> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{evaluate_policy, PolicyEngine, Verdict};
    use Verdict::{Unknown, Unverified, Verified};

    fn kernel_rule(value: &i32) -> Verdict {
        evaluate_policy(*value)
    }

    fn always_verified(_: &i32) -> Verdict {
        Verified
    }

    fn always_unverified(_: &i32) -> Verdict {
        Unverified
    }

    #[test]
    fn empty_engine_is_unknown() {
        let engine = PolicyEngine::<i32, 2>::new();
        assert_eq!(engine.evaluate(&1), Unknown);
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

    #[test]
    fn engine_evaluates_kernel_rule() {
        let mut engine = PolicyEngine::<i32, 1>::new();
        assert_eq!(engine.add_rule(kernel_rule), Ok(()));
        assert_eq!(engine.evaluate(&7), Verified);
        assert_eq!(engine.evaluate(&0), Unverified);
    }

    #[test]
    fn engine_folds_all_verified_rules() {
        let mut engine = PolicyEngine::<i32, 2>::new();
        assert_eq!(engine.add_rule(always_verified), Ok(()));
        assert_eq!(engine.add_rule(always_verified), Ok(()));
        assert_eq!(engine.evaluate(&0), Verified);
    }

    #[test]
    fn one_unverified_rule_forces_unverified() {
        let mut engine = PolicyEngine::<i32, 2>::new();
        assert_eq!(engine.add_rule(always_verified), Ok(()));
        assert_eq!(engine.add_rule(always_unverified), Ok(()));
        assert_eq!(engine.evaluate(&0), Unverified);
    }

    #[test]
    fn unverified_rule_is_absorbing_regardless_of_order() {
        let mut first = PolicyEngine::<i32, 2>::new();
        assert_eq!(first.add_rule(always_unverified), Ok(()));
        assert_eq!(first.add_rule(always_verified), Ok(()));

        let mut second = PolicyEngine::<i32, 2>::new();
        assert_eq!(second.add_rule(always_verified), Ok(()));
        assert_eq!(second.add_rule(always_unverified), Ok(()));

        assert_eq!(first.evaluate(&0), Unverified);
        assert_eq!(second.evaluate(&0), Unverified);
    }

    #[test]
    fn capacity_is_enforced_without_mutation() {
        let mut engine = PolicyEngine::<i32, 1>::new();
        assert_eq!(engine.add_rule(always_verified), Ok(()));
        assert!(engine.add_rule(always_unverified).is_err());
        assert_eq!(engine.len(), 1);
        assert_eq!(engine.capacity(), 1);
    }

    #[test]
    fn merge_is_commutative_and_associative() {
        let all = [Verified, Unverified, Unknown];
        for left in all {
            for middle in all {
                for right in all {
                    assert_eq!(left.merge(middle), middle.merge(left));
                    assert_eq!(
                        left.merge(middle).merge(right),
                        left.merge(middle.merge(right))
                    );
                }
            }
        }
    }
}
