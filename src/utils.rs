use skyline::patching::Patch;
use unity::prelude::*;

pub fn override_vtable(namespace: &str, name: &str, method_name: &str, method: *mut u8) {
    if let Some(virtual_method) = Class::try_lookup(namespace, name).ok()
        .map(|v| v.raw_mut())
        .and_then(|v| v.get_virtual_method_mut(method_name))
    {
        virtual_method.method_ptr = method;
    }
}
pub fn override_vtable_by_slot(namespace: &str, name: &str, slot: usize, method: *mut u8) {
    if let Some(class) = Class::try_lookup(namespace, name).ok()
        .map(|v| v.raw_mut())
    {
        class.get_vtable_mut()[slot].method_ptr = method;
    }
}

pub fn clamp_value<T: PartialOrd>(value: T, min: T, max: T) -> T { if value < min { min } else if value > max { max } else { value } }
pub fn wrap_value<T: PartialOrd>(value: T, min: T, max: T) -> T { if value > max { min } else if value < min { max } else { value } }

pub fn to_opt<C: Cast>(obj: C) -> Option<C> { if !obj.is_null() { Some(obj) } else { None } }
pub fn il2str(str: Il2CppString) -> Option<String> { if !str.is_null() { Some(str.to_rust_string()) } else { None } }

/// ARM Patches
pub fn mov_1(address: usize){ let _ = Patch::in_text(address).bytes(&[0x20,0x00, 0x80, 0x52]).unwrap(); }
pub fn mov_x0_0(address: usize){ let _ = Patch::in_text(address).bytes(&[0x00,0x00, 0x80, 0x52]).unwrap(); }
pub fn mov_xn_xzr(address: usize, register: u8){
    let _ = Patch::in_text(address).bytes(&[224 + register, 0x03, 0x1f, 0xaa]).unwrap();
}
pub fn return_true(address: usize){
    let _ = Patch::in_text(address).bytes(&[0x20,0x00, 0x80, 0x52]).unwrap();
    let _ = Patch::in_text(address+0x4).bytes(&[0xC0, 0x03, 0x5F, 0xD6]).unwrap();
}