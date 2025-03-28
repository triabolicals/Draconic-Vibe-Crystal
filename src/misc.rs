use skyline::patching::Patch;
use engage::gamedata::{Gamedata, JobData, PersonData};
use crate::CONFIG;

pub fn code_patches() {
    disable_support_restriction();
    fx_patch();
    battle_save_patch();
    skill_equip_patch();
    dual_guard_code_patch();
    // Prevents Class Roll
    Patch::in_text(0x022957d4).bytes(&[0x28, 0x00, 0x80, 0x52]).unwrap();

    // Expands Save Slots to 32 for debuging
    if CONFIG.lock().unwrap().debug { 
        Patch::in_text(0x0228151c).bytes(&[0x0A, 0x04, 0x80, 0x52]).unwrap();
        Patch::in_text(0x02281fb8).bytes(&[0x08, 0x04, 0x80, 0x52]).unwrap();
    }
}

pub fn dual_guard_code_patch() {
    Patch::in_text(0x01e4ee00).nop().unwrap();
    Patch::in_text(0x026800a8).bytes(&[0x05, 0x00, 0x00, 0x14]).unwrap();
    Patch::in_text(0x01a34db8).bytes(&[0x5e, 0x00, 0x00, 0x14]).unwrap(); // 5e 00 00 14     b          LAB_7101a34f30
    Patch::in_text(0x01a34fe0).bytes(&[0x06, 0x00, 0x00, 0x14]).unwrap(); //     b          LAB_7101a34ff8
    Patch::in_text(0x01e7afb4).bytes(&[0x5e, 0x00, 0x00, 0x14]).unwrap(); //     b          LAB_7101e7b12c
    Patch::in_text(0x0194d1c8).bytes(&[0x60, 0x00, 0x00, 0x14]).unwrap(); //    b          LAB_710194d348

    Patch::in_text(0x02713da0).nop().unwrap();

    Patch::in_text(0x02680160).nop().unwrap();
    Patch::in_text(0x01c77dfc).bytes(&[0xdc, 0xfe, 0xff, 0x17]).unwrap(); // dc fe ff 17     b          LAB_7101c7796c

}
pub fn disable_support_restriction() {
    let replace = &[0x1f, 0x25, 0x00, 0x71];
    Patch::in_text(0x0209950C).bytes(replace).unwrap();
    Patch::in_text(0x020994E0).bytes(replace).unwrap();
    Patch::in_text(0x02099538).bytes(replace).unwrap();
    Patch::in_text(0x01a2a7c0).bytes(&[0xe1,0x0e,0x80,0x12]).unwrap();
    Patch::in_text(0x01a2a7c4).bytes(&[0x02,0x0f,0x80,0x52]).unwrap();
    Patch::in_text(0x01fdea34).bytes(&[0x01,0x04,0x80, 0x52]).unwrap();
}

pub fn fx_patch() {
    Patch::in_text(0x01c79694).nop().unwrap();
    Patch::in_text(0x01c79714).nop().unwrap();
    Patch::in_text(0x01c79790).nop().unwrap();
    Patch::in_text(0x01c79738).bytes(&[0x62, 0x0e, 0x00, 0x11]).unwrap();
    Patch::in_text(0x01c79708).nop().unwrap(); 
}

pub fn battle_save_patch() {
    Patch::in_text(0x01e41118).bytes(&[0x3f,0x0d, 0x00,0x71]).unwrap();
    Patch::in_text(0x02677308).bytes(&[0x1f,0x15, 0x00,0x71]).unwrap();
    Patch::in_text(0x01e40d7c).bytes(&[0x3F,0x0d,0x00,0x71]).unwrap();
    Patch::in_text(0x01e40f0c).bytes(&[0x3F,0x0d,0x00,0x71]).unwrap();
}

pub fn skill_equip_patch() {
    Patch::in_text(0x01a379b4).bytes(&[0x09, 0x00, 0x00, 0x14]).unwrap();
    crate::utils::return_true(0x02490780);
    let offsets = [0x01a36588, 0x01a38b68, 0x01a38144, 0x01a35fa4, 0x01a391e8, 0x024a63fc, 0x01a36f34, 0x01a35ec8];
    for x in offsets { Patch::in_text(x).nop().unwrap(); }
}

pub fn set_personal_caps(){
    if crate::randomizer::RANDOMIZER_STATUS.read().unwrap().stat_caps { return; }
    if let Ok(lock) = CONFIG.try_lock() {
        if lock.max_stat_caps {
            JobData::get_list_mut().unwrap().iter().for_each(|job|{
                let base = job.get_base();
                let cap = job.get_limit();
                for x in 0..10 { cap[x] = base[x] + 125; }
                cap[10] = 99;
            });
            PersonData::get_list_mut().unwrap().iter_mut().for_each(|person|{ let limits = person.get_limit(); for y in 0..11 { limits[y] = 0; }});

            println!("Stat Caps Maxed out");
        }
        let _ = crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock|lock.stat_caps = true);
    }
}

pub fn full_bullet_patch() {

}