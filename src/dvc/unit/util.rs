use engage::app::{Unit, UnitFor, UnitPool};
use unity::Cast;

pub fn for_each_unit(force_mask: u32, function: impl Fn(Unit)){
    if let Some(unit) = crate::utils::to_opt(UnitPool::get_first(force_mask, 0)) {
        function(unit);
        let mut next = UnitFor::get_next(unit);
        while !next.is_null() {
            function(next);
            next = UnitFor::get_next_2(next, force_mask);
        }
    }
}