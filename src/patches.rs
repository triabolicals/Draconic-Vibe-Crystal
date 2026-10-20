use skyline::patching::Patch;
use crate::utils::mov_xn_xzr;
pub fn code_patches() {
    disable_support_restriction();
    // fx_patch();
    battle_save_patch();
    skill_equip_patch();
    dual_guard_code_patch();
    hero_unit_pool_search_patch();
    // Prevents Class Roll
    Patch::in_text(0x022957d4).bytes(&[0x28, 0x00, 0x80, 0x52]).unwrap();

    // Expands Save Slots to 32 for debuging
    Patch::in_text(0x0228151c).bytes(&[0x8A, 0x0C, 0x80, 0x52]).unwrap();
    Patch::in_text(0x02281fb8).bytes(&[0x88, 0x0C, 0x80, 0x52]).unwrap();

    // Call Doubles
    Patch::in_text(0x01c76558).bytes(&[0x02, 0x00, 0x00, 0x14]).unwrap();
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
    // ACall Bypass
    // mov_x0_xzr(0x209949c);
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
pub fn hero_unit_pool_search_patch() {
    mov_xn_xzr(0x2b374b0, 0);
    Patch::in_text(0x2b374b4).bytes(&[0x73, 0x73, 0xc4, 0x97]).unwrap();
    /*
    mov_x0_xzr(0x20c5d70); //  App.Talk3D.TalkSequence$$InitTalk
    mov_x0_xzr(0x2b374a8); // GmapPlayerUnit$$SetFromUnitPool
    // mov_x0_xzr(0x1bfb3a4);  // ExecuteGameStart
    mov_x0_xzr(0x1bdf790);  // LaterTalkSetter Init
    mov_x0_xzr(0x1bdd75c); // PreSetup LaterTalkSequence
    mov_x0_xzr(0x02792784);     // PushUp GetPrizeBonus
    mov_x0_xzr(0x2dd4074);  // Squat GetPrizeBonus
    mov_x0_xzr(0x2dc8ed4);
    mov_x0_xzr(0x277ecdc); // ResolveMovieFileName

     */
}
pub fn reset_patch() {
    Patch::in_text(0x01dc9f8c).bytes(&[0xb5, 0xd9, 0x15, 0x94]).unwrap();   //  Reset God Exp bypass check for Engage+ Links
    Patch::in_text(0x01a39fe4).bytes(&[0x68,0x00, 0x00, 0xb4]).unwrap();    // Reset SP = EXP
    Patch::in_text(0x01d76320).bytes(&[0xfd, 0x7b, 0xbd, 0xa9]).unwrap();   // Revert Back menu item in Sortie
    Patch::in_text(0x01d76324).bytes(&[0xf6, 0x57, 0x01, 0xa9]).unwrap();
    // Alear Randomization Revert
    Patch::in_text(0x02d524e0).bytes(&[0x1f, 0x00, 0x00, 0x72]).unwrap();   // Lueur God Face Stuff
    Patch::in_text(0x02d524e4).bytes(&[0x08, 0x11, 0x89, 0x9a]).unwrap();
    Patch::in_text(0x02d524e8).bytes(&[0x08, 0x01, 0x40, 0xb9]).unwrap();

    Patch::in_text(0x0233f104).bytes(&[0x01, 0x00, 0xb0, 0x52]).unwrap();   // Emblem Alear Stuff
    Patch::in_text(0x02d51dec).bytes(&[0xb1, 0x60, 0xc7, 0x97]).unwrap();   //FaceThumbnail removes check for hero
    Patch::in_text(0x021e12ac).bytes(&[0x81, 0x23, 0xf5, 0x97]).unwrap();   //GetBondLevelFacePath
    /*
    Patch::in_text(0x02915844).bytes(&[0x1b, 0x52, 0xd8, 0x97]).unwrap();   //InfoUtil$$SetGodName to prevent the Emblem name to disable for the Hero with Emblem Alear
    Patch::in_text(0x02915694).bytes(&[0x87, 0x52, 0xd8, 0x97]).unwrap();   //SetUnitName - prevents Emblem X on hero unit when engaged with Alear
    Patch::in_text(0x01c66588).bytes(&[0xca, 0x0e, 0x0b, 0x94]).unwrap();   // Bond Exp Gauge-Related Hero check
    Patch::in_text(0x01c666ac).bytes(&[0x81, 0x0e, 0x0b, 0x94]).unwrap();   // Bond Exp Gauge-Related Hero Check
    Patch::in_text(0x02081edc).bytes(&[0x75, 0xa0, 0xfa, 0x97]).unwrap();   // god face for hero + emblem alear

     */
    Patch::in_text(0x01c69d60).bytes(&[0xd4, 0x00, 0x0b, 0x94]).unwrap();   // hero disappear when selecting emblem alear

    Patch::in_text(0x02ae9000).bytes(&[0x60, 0xc7, 0xfd, 0x97]).unwrap(); // Gender animation for the replacement unit
    Patch::in_text(0x02ae8d28).bytes(&[0x16, 0xc8, 0xfd, 0x97]).unwrap();
    Patch::in_text(0x02a5d0f4).bytes(&[0x23, 0xf7, 0xff, 0x97]).unwrap();
    Patch::in_text(0x01cfd4c4).bytes(&[0x2f, 0x76, 0x35, 0x94]).unwrap();
    Patch::in_text(0x01d03184).bytes(&[0xff, 0x5e, 0x35, 0x94]).unwrap();
    Patch::in_text(0x01e5fe00).bytes(&[0xe0, 0xeb, 0x2f, 0x94]).unwrap();
    Patch::in_text(0x01e5ff4c).bytes(&[0x8d, 0xeb, 0x2f, 0x94]).unwrap();
    Patch::in_text(0x027049c8).bytes(&[0xee, 0x58, 0x0d, 0x94]).unwrap();
    Patch::in_text(0x01c77620).bytes(&[0xfd, 0x7b, 0xbc, 0xa9]).unwrap();   // Summon Delete Impl
    Patch::in_text(0x01dee3a8).bytes(&[0x42, 0x00, 0x80, 0x52]).unwrap();
}