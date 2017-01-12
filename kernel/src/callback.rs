use core::nonzero::NonZero;
use process;
use core::mem;

#[derive(Clone,Copy)]
pub struct AppId {
    idx: usize,
}

impl AppId {
    pub unsafe fn new(idx: usize) -> AppId {
        AppId { idx: idx }
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}


pub trait FakeCallback {
    fn callback(&mut self, r0: usize, r1: usize, r2: usize);
}

#[derive(Clone, Copy)]
pub struct Callback {
    app_id: AppId,
    appdata: usize,
    fn_ptr: NonZero<*mut ()>,
    fake_callback: Option<*mut FakeCallback>,
}

impl Callback {
    pub unsafe fn new(appid: AppId, appdata: usize, fn_ptr: *mut (), fake_callback: Option<*mut FakeCallback>) -> Callback {
        Callback {
            app_id: appid,
            appdata: appdata,
            fn_ptr: NonZero::new(fn_ptr),
            fake_callback: fake_callback,
        }
    }

    pub fn schedule(&mut self, r0: usize, r1: usize, r2: usize) -> bool {
        match self.fake_callback {
            Some(ref mut fcb) => {
                unsafe {
                    let c: &mut FakeCallback = mem::transmute(*fcb);
                    c.callback(r0, r1, r2);
                }
                true
            }
            None => {
                process::schedule(process::FunctionCall {
                                      r0: r0,
                                      r1: r1,
                                      r2: r2,
                                      r3: self.appdata,
                                      pc: *self.fn_ptr as usize,
                                  },
                                  self.app_id)
            }
        }
    }

    pub fn app_id(&self) -> AppId {
        self.app_id
    }
}

