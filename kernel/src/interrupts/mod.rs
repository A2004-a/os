pub mod pic;
pub mod timer;

pub fn init() {
    pic::init();
    timer::init();
}
