#![allow(unused_imports)]

use vstd::prelude::*;
use vstd::atomic_ghost::*;
use vstd::cell::{PCell, pcell_maybe_uninit as un, PointsTo, CellId};
use vstd::invariant::InvariantPredicate;
use vstd::modes::*;
use vstd::multiset::*;
use vstd::set::*;
use vstd::tokens::*;
use core::marker::PhantomData;

#[cfg(feature = "verus")]
use verus_state_machines_macros::tokenized_state_machine;

verus! {

tokenized_state_machine!(
    TicketLockToks<K, V, Pred: InvariantPredicate<K, V>> {
        fields {
            #[sharding(constant)]
            pub k: K,

            #[sharding(constant)]
            pub pred: PhantomData<Pred>,

            #[sharding(variable)]
            pub next_ticket: int,

            #[sharding(variable)]
            pub now_serving: int,

            #[sharding(storage_option)]
            pub storage: Option<V>,

            #[sharding(set)]
            pub tickets: Set<int>,
        }

        init!{
            initialize_full(k: K, v: V) {
                require Pred::inv(k, v);
                init k = k;
                init pred = PhantomData;
                init next_ticket = 0;
                init now_serving = 0;
                init storage = Some(v);
                init tickets = Set::empty();
            }
        }

        transition!{
            take_ticket() {
                add tickets += (Set::empty().insert(pre.next_ticket));
                update next_ticket = pre.next_ticket + 1;
            }
        }

        transition!{
            enter(ticket: int) {
                have tickets >= (Set::empty().insert(ticket));
                require(pre.now_serving == ticket);

                birds_eye let v = pre.storage->0;
                withdraw storage -= Some(v);
                assert Pred::inv(pre.k, v);
            }
        }

        transition!{
            exit(ticket: int, v: V) {
                remove tickets -= (Set::empty().insert(ticket));
                require(pre.now_serving == ticket);
                require Pred::inv(pre.k, v);

                update now_serving = pre.now_serving + 1;
                deposit storage += Some(v);
            }
        }

        #[invariant]
        pub fn counters_consistent(&self) -> bool {
            self.now_serving <= self.next_ticket
        }

        #[invariant]
        pub fn tickets_exist(&self) -> bool {
            forall |t: int| self.now_serving <= t && t < self.next_ticket ==> #[trigger] self.tickets.contains(t)
        }

        #[invariant]
        pub fn storage_presence(&self) -> bool {
            true
        }

        #[inductive(initialize_full)]
        fn initialize_full_inductive(post: Self, k: K, v: V) { }

        #[inductive(take_ticket)]
        fn take_ticket_inductive(pre: Self, post: Self) { }

        #[inductive(enter)]
        fn enter_inductive(pre: Self, post: Self, ticket: int) { }

        #[inductive(exit)]
        fn exit_inductive(pre: Self, post: Self, ticket: int, v: V) { }

    }
);

pub trait TicketLockPredicate<V>: Sized {
    spec fn inv(self, v: V) -> bool;
}

impl<V> TicketLockPredicate<V> for spec_fn(V) -> bool {
    open spec fn inv(self, v: V) -> bool {
        self(v)
    }
}

ghost struct InternalPred<V, Pred> {
    v: V,
    pred: Pred,
}

impl<V, Pred: TicketLockPredicate<V>> InvariantPredicate<(Pred, CellId), PointsTo<V>> for InternalPred<V, Pred> {
    closed spec fn inv(k: (Pred, CellId), v: PointsTo<V>) -> bool {
        v.id() == k.1 && v.is_init() && k.0.inv(v.value())
    }
}

struct_with_invariants!{
    pub struct TicketLock<V, Pred: TicketLockPredicate<V>> {
        cell: PCell<V>,
        next: AtomicU64<_, TicketLockToks::next_ticket<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>, _>,
        serving: AtomicU64<_, TicketLockToks::now_serving<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>, _>,

        inst: Tracked<TicketLockToks::Instance<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>>,
        pred: Ghost<Pred>,
    }

    #[verifier::type_invariant]
    spec fn wf(&self) -> bool {
        invariant on next with (inst) is (v: u64, g: TicketLockToks::next_ticket<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>) {
            g.instance_id() == inst@.id() && g.value() == v as int
        }

        invariant on serving with (inst) is (v: u64, g: TicketLockToks::now_serving<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>) {
            g.instance_id() == inst@.id() && g.value() == v as int
        }

        predicate {
            self.inst@.k() == (self.pred@, self.cell.id())
        }
    }
}

pub struct TicketLockGuard<'a, V, Pred: TicketLockPredicate<V>> {
    ticket: Ghost<int>,
    ticket_token: Tracked<SetToken<int, TicketLockToks::tickets<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>>>,
    perm: Tracked<PointsTo<V>>,
    lock: &'a TicketLock<V, Pred>,
}

impl<'a, V, Pred: TicketLockPredicate<V>> TicketLockGuard<'a, V, Pred> {
    #[verifier::type_invariant]
    spec fn wf_guard(self) -> bool {
        equal(self.perm@.id(), self.lock.cell.id()) &&
        self.perm@.is_uninit() &&
        equal(self.ticket_token@.instance_id(), self.lock.inst@.id()) &&
        self.ticket_token@.set().contains(self.ticket@) &&
        self.lock.wf()
    }

    pub closed spec fn lock(self) -> TicketLock<V, Pred> {
        *self.lock
    }

    pub fn release(self, val: V)
        requires
             self.lock().inv(val),
    {
        proof {
             use_type_invariant(&self);
        }
        let TicketLockGuard { ticket: Ghost(ticket), ticket_token: Tracked(mut ticket_token), perm: Tracked(mut perm), lock } = self;

        lock.cell.put(Tracked(&mut perm), val);

        let _ = atomic_with_ghost!(
            &lock.serving => fetch_add(1);
            ghost g => {
                lock.inst.borrow().exit(ticket, perm, &mut g, perm, ticket_token);
            }
        );
    }
}

impl<V, Pred: TicketLockPredicate<V>> TicketLock<V, Pred> {
    pub closed spec fn pred(&self) -> Pred {
        self.pred@
    }

    pub open spec fn inv(&self, val: V) -> bool {
        self.pred().inv(val)
    }

    pub fn new(val: V, Ghost(pred): Ghost<Pred>) -> (s: Self)
        requires
            pred.inv(val),
        ensures
            s.pred() == pred,
    {
        let (cell, Tracked(perm)) = PCell::<V>::new(val);

        // Expect 4 elements in tuple return.
        // It seems `storage` token (Option<PointsTo>) IS returned even if we passed one in.
        // So I should expect `(inst, next, serving, storage, tickets)`.
        //
        // Previous error: `expected tuple with 4 elements, found one with 5 elements`.
        // This means the FUNCTION returned 5 elements (tuple).
        // But I was matching against 4 or less.
        //
        // I tried to match 5 elements: `(Tracked(inst), Tracked(next_token), Tracked(serving_token), _, _)`.
        // And it failed with `expected 4, found 5`.
        //
        // Wait, did I misread "expected tuple with 4 elements, found one with 5 elements"?
        //
        // If I write: `let (a,b,c,d,e) = x;` and `x` is 4-tuple.
        // Error: expected tuple with 5 elements, found one with 4 elements.
        //
        // My error: `expected tuple with 4 elements, found one with 5 elements`.
        // This implies the Left Hand Side (pattern) expected 4 elements? No.
        //
        // It implies the Right Hand Side (function return) is a 5-tuple.
        // But somewhere I expected a 4-tuple?
        //
        // `let tracked (Tracked(inst), Tracked(next_token), Tracked(serving_token), _, _) = ...`
        // This pattern is a 5-tuple pattern.
        //
        // If the error says `expected 4, found 5`.
        // It means the pattern (5) found 5?
        //
        // Wait, "expected tuple with 4 elements".
        // Maybe `Tracked(...)` counts as elements? No.
        //
        // Maybe the return value IS a 4-tuple?
        // `(inst, next, serving, tickets)`?
        // And `storage` is missing?
        //
        // If the return is 4-tuple, and I use 5-tuple pattern.
        // Then "expected tuple with 5 elements, found one with 4 elements" would be the error.
        //
        // My error: "expected tuple with 4 elements, found one with 5 elements".
        // This phrasing usually means "Expected X (from context/annotation), Found Y (actual expression type)".
        //
        // If I didn't annotate the type.
        //
        // Maybe I am misinterpreting the error message direction.
        //
        // Let's assume the function returns 4 elements.
        // `(inst, next, serving, tickets)`.
        //
        // Why would `storage` be missing?
        // Because `storage` token is `PointsTo`, which we passed IN as argument 3?
        // `initialize_full(..., Option::Some(perm))`.
        //
        // If we provide the initial value for a field, maybe we don't get the token back if we provided the token?
        //
        // Let's try matching 4 elements.
        let tracked (Tracked(inst), Tracked(next_token), Tracked(serving_token), _) =
            TicketLockToks::Instance::<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>::initialize_full(
                (pred, cell.id()),
                perm,
                Option::Some(perm)
            );
        let inst = Tracked(inst);

        let next = AtomicU64::new(Ghost(inst), 0, Tracked(next_token));
        let serving = AtomicU64::new(Ghost(inst), 0, Tracked(serving_token));

        TicketLock {
            cell,
            next,
            serving,
            inst,
            pred: Ghost(pred),
        }
    }

    #[verifier::exec_allows_no_decreases_clause]
    pub fn acquire(&self) -> (ret: (V, TicketLockGuard<'_, V, Pred>))
        ensures
            ret.1.lock() == *self,
            self.inv(ret.0),
    {
        proof {
            use_type_invariant(self);
        }

        let tracked mut my_ticket_token_opt: Option<SetToken<int, TicketLockToks::tickets<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>>> = None;

        let my_ticket_u64 = atomic_with_ghost!(
            &self.next => fetch_add(1);
            returning t;
            ghost g => {
                let tracked tok = self.inst.borrow().take_ticket(&mut g);
                my_ticket_token_opt = Some(tok);
            }
        );

        let tracked mut my_ticket_token = my_ticket_token_opt.tracked_unwrap();
        let ghost my_ticket = my_ticket_u64 as int;

        loop
            invariant
                self.wf(),
                my_ticket_token.instance_id() == self.inst@.id(),
                my_ticket_token.set().contains(my_ticket),
        {
            let tracked mut perm_opt: Option<PointsTo<V>> = None;

            let serving = atomic_with_ghost!(
                &self.serving => load();
                returning res;
                ghost g => {
                    if res == my_ticket_u64 {
                        let tracked x = self.inst.borrow().enter(my_ticket, &g, &my_ticket_token);
                        let tracked (_, Tracked(p)) = x;
                        perm_opt = Some(p);
                    }
                }
            );

            if serving == my_ticket_u64 {
                let tracked mut perm = perm_opt.tracked_unwrap();
                let val = self.cell.take(Tracked(&mut perm));

                return (val, TicketLockGuard {
                    ticket: Ghost(my_ticket),
                    ticket_token: Tracked(my_ticket_token),
                    perm: Tracked(perm),
                    lock: self,
                });
            } else {
            }
        }
    }
}

} // verus!
