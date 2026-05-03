use engage::{
    gamedata::{ChapterData, Gamedata},
    gameuserdata::GameUserData, gamevariable::GameVariableManager,
    proc::{desc::ProcDesc, Bindable, ProcInst, ProcVoidMethod},
    menu::menus::accessory::change::AccessoryShopChangeRoot,
    sequence::mainsequence::MainSequence,
};
use crate::{
    menus, message, DVCVariables, config::menu::DVCConfigText,
    procs::{call_proc_original_method, replace_desc_void_function},
};
use outfit_core::UnitAssetMenuData;
use unity::{prelude::OptionalMethod, il2cpp::object::Array};
use crate::randomizer::status::RandomizerStatus;

pub fn main_sequence_desc_edit(descs: &mut Array<&mut ProcDesc>) {
    descs[18] = ProcDesc::call(ProcVoidMethod::new(None, main_sequence_initialize));

    replace_desc_void_function(descs, "LoadResource", main_sequence_load_resource as _);
    // descs[30] = ProcDesc::call(ProcVoidMethod::new(None, main_sequence_load_resource));
    descs[43] = ProcDesc::call(ProcVoidMethod::new(None, main_sequence_game_reset));
    descs[101] = ProcDesc::call(ProcVoidMethod::new(None, main_sequence_try_jump_to_next_chapter));
    descs[116] = ProcDesc::call(ProcVoidMethod::new(None, main_sequence_jump_to_continue_map));
    /*
        115 - TryJumpKizuana
        116 - TryContinueMap
        117 - TryJumpHub
        118 - TryJumpGmap
        119 - TryJumpNextChapter
     */
    //
}

extern "C" fn main_sequence_initialize(map_sequence: &mut ProcInst, _optional_method: OptionalMethod) {
    UnitAssetMenuData::get().is_dvc = true;
    call_proc_original_method(map_sequence, "Initialize");
    crate::randomizer::initialize_game_data();
    menus::menu_calls_install();
}

extern "C" fn main_sequence_load_resource(main_sequence: &mut MainSequence, _optional_method: OptionalMethod) {
    crate::config::menu::CONFIG_TEXT.get_or_init(|| DVCConfigText::init());
    main_sequence.load_resource();
    main_sequence.pad = 1;
    message::initialize_mess_hashs();
    crate::randomizer::emblem::god_pool();
    AccessoryShopChangeRoot::load_prefab_async();
}

extern "C" fn main_sequence_game_reset(main_sequence: &mut ProcInst, _optional_method: OptionalMethod) {
    if RandomizerStatus::get().enabled { crate::randomizer::reset_gamedata(); }
    call_proc_original_method(main_sequence, "GameReset");
}
extern "C" fn main_sequence_jump_to_continue_map(main_sequence: &mut ProcInst, _optional_method: OptionalMethod) {
    let con_mode = DVCVariables::Continuous.get_value();
    if con_mode == 2 && GameUserData::get_status().value & 64 != 0{
        if let Some(chapter) = ChapterData::try_get_hash(GameVariableManager::get_number("G_DVC_Next")) {
            GameUserData::set_chapter(chapter);
            main_sequence.jump(5);
            return;
        }
    }
    call_proc_original_method(main_sequence, "TryJumpToContinueMap");
}

extern "C" fn main_sequence_try_jump_to_next_chapter(main_sequence: &mut ProcInst, _optional_method: OptionalMethod) {
    let con_mode = DVCVariables::Continuous.get_value();
    if con_mode == 2 || con_mode == 1 {
        if let Some(chapter) = ChapterData::try_get_hash(GameVariableManager::get_number("G_DVC_Next")) {
            GameUserData::set_chapter(chapter);
            main_sequence.jump(5);
            return;
        }
    }
    call_proc_original_method(main_sequence, "TryJumpToNextChapter");
}
