pub use crate::prelude::*;

use super::{ConflictActionBuilder, ConflictTarget, OnConflict};

pub trait ConflictQuery<'args>: HasArguments<'args> {
    fn set_on_conflict(&mut self, on_conflict: OnConflict) -> &mut Self;

    fn on_conflict(
        &mut self,
        target: ConflictTarget,
        action: ConflictActionBuilder<'args>,
    ) -> &mut Self {
        let on_conflict = OnConflict {
            conflict_target: target,
            action: action.process(self.holder()),
        };
        self.set_on_conflict(on_conflict)
    }

    fn on_conflict_set_excluded<C: ColumnType + 'static>(
        &mut self,
        target: ConflictTarget,
        columns: Vec<C>,
    ) -> &mut Self {
        self.on_conflict(target, ConflictActionBuilder::update_to_excluded(columns))
    }

    fn on_conflict_do_nothing(&mut self, target: ConflictTarget) -> &mut Self {
        self.on_conflict(target, ConflictActionBuilder::do_nothing())
    }
}
