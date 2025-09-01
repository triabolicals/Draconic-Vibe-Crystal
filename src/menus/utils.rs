use engage::dialog::yesno::BasicDialogItemYes;
use engage::menu::config::{ConfigBasicMenuItem, ConfigBasicMenuItemCommandMethods, ConfigBasicMenuItemSwitchMethods};

pub fn dialog_restore_text<Methods: ConfigBasicMenuItemSwitchMethods>(this: &mut BasicDialogItemYes) {
    let menu =  unsafe {
        std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap())
    };
    let index = menu.select_index;
    Methods::set_help_text(menu.menu_item_list[index as usize], None);
    Methods::set_command_text(menu.menu_item_list[index as usize], None);
    menu.menu_item_list[index as usize].update_text();
}

pub fn dialog_restore_text_command<Methods: ConfigBasicMenuItemCommandMethods>(this: &mut BasicDialogItemYes) {
    let menu =  unsafe {
        std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap())
    };
    let index = menu.select_index;
    Methods::set_help_text(menu.menu_item_list[index as usize], None);
    menu.menu_item_list[index as usize].update_text();
}

