use unity::{
    il2cpp::object::Array,
    prelude::*,
};
use skyline::patching::Patch;
use engage::{
    gameuserglobaldata::*,
    gamevariable::*, 
    gamedata::{dispos::*},
    menu::{
        BasicMenuResult,
        config::{ ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods},
        BasicMenu, BasicMenuItem,
    },
    gamedata::unit::Unit,
    mess::*,
    force::*,
    proc::ProcInst,
};
use crate::CONFIG;

#[skyline::from_offset(0x01ec5190)]
pub fn save_data_delete(path: &Il2CppString, method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x02281490)]
pub fn game_save_data_get_file_path(_type: i32, index: i32, method_info: OptionalMethod) -> &'static Il2CppString;

#[unity::from_offset("App","SaveData", "IsExist")]
pub fn save_data_is_exists(path: &Il2CppString, method_info: OptionalMethod ) -> bool;

#[skyline::from_offset(0x02285890)]
pub fn game_save_data_write(proc: u64, _type: i32, index: i32, m1: OptionalMethod, method_info: OptionalMethod);

#[skyline::hook(offset=0x0251ba60)]
pub fn set_last_save_data_info(this: &GameUserGlobalData, _type: i32, index: i32, method_info: OptionalMethod){
    call_original!(this, _type, index, method_info);
    println!("Set Last Save Data Info hook");
    // marks the file as saved so when game over happens the game delete the file
    if GameVariableManager::get_bool("G_Ironman") { 
        GameVariableManager::make_entry("G_IronmanSaved", 1);
    }
}
#[unity::hook("App", "MapSequence", "TryRestart")]
pub fn game_over_hook(this: u64, method_info: OptionalMethod) {
    // if ironman mode and save file is saved, delete the save file
    println!("Game Over Hook");
    if GameVariableManager::get_bool("G_Ironman") && CONFIG.lock().unwrap().iron_man {
        if GameVariableManager::get_bool("G_IronmanSaved") && GameUserGlobalData::get_last_save_data_type() == 6 {
            unsafe {
                let path = game_save_data_get_file_path(6, GameUserGlobalData::get_last_save_data_index(), None);
                if save_data_is_exists(path,None) { save_data_delete(path, None); }
            }
        }
    }
    else { call_original!(this, method_info); }
}

#[skyline::hook(offset=0x01fd9ca0)]
pub fn game_mode_bind(this: u64, proc: &mut ProcInst, method_info: OptionalMethod){
    call_original!(this, proc, method_info);
    if CONFIG.lock().unwrap().iron_man {
        let config_menu = proc.child.cast_mut::<BasicMenu<BasicMenuItem>>();
        config_menu.full_menu_item_list.items[1].get_class_mut().get_virtual_method_mut("GetName").map(|method| method.method_ptr = ironman_name as _);
        config_menu.full_menu_item_list.items[1].get_class_mut().get_virtual_method_mut("GetHelp").map(|method| method.method_ptr = ironman_help as _);
    }
}

pub extern "C" fn ironman_name(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString { "Ironman".into() }
pub extern "C" fn ironman_help(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString { "For the real Fire Emblem purists.\nSave file will be deleted upon a game over.".into() }

pub fn ironman_code_edits(){
    //Code Edits to disable restart/reset/time crystal and forced bookmark if on ironman mode
    if GameVariableManager::get_bool("G_Ironman") {
        println!("Iron man code patch are active");
        // Restart Build Attribute = 4
        Patch::in_text(0x01b72cb0).bytes(&[0x80, 0x00, 0x80, 0xD2]).unwrap();
        Patch::in_text(0x01b72cb4).bytes(&[0xC0, 0x03, 0x5F, 0xD6]).unwrap();

        // Reset Build Attribute = 4
        Patch::in_text(0x01b72950).bytes(&[0x80, 0x00, 0x80, 0xD2]).unwrap();
        Patch::in_text(0x01b72954).bytes(&[0xC0, 0x03, 0x5F, 0xD6]).unwrap();

        // Rewind Attribute = 4
        Patch::in_text(0x01f52230).bytes(&[0x80, 0x00, 0x80, 0xD2]).unwrap();
        Patch::in_text(0x01f52234).bytes(&[0xC0, 0x03, 0x5F, 0xD6]).unwrap();

    // bookmark instead of save for all difficulies
        Patch::in_text(0x01e4111c).bytes(&[0x2D,0x02,0x00,0x54]).unwrap();
        //SaveSuspendBefore
        Patch::in_text(0x0267730c).bytes(&[0x4D, 0xFF, 0xFF, 0x54]).unwrap();
        //SaveSuspendAfter
        Patch::in_text(0x02677448).bytes(&[0xC1, 0x07, 0x00, 0x54]).unwrap();
        // Save instead of bookmark
        Patch::in_text(0x01e40d7c).bytes(&[0x3F,0x15,0x00,0x71]).unwrap();
        Patch::in_text(0x01e40f0c).bytes(&[0x3F,0x15,0x00,0x71]).unwrap();

    }
    // if not store the original code
    else {
        // Restart Build Attribute 
        println!("Iron man code patch are inactive");
        Patch::in_text(0x01b72cb0).bytes(&[0xfd, 0x7b, 0xbc, 0xa9]).unwrap();
        Patch::in_text(0x01b72cb4).bytes(&[0xf7, 0x0b, 0x00, 0xf9]).unwrap();
    
        // Reset Build Attribute
        Patch::in_text(0x01b72950).bytes(&[0xfd, 0x7b, 0xbe, 0xa9]).unwrap();
        Patch::in_text(0x01b72954).bytes(&[0xf3, 0x0b, 0x00, 0xf9]).unwrap();
        // Rewind Attribute 
        Patch::in_text(0x01f52230).bytes(&[0xfd, 0x7b, 0xbe, 0xa9]).unwrap();
        Patch::in_text(0x01f52234).bytes(&[0xf4, 0x4f, 0x01, 0xa9]).unwrap();

    //Bookmark/Save 
        Patch::in_text(0x01e4111c).bytes(&[0x21, 0x02, 0x00, 0x54]).unwrap();
        //SaveSuspendBefore
        Patch::in_text(0x0267730c).bytes(&[0x41, 0xff, 0xff, 0x54]).unwrap();
        //SaveSuspendAfter
        Patch::in_text(0x02677448).bytes(&[0xc0, 0x07, 0x00, 0x54]).unwrap();
        // Save instead of bookmark
        Patch::in_text(0x01e40d7c).bytes(&[0x3F,0x09,0x00,0x71]).unwrap();
        Patch::in_text(0x01e40f0c).bytes(&[0x3F,0x09,0x00,0x71]).unwrap();
    }
} 
pub struct IronmanMod;
impl ConfigBasicMenuItemSwitchMethods for IronmanMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().iron_man);
        if CONFIG.lock().unwrap().iron_man != result {
            CONFIG.lock().unwrap().iron_man = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().iron_man { this.help_text = "New game saves will be marked as 'Ironman'".into();  }
        else { this.help_text = "Disables Ironman mode.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().iron_man { this.command_text = "On".into();  }
        else { this.command_text = "Off".into(); }
    }
}
#[no_mangle]
extern "C" fn ironman_create() -> &'static mut ConfigBasicMenuItem { 
    ConfigBasicMenuItem::new_switch::<IronmanMod>("Ironman Mode")
 } 
 #[unity::class("TMPro", "TextMeshProUGUI")]
 pub struct TextMeshProUGUI {
 }
 
 #[unity::class("App", "LoadingLogo")]
 pub struct LoadingLogo {
    __: [u8; 0x60],
    pub title_text: &'static mut TextMeshProUGUI,
    pub tips_text: &'static mut TextMeshProUGUI,
 }


 impl TextMeshProUGUI {
     pub fn set_text(&mut self, source_text: &Il2CppString, sync_text_input_box: bool) {
         unsafe { tmptext_settext(self, source_text, sync_text_input_box, None) };
     }
 }
 
 #[skyline::from_offset(0x2837690)]
 fn tmptext_settext(this: &mut TextMeshProUGUI, source_text: &Il2CppString, sync_text_input_box: bool, method_info: OptionalMethod);

#[unity::class("App", "UnitRecord")]
pub struct UnitRecord {
    pub values: &'static Array<i32>,
}

 #[unity::hook("App", "LoadingLogo", "SetTipsData")]
 pub fn set_tip_text(this: &mut LoadingLogo, tips: u64, method_info: OptionalMethod){
    let force = Force::get(ForceType::Dead);
    println!("set_tip_text hook");
    call_original!(this, tips, method_info);
    if force.is_none() { return; }
    let dead_force = force.unwrap();
    let count = dead_force.get_count();
    if count == 0 { return; }
    let mut string_dead = format!("{} Dead Units", count);
    this.title_text.set_text( format!("{} Dead Units", count).into(), true);
    let mut force_iter = Force::iter(dead_force);
    let mut unit_count = 0;
    while let Some(unit) = force_iter.next() {
        unsafe {
            let name = Mess::get(unit.person.get_name().unwrap()).get_string().unwrap();
            let record = unit_get_record(unit, None);
            let dead_chapter = unit_record_get_dead_chapter(record, None);
            let mut dead_chapter_name = "";
            if dead_chapter.is_some() {
                println!("Dead Chapter for {}: {}", name, dead_chapter.unwrap().name.get_string().unwrap());
                let dead_chapter_name  = chapter_get_name(dead_chapter.unwrap(), None).get_string().unwrap();
                let prefix = Mess::get(format!("{}_PREFIX", dead_chapter.unwrap().name.get_string().unwrap())).get_string().unwrap();
                if unit_count != 0 { string_dead = format!("{}\n{} in {}: {}", string_dead, name, prefix, dead_chapter_name);}
                else { string_dead = format!("{} in {}: {}", name, prefix, dead_chapter_name); }
            }
            else {
                if unit_count != 0 {
                    if unit_count % 2 == 0 { format!("{} \n {}", string_dead, name); }
                    else { string_dead = format!("{} - {}", string_dead, name); }
                }
                else {
                    string_dead = name;
                }
            }
            unit_count += 1;
        }
    }
    this.tips_text.set_text(string_dead.into(), true);
 }
 #[skyline::from_offset(0x01a57fb0)]
 fn unit_get_record(this: &Unit, method_info: OptionalMethod) -> &UnitRecord;

 #[skyline::from_offset(0x01c57f30)]
 fn unit_record_get_dead_chapter(this: &UnitRecord, method_info: OptionalMethod) -> Option<&'static ChapterData>;

 #[skyline::from_offset(0x02af9a40)]
 fn chapter_get_name(this: &ChapterData,method_info: OptionalMethod) -> &'static Il2CppString;