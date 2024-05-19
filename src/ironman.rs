use unity::prelude::*;
use skyline::patching::Patch;
use engage::{
    gamevariable::*, 
    menu::{
        BasicMenuResult,
        config::{ ConfigBasicMenuItem, ConfigBasicMenuItemSwitchMethods},
    },
};
use crate::CONFIG;

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
