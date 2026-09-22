(** * Epistemic Verdict Algebra

    A 5-state epistemic verdict lattice used to combine verification
    outcomes.  The information order is the diamond:

                 Contradicted
                /            \
           Verified        Unverified
                \            /
             PartiallyVerified
                     |
                  Unknown

    [merge] is the join (least upper bound) of this lattice:
    - [Unknown] is the identity (no information),
    - [Contradicted] is absorbing (conflict dominates),
    - [Verified] and [Unverified] are incompatible and join to
      [Contradicted].
*)

Inductive Verdict : Type :=
  | Unverified
  | PartiallyVerified
  | Verified
  | Unknown
  | Contradicted.

(** Combine two verdicts by taking their join in the epistemic lattice. *)
Definition merge (a b : Verdict) : Verdict :=
  match a, b with
  | Contradicted, _ => Contradicted
  | _, Contradicted => Contradicted
  | Unknown, x => x
  | x, Unknown => x
  | Verified, Verified => Verified
  | Unverified, Unverified => Unverified
  | PartiallyVerified, PartiallyVerified => PartiallyVerified
  | Verified, PartiallyVerified => Verified
  | PartiallyVerified, Verified => Verified
  | Unverified, PartiallyVerified => Unverified
  | PartiallyVerified, Unverified => Unverified
  | Verified, Unverified => Contradicted
  | Unverified, Verified => Contradicted
  end.

(** Commutativity of [merge]. *)
Theorem merge_comm : forall a b : Verdict, merge a b = merge b a.
Proof.
  intros a b.
  destruct a, b; reflexivity.
Qed.

(** Associativity of [merge]. *)
Theorem merge_assoc :
  forall a b c : Verdict, merge a (merge b c) = merge (merge a b) c.
Proof.
  intros a b c.
  destruct a, b, c; reflexivity.
Qed.
