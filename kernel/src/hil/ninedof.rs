use returncode::ReturnCode;

pub trait NineDof {
    fn set_client(&self, client: &'static NineDofClient);

    fn read_accelerometer(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn read_magnetometer(&self) -> ReturnCode {
        ReturnCode::FAIL
    }
}

pub trait NineDofClient {
    fn callback(&self, arg1: usize, arg2: usize, arg3: usize);
}
