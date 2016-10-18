pub trait Watchdog {
	fn start(&self);
	fn stop(&self);
	fn tickle(&self);
}