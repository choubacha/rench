use stats::Fact;
use plan::Plan;

pub trait Drive: Clone {
    fn drive(self, plan: Plan, collect: impl FnMut(Fact));
}


