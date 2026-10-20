use engage::{
    app::{dialogitemnextchapter::*, BasicDialog, BasicDialogContent, BasicMenuItem, BasicMenu_Result, ChapterData, ConfirmDialogItemNo, DialogItemNextChapter, HubMenuSequence, HubSequence, IBasicDialogItemMethods, IBasicDialogMethods, IBasicMenuMethods, IChapterDataMethods, IDialogItemNextChapter, IGameUserDataMethods, IHubMenuSequenceMethods, IProcInst, IStructData_1Methods, MainSequence, MainSequence_Label},
    prelude::List_1,
    system::{Action, IActionMethods},
    GameVariableManager,
    Mess,
    app::assettable::prelude::IList_1Methods
};
use engage::app::IStructBase;
use unity::{Cast, FromIlInstance, Il2CppString, OptionalMethod};
use crate::{DVCVariable, DVCVariables, dvc::DVC, utils::to_opt};
use super::*;

pub fn hub_menu_sequence_proc_edit(proc_inst: ProcInst) {
    let mode = DVCVariable::Continuous.get_value();
    if (mode == 1 || mode == 2) && DVCVariables::is_main_chapter_complete(4){
        if let Some(proc) = proc_inst.try_cast(){
            let descs = proc_inst.m_descs();
            descs.set(21, Proc::call_2(ProcVoidMethod::from_fn(proc, hub_menu_open_dialog).unwrap()));
            descs.set(24, Proc::call_2(ProcVoidMethod::from_fn(proc, hub_menu_open_dialog).unwrap()));
        }
    }
}
#[unity::inject(name="DVCNextChapter", namespace="DVC", parent=DialogItemNextChapter)]
pub struct DVCNextChapterMenuItem {
    pub chapter: ChapterData,
}
#[unity::injected_methods]
impl DVCNextChapterMenuItem {
    #[override_virtual(name="ACall")]
    pub fn a_call(self) -> BasicMenu_Result {
        DVC::get().con_chapter = Some(DVCVariables::get_current_chapter().hash());
        if let Some(chapter) = to_opt(self.chapter()){
            HubSequence::next_chapter(chapter.get_cid());
            MainSequence::set_s_jump_label(MainSequence_Label::map());
        }
        let func = self.m_func();
        if !func.is_null(){ func.invoke(); }
        else { HubSequence::exit_2(MainSequence_Label::map()); }
        BasicMenu_Result::close_all()
    }
}
impl DVCNextChapterMenuItem {
    pub fn new(chapter: ChapterData, func: Action, paralogue: bool) -> Self {
        let item = Self::instantiate().unwrap();
        item.set_chapter(chapter);
        let prefix = Mess::get(format!("{}_PREFIX", chapter.get_name()));
        let name = Mess::get(chapter.get_name());
        if paralogue { IBasicDialogItemMethods::ctor(item, format!("{}: {}", prefix, name)); }
        else {
            let count = DVCVariables::chapter_number_complete(true);
            IBasicDialogItemMethods::ctor(item, format!("{}: {} [#{}]", prefix, name, count+1));
        }
        item.set_m_func(func);
        item
    }
}

extern "C" fn hub_menu_open_dialog(hub_menu_sequence: HubMenuSequence, _: OptionalMethod) {
    let mode = DVCVariable::Continuous.get_value();
    let action = Action::new(hub_menu_sequence.into(), hub_menu_sequence_jump_to_exit_method_info().into());
    if mode == 1 {
        if let Some(chapter) = DVCVariables::get_next_main_chapter() { DVCVariables::set_next_chapter(chapter); }
        else { hub_menu_sequence.open_dialog_next(); }
        next_chapter_dialog(hub_menu_sequence, action);
        return;
    }
    else if mode == 2 {
        if DVCVariables::get_next_chapter().is_none() {
            if let Some(chapter) = dvc::continuous::set_next_chapter(DVCVariables::user_data().get_chapter()) {
                DVCVariables::set_next_chapter(chapter);
            }
            else { hub_menu_sequence.open_dialog_next(); }
        }
        next_chapter_dialog(hub_menu_sequence, action);
        return;
    }
    hub_menu_sequence.open_dialog_next();
}

pub fn next_chapter_dialog(proc: impl Into<ProcInst>, func: Action) {
    let menu_item_list = List_1::<BasicMenuItem>::new();
    if let Some(chapter) = DVCVariables::get_next_chapter(){
        menu_item_list.add(DVCNextChapterMenuItem::new(chapter, func, false).into());
        if DVCVariable::Continuous.get_value() == 1 {
            let side = ["CID_S001", "CID_S002"];
            for i in 0..2{
                let paralogue = ChapterData::get(side[i].into());
                if !GameVariableManager::get_bool(paralogue.get_cleared_flag_name()) && DVCVariables::is_main_chapter_complete(i as i32 + 5){
                    menu_item_list.add(DVCNextChapterMenuItem::new(paralogue, func, true).into());
                    break;
                }
            }
            let mut paralogues = gmap::get_available_ring_paralogue();
            paralogues.sort_by(|a, b| a.1.cmp(&b.1));
            if let Some(p) = paralogues.iter().find(|&(_, level, cleared)| !cleared && *level != 255) {
                menu_item_list.add(DVCNextChapterMenuItem::new(p.0, func, true).into());
            }
        }
        if DVCVariables::has_dlc() && DVCVariables::is_main_chapter_complete(6) {
            for i in 1..7 {
                let key = format!("CID_G00{}", i);
                if let Some(divine) = to_opt(ChapterData::get(key.into())){
                    if !GameVariableManager::get_bool(divine.get_cleared_flag_name()) {
                        menu_item_list.add(DVCNextChapterMenuItem::new(divine, func, true).into());
                        break;
                    }
                }
            }
        }

        let no = ConfirmDialogItemNo::new("MID_MENU_CANCEL_NO".into());
        menu_item_list.add(no.into());
        let dialog = BasicDialog::new(menu_item_list, BasicDialogContent::create());
        let descs = IBasicDialogMethods::create_default_desc(dialog);
        dialog.create_bind(proc.into(), descs, Il2CppString::null());
        dialog.bind_parent_menu();
        dialog.set_text(Mess::get("MID_Hub_Next_Confirm"));
    }
}
#[unity::callback]
fn hub_menu_sequence_jump_to_exit(hub_menu_sequence: HubMenuSequence, _method_info: OptionalMethod) {
    hub_menu_sequence.jump(9);
}