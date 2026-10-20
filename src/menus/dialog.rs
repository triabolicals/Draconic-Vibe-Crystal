use engage::app::{basicdialog::*, basicdialogitem::*, yesmenuitem::*};
use super::*;

pub fn create_dialog_bind(
    proc: impl Into<ProcInst>,
    message: impl Into<Il2CppString>,
    yes_name: impl Into<Il2CppString>,
    yes_action: Action,
    no_name: impl Into<Il2CppString>,
    no_action: Action
)
{
    let yes = YesMenuItem::new(yes_action);
    yes.set_m_text(yes_name.into());
    let no = YesMenuItem::new(no_action);
    no.set_m_text(no_name.into());
    let list: List_1::<BasicMenuItem> = List_1::new_2(2);
    if let Some(i) = yes.try_cast() { list.add(i); }
    if let Some(i) = no.try_cast() { list.add(i); }
    if list.count() ==2 {
        let dialog = BasicDialog::create_basic_dialog_bind(proc, list);
        dialog.set_text(message);
    }
}
pub fn create_dialog_change_confirm(proc: impl Into<ProcInst>, title: Il2CppString, command: Il2CppString, on_yes: Action){
    let message = DVCConfigText::change_text(title.to_rust_string().as_str(), command);
    create_dialog_confirm(proc, message, on_yes);
}
pub fn create_dialog_confirm(proc: impl Into<ProcInst>, message: impl Into<Il2CppString>, yes_action: Action) {
    let yes_name = engage::app::Mess::get("MID_Decision");
    let no_name = engage::app::Mess::get("MID_MATCH_Map_UpRoad_No");
    create_dialog_bind(proc, message, yes_name, yes_action, no_name, Action::null());
}