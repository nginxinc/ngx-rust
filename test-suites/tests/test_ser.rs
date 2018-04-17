#[macro_use]
extern crate ngx_module;
extern crate ngx_binding;

use ngx_binding::ngx_str_t;

#[repr(C)]
#[derive(NgxLocConf)]
pub struct test_loca_conf_t {
    pub topic: ngx_str_t,
    pub destination_service: ngx_str_t
}

#[test]
fn test1() {

    assert!(true,"test successfull");
}