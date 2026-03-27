#![no_std]
#[cfg(feature = "std")]
extern crate std;
extern crate alloc;

#[cfg(feature = "std")]
pub mod verifier {
    use z3::ast::{Int};
    use z3::{Config, Context, Solver};

    /// Verifies simple bounds arithmetic
    pub fn verify_simple_bounds() -> bool {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let x = Int::new_const(&ctx, "x");
        let zero = Int::from_i64(&ctx, 0);
        let one = Int::from_i64(&ctx, 1);

        let premise = x.gt(&zero);
        let conclusion = Int::add(&ctx, &[&x, &one]).gt(&zero);

        let to_prove = z3::ast::Bool::and(&ctx, &[&premise, &conclusion.not()]);

        solver.assert(&to_prove);

        match solver.check() {
            z3::SatResult::Unsat => true,
            _ => false,
        }
    }
}

#[cfg(not(feature = "std"))]
pub mod verifier {
    pub fn verify_simple_bounds() -> bool {
        false // unsupported without std
    }
}

pub use verifier::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "std")]
    fn test_verify_simple_bounds() {
        assert!(verify_simple_bounds());
    }
}
