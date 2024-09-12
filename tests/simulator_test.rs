use hexedit::simulator::*;
use hexedit::actions::maths::*;

#[test]
fn adding() {
	let stack = StackState::new(vec![13.0.into(), 8.5.into()], None);
	let stack_holder = StackHolder::single_state(stack);
	let mut stack_manager = StackManager::new(stack_holder);

	stack_manager.apply_action(Box::new(Add));

	
}