From Stdlib Require Import ZArith.

Inductive Verdict : Type :=
| Verified
| Unverified
| Unknown.

Definition evaluate_policy (n : Z) : Verdict :=
  if Z.gtb n 0 then Verified else Unverified.

Definition merge_verdict (v1 v2 : Verdict) : Verdict :=
  match v1, v2 with
  | Unknown, v => v
  | Verified, Unknown => Verified
  | Verified, Verified => Verified
  | Verified, Unverified => Verified
  | Unverified, Unknown => Unverified
  | Unverified, Verified => Verified
  | Unverified, Unverified => Unverified
  end.

Theorem merge_commutative :
  forall v1 v2 : Verdict,
    merge_verdict v1 v2 = merge_verdict v2 v1.
Proof.
  intros v1 v2.
  destruct v1, v2; reflexivity.
Qed.

Theorem merge_associative :
  forall v1 v2 v3 : Verdict,
    merge_verdict (merge_verdict v1 v2) v3 =
    merge_verdict v1 (merge_verdict v2 v3).
Proof.
  intros v1 v2 v3.
  destruct v1, v2, v3; reflexivity.
Qed.

Theorem evaluate_soundness :
  forall n : Z,
    Z.gtb n 0 = true -> evaluate_policy n = Verified.
Proof.
  intros n H.
  unfold evaluate_policy.
  rewrite H.
  reflexivity.
Qed.

Theorem evaluate_completeness :
  forall n : Z,
    Z.gtb n 0 = false -> evaluate_policy n = Unverified.
Proof.
  intros n H.
  unfold evaluate_policy.
  rewrite H.
  reflexivity.
Qed.