From Stdlib Require Import List.
Import ListNotations.

Record Rule := { rule_id : nat; required : nat }.

Definition Evidence := nat -> bool.

Inductive Verdict := VERIFIED | UNVERIFIED | UNKNOWN.

Record UnverifiedCertificate :=
{ cert_rule_id : nat; cert_predicate : nat }.

Definition Eval (policy : list Rule) (e : Evidence)
  : Verdict * UnverifiedCertificate :=
  match policy with
  | nil => (UNKNOWN, {| cert_rule_id := 0; cert_predicate := 0 |})
  | {| rule_id := rid; required := p |} :: _ =>
      (if e p then VERIFIED else UNVERIFIED,
       {| cert_rule_id := rid; cert_predicate := p |})
  end.

Theorem unverified_cert_soundness :
  forall policy e c,
    Eval policy e = (UNVERIFIED, c) ->
    exists r p, In r policy /\
      rule_id r = cert_rule_id c /\ required r = p /\ e p = false.
Proof.
  intros policy e c; induction policy as [|r policy IH]; simpl; intros H.
  - discriminate H.
  - destruct r as [rid p]. destruct (e p) eqn:Hp; try discriminate H.
    inversion H. exists {| rule_id := rid; required := p |}, p. simpl; firstorder.
Qed.