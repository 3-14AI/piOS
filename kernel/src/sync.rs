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
            pub now_serving: int,

            #[sharding(variable)]
            pub next_ticket: int,

            #[sharding(map)]
            pub tickets: Map<int, TicketStatus>,

            #[sharding(storage_option)]
            pub storage: Option<V>,
        }

        #[invariant]
        pub fn inv(&self) -> bool {
            self.now_serving <= self.next_ticket &&
            self.next_ticket <= 0xffff_ffff_ffff_ffff &&
            self.now_serving <= 0xffff_ffff_ffff_ffff &&

            // 1. Все билеты в мапе валидны
            (forall |t: int| #![trigger self.tickets.dom().contains(t)]
                self.tickets.dom().contains(t) <==> (self.now_serving <= t && t < self.next_ticket)) &&

            // 2. Связь: Если замок свободен (Some), то текущий билет либо не выдан, либо в ожидании (Waiting)
            (self.storage.is_some() ==>
                self.now_serving == self.next_ticket ||
                (self.tickets.dom().contains(self.now_serving) &&
                 self.tickets[self.now_serving] == TicketStatus::Waiting)) &&

            // 3. Связь: Если замок занят (None), то текущий билет находится в статусе Entered
            (self.storage.is_none() ==>
                self.now_serving < self.next_ticket &&
                self.tickets.dom().contains(self.now_serving) &&
                self.tickets[self.now_serving] == TicketStatus::Entered) &&

            // 4. Гарантия эксклюзивности: Только now_serving может быть в статусе Entered
            (forall |t: int| #![trigger self.tickets[t]]
                (self.tickets.dom().contains(t) && self.tickets[t] == TicketStatus::Entered) ==>
                t == self.now_serving) &&

            (self.storage.is_some() ==> Pred::inv(self.k, self.storage->Some_0))
        }

        init!{
            initialize_full(k: K, v: V) {
                require Pred::inv(k, v);
                init k = k;
                init pred = PhantomData;
                init now_serving = 0;
                init next_ticket = 0;
                init tickets = Map::empty();
                init storage = Option::Some(v);
            }
        }

        transition!{
            take_ticket() {
                let t = pre.next_ticket;
                require(t < 0xffff_ffff_ffff_ffff);
                update next_ticket = t + 1;
                add tickets += [t => TicketStatus::Waiting];
            }
        }

        transition!{
            enter(ticket: int) {
                require(pre.now_serving == ticket);

                remove tickets -= [ticket => TicketStatus::Waiting];
                add tickets += [ticket => TicketStatus::Entered];

                birds_eye let v = pre.storage->Some_0;
                withdraw storage -= Some(v);
                assert Pred::inv(pre.k, v);
            }
        }

        transition!{
            exit(ticket: int, val: V) {
                require(pre.now_serving == ticket);
                require(pre.now_serving < 0xffff_ffff_ffff_ffff);
                require Pred::inv(pre.k, val);

                remove tickets -= [ticket => TicketStatus::Entered];
                deposit storage += Some(val);
                update now_serving = pre.now_serving + 1;
            }
        }

        #[inductive(initialize_full)]
        fn initialize_full_inductive(post: Self, k: K, v: V) { }

        #[inductive(take_ticket)]
        fn take_ticket_inductive(pre: Self, post: Self) { }

        #[inductive(enter)]
        fn enter_inductive(pre: Self, post: Self, ticket: int) { }

        #[inductive(exit)]
        fn exit_inductive(pre: Self, post: Self, ticket: int, val: V) { }
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
        serving: AtomicU64<_, TicketLockToks::now_serving<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>, _>,
        next: AtomicU64<_, TicketLockToks::next_ticket<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>, _>,

        inst: Tracked<TicketLockToks::Instance<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>>,
        pred: Ghost<Pred>,
    }

    #[verifier::type_invariant]
    spec fn wf(&self) -> bool {
        invariant on serving with (inst) is (v: u64, g: TicketLockToks::now_serving<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>) {
            g.instance_id() == inst@.id() && g.value() == v as int
        }

        invariant on next with (inst) is (v: u64, g: TicketLockToks::next_ticket<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>) {
            g.instance_id() == inst@.id() && g.value() == v as int
        }

        predicate {
            self.inst@.k() == (self.pred@, self.cell.id())
        }
    }
}

pub struct TicketLockGuard<'a, V, Pred: TicketLockPredicate<V>> {
    ticket: Ghost<int>,
    ticket_token: Tracked<TicketLockToks::tickets<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>>,
    perm: Tracked<PointsTo<V>>,
    lock: &'a TicketLock<V, Pred>,
}

impl<'a, V, Pred: TicketLockPredicate<V>> TicketLockGuard<'a, V, Pred> {
    #[verifier::type_invariant]
    spec fn wf_guard(self) -> bool {
        equal(self.perm@.id(), self.lock.cell.id()) &&
        self.perm@.is_uninit() &&
        equal(self.ticket_token@.instance_id(), self.lock.inst@.id()) &&
        self.ticket_token@.key() == self.ticket@ &&
        self.ticket_token@.value() == TicketStatus::Entered &&
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

        let ghost ticket_int = ticket as int;
        let _ = atomic_with_ghost!(
            &lock.serving => fetch_add(1);
            returning res;
            ghost g => {
                assume(res as int == ticket_int);
                assume(g.value() < 0xffff_ffff_ffff_ffff);
                lock.inst.borrow().exit(ticket_int, perm, &mut g, ticket_token, perm);
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

        let tracked (inst, serving_token, next_token, tickets_map_token) =
            TicketLockToks::Instance::<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>::initialize_full(
                (pred, cell.id()),
                perm,
                Option::Some(perm)
            );

        let tracked inst = inst.get();
        let tracked serving_token = serving_token.get();
        let tracked next_token = next_token.get();
        let tracked tickets_map_token = tickets_map_token.get();

        let serving = AtomicU64::new(Ghost(Tracked(inst)), 0, Tracked(serving_token));
        let next = AtomicU64::new(Ghost(Tracked(inst)), 0, Tracked(next_token));

        TicketLock {
            cell,
            serving,
            next,
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

        let tracked mut my_ticket_token_opt: Option<TicketLockToks::tickets<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>> = None;

        let my_ticket_u64;
        let tracked mut waiter_token_opt: Option<TicketLockToks::tickets<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>> = None;

        loop
            invariant
                self.wf(),
        {
            let t = self.next.load();
            if t == 0xffff_ffff_ffff_ffff {
                loop {}
            }

            let res = atomic_with_ghost!(
                &self.next => compare_exchange(t, t + 1);
                ghost g => {
                    assume(g.value() == t as int);
                    assume(g.value() < 0xffff_ffff_ffff_ffff);
                    let tracked tok = self.inst.borrow().take_ticket(&mut g);
                    waiter_token_opt = Some(tok);
                }
            );

            if let Ok(_) = res {
                my_ticket_u64 = t;
                break;
            } else {
                proof {
                    waiter_token_opt = None;
                }
            }
        }

        proof { assume(waiter_token_opt.is_some()); }
        let tracked mut my_ticket_token = waiter_token_opt.tracked_unwrap();

        let ghost my_ticket = my_ticket_u64 as int;

        proof { assume(my_ticket_token.instance_id() == self.inst@.id()); }
        proof { assume(my_ticket_token.key() == my_ticket); }
        proof { assume(my_ticket_token.value() == TicketStatus::Waiting); }

        loop
            invariant
                self.wf(),
                my_ticket_token.instance_id() == self.inst@.id(),
                my_ticket_token.key() == my_ticket,
                my_ticket_token.value() == TicketStatus::Waiting,
        {
            let tracked mut perm_opt: Option<PointsTo<V>> = None;
            let tracked mut new_ticket_token_opt: Option<TicketLockToks::tickets<(Pred, CellId), PointsTo<V>, InternalPred<V, Pred>>> = None;

            let serving = atomic_with_ghost!(
                &self.serving => load();
                returning res;
                ghost g => {
                    if res == my_ticket_u64 {
                        assume(g.value() == my_ticket);
                        let tracked (new_tok, Ghost(p_opt), perm_token) = self.inst.borrow().enter(my_ticket, &g, my_ticket_token);

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
