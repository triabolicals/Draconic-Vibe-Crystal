use engage::dialog::yesno::{BasicDialogItemYes};
use engage::menu::BasicMenuResult;
use engage::menu::config::{ConfigBasicMenuItem, ConfigBasicMenuItemCommandMethods, ConfigBasicMenuItemSwitchMethods};
use unity::prelude::OptionalMethod;
use paste::paste;
use crate::config::DVCVariables;
use crate::randomizer::skill::learn::JobLearnSkillMode;

pub fn dialog_restore_text<Methods: ConfigBasicMenuItemSwitchMethods>(this: &mut BasicDialogItemYes, parent: bool) {
    let menu =  unsafe {
        if parent { std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap().parent.as_mut().unwrap()) }
        else { std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap()) }
    };

    let index = menu.select_index;
    Methods::set_help_text(menu.menu_item_list[index as usize], None);
    Methods::set_command_text(menu.menu_item_list[index as usize], None);
    menu.menu_item_list[index as usize].update_text();
}
pub fn dialog_remove_command_change(this: &mut BasicDialogItemYes) {
    let menu =  unsafe {
        std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap())
    };

    let index = menu.select_index;
    let command_str = menu.menu_item_list[index as usize].command_text.to_string();
    let help_str = menu.menu_item_list[index as usize].help_text.to_string();
    menu.menu_item_list[index as usize].command_text = command_str.replace("*", "").into();
    menu.menu_item_list[index as usize].help_text = help_str.replace("(A to Confirm)", "").into();
    menu.menu_item_list[index as usize].update_text();
}
pub fn dialog_restore_text_command<Methods: ConfigBasicMenuItemCommandMethods>(this: &mut BasicDialogItemYes) {
    let menu =  unsafe { std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap()) };
    let index = menu.select_index;
    Methods::set_help_text(menu.menu_item_list[index as usize], None);
    menu.menu_item_list[index as usize].update_text();
}

pub fn close_this_with_cancel(_this: &engage::proc::ProcInst, _optional_method: unity::prelude::OptionalMethod) -> BasicMenuResult {
    BasicMenuResult::new().with_close_this(true).with_se_cancel(true)
}


#[macro_export] macro_rules! random_confirm {
    ($dvc:ident, $name:ident) => {
        paste::paste!{
            pub struct [<$name ChangeConfirm>];
            impl engage::dialog::yesno::TwoChoiceDialogMethods for [<$name ChangeConfirm>]{
                extern "C" fn on_first_choice(this: &mut engage::dialog::yesno::BasicDialogItemYes, _method_info: unity::prelude::OptionalMethod) ->engage::menu::BasicMenuResult {
                    crate::menus::utils::dialog_remove_command_change(this);
                    crate::config::DVCVariables::update_var_from_temp(DVCVariables::$dvc);
                    engage::menu::BasicMenuResult::new().with_close_this(true).with_se_decide(true)
                }
            }
        }
    }
}
