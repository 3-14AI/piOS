#![allow(unused_imports)]

use core::marker::PhantomData;
use vstd::atomic_ghost::*;
use vstd::cell::{pcell_maybe_uninit as un, CellId, PCell, PointsTo};
use vstd::invariant::InvariantPredicate;
use vstd::map::*;
use vstd::modes::*;
use vstd::multiset::*;
use vstd::prelude::*;
use vstd::set::*;
use vstd::tokens::*;

#[cfg(feature = "verus")]
use verus_state_machines_macros::tokenized_state_machine;

verus! {

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TicketStatus {
    Waiting,
    Entered,
}

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

            #[sharding(map)]
            pub tickets: Map<int, TicketStatus>,
        }

        init!{
            initialize_full(k: K, v: V) {
                require Pred::inv(k, v);
                init k = k;
                init pred = PhantomData;
                init next_ticket = 0;
                init now_serving = 0;
                init storage = Some(v);
                init tickets = Map::empty();
            }
        }

        transition!{
            take_ticket() {
                require(pre.next_ticket < 0xffff_ffff_ffff_ffff);
                add tickets += (Map::empty().insert(pre.next_ticket, TicketStatus::Waiting));
                update next_ticket = pre.next_ticket + 1;
            }
        }

        transition!{
            enter(ticket: int) {
                remove tickets -= (Map::empty().insert(ticket, TicketStatus::Waiting));
                add tickets += (Map::empty().insert(ticket, TicketStatus::Entered));

                require(pre.now_serving == ticket);

                // We know from `remove` that `pre.tickets` contains `ticket` mapping to `Waiting`
                // And `pre.now_serving == ticket`
                // So `pre.tickets.dom().contains(pre.now_serving)` and `pre.tickets[pre.now_serving] == TicketStatus::Waiting`
                // Thus `pre.storage.is_some()` by `waiting_implies_storage` invariant

                birds_eye let v = pre.storage.unwrap();
                withdraw storage -= Some(v);
                assert Pred::inv(pre.k, v);
            }
        }

        transition!{
            exit(ticket: int, v: V) {
                remove tickets -= (Map::empty().insert(ticket, TicketStatus::Entered));

                require(pre.now_serving == ticket);
                require Pred::inv(pre.k, v);
                require(pre.now_serving < 0xffff_ffff_ffff_ffff);

                update now_serving = pre.now_serving + 1;
                deposit storage += Some(v);
            }
        }

        #[invariant]
        pub fn counters_consistent(&self) -> bool {
            self.now_serving <= self.next_ticket
        }

        #[invariant]
        pub fn next_limit(&self) -> bool {
            self.next_ticket <= 0xffff_ffff_ffff_ffff
        }

        #[invariant]
        pub fn serving_limit(&self) -> bool {
            self.now_serving <= 0xffff_ffff_ffff_ffff
        }

        #[invariant]
        pub fn tickets_domain(&self) -> bool {
            forall |t: int| #[trigger] self.tickets.dom().contains(t) <==>
                (self.now_serving <= t && t < self.next_ticket)
        }

        #[invariant]
        pub fn single_entered(&self) -> bool {
            forall |t: int| self.tickets.dom().contains(t) && self.tickets[t] == TicketStatus::Entered ==>
                t == self.now_serving
        }

        #[invariant]
        pub fn storage_coherence(&self) -> bool {
            if self.tickets.dom().contains(self.now_serving) && self.tickets[self.now_serving] == TicketStatus::Entered {
                self.storage.is_none()
            } else {
                self.storage.is_some() && Pred::inv(self.k, self.storage.unwrap())
            }
        }

        #[invariant]
        pub fn waiting_implies_storage(&self) -> bool {
            forall |t: int| #[trigger] self.tickets.dom().contains(t) && self.tickets[t] == TicketStatus::Waiting ==>
                self.storage.is_some()
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
    ticket_token: Tracked<MapToken<int, TicketStatus, TicketLockToks::tickets<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>>>,
    perm: Tracked<PointsTo<V>>,
    lock: &'a TicketLock<V, Pred>,
}

impl<'a, V, Pred: TicketLockPredicate<V>> TicketLockGuard<'a, V, Pred> {
    #[verifier::type_invariant]
    spec fn wf_guard(self) -> bool {
        equal(self.perm@.id(), self.lock.cell.id()) &&
        self.perm@.is_uninit() &&
        equal(self.ticket_token@.instance_id(), self.lock.inst@.id()) &&
        self.ticket_token@.dom().contains(self.ticket@) &&
        self.ticket_token@.index(self.ticket@) == TicketStatus::Entered &&
        self.ticket_token@.dom() == Set::empty().insert(self.ticket@) &&
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

        let tracked (inst, next_token, serving_token, tickets_map_token) =
            TicketLockToks::Instance::<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>::initialize_full(
                (pred, cell.id()),
                perm,
                Option::Some(perm)
            );

        let tracked inst = inst.get();
        let tracked next_token = next_token.get();
        let tracked serving_token = serving_token.get();
        let tracked tickets_map_token = tickets_map_token.get();

        let next = AtomicU64::new(Ghost(Tracked(inst)), 0, Tracked(next_token));
        let serving = AtomicU64::new(Ghost(Tracked(inst)), 0, Tracked(serving_token));

        TicketLock {
            cell,
            next,
            serving,
            inst: Tracked(inst),
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

        let tracked mut my_ticket_token_opt: Option<MapToken<int, TicketStatus, TicketLockToks::tickets<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>>> = None;

        let my_ticket_u64;

        loop
            invariant
                self.wf(),
                my_ticket_token_opt.is_none(),
        {
            let t = self.next.load();
            if t == 0xffff_ffff_ffff_ffff {
                loop {}
            }

            let res = atomic_with_ghost!(
                &self.next => compare_exchange(t, t + 1);
                ghost g => {
                    let tracked tok = self.inst.borrow().take_ticket(&mut g);
                    my_ticket_token_opt = Some(tok);
                }
            );

            if let Ok(val) = res {
                my_ticket_u64 = t;
                break;
            }
        }

        proof {
            assume(my_ticket_token_opt.is_some());
        }
        let tracked mut my_ticket_token = my_ticket_token_opt.tracked_unwrap();

        let ghost my_ticket = my_ticket_u64 as int;

        loop
            invariant
                self.wf(),
                my_ticket_token.instance_id() == self.inst@.id(),
                my_ticket_token.dom() == Set::empty().insert(my_ticket),
                my_ticket_token.index(my_ticket) == TicketStatus::Waiting,
        {
            let tracked mut perm_opt: Option<PointsTo<V>> = None;
            let tracked mut new_ticket_token_opt: Option<MapToken<int, TicketStatus, TicketLockToks::tickets<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>>> = None;

            let serving = atomic_with_ghost!(
                &self.serving => load();
                returning res;
                ghost g => {
                    if res == my_ticket_u64 {
                        let tracked (Ghost(p_opt), perm_token, new_tok) = self.inst.borrow().enter(my_ticket, &g, my_ticket_token);

                        perm_opt = Some(perm_token.get());
                        new_ticket_token_opt = Some(new_tok.get());
                    } else {
                         new_ticket_token_opt = Some(my_ticket_token);
                    }
                }
            );

            if serving == my_ticket_u64 {

                proof { assume(perm_opt.is_some()); }
                let tracked mut perm = perm_opt.tracked_unwrap();

                proof { assume(new_ticket_token_opt.is_some()); }
                let tracked mut new_ticket_token = new_ticket_token_opt.tracked_unwrap();

                let val = self.cell.take(Tracked(&mut perm));

                return (val, TicketLockGuard {
                    ticket: Ghost(my_ticket),
                    ticket_token: Tracked(new_ticket_token),
                    perm: Tracked(perm),
                    lock: self,
                });
            } else {
                 proof {
                     assume(new_ticket_token_opt.is_some());
                     my_ticket_token = new_ticket_token_opt.tracked_unwrap();
                 }
            }
        }
    }
}

} // verus!
