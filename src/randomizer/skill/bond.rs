use engage::dialog::yesno::{BasicDialogItemNo, BasicDialogItemYes, TwoChoiceDialogMethods, YesNoDialog};
use engage::gamedata::Gamedata;
use engage::gamedata::ring::RingData;
use engage::gamedata::skill::SkillData;
use engage::gamevariable::GameVariableManager;
use engage::menu::{BasicMenuResult, ConfigMenu};
use engage::menu::config::{ConfigBasicMenuItem, ConfigBasicMenuItemGaugeMethods, ConfigBasicMenuItemSwitchMethods};
use engage::pad::Pad;
use engage::sequence::configsequence::ConfigSequence;
use engage::titlebar::TitleBar;
use engage::util::get_instance;
use unity::prelude::OptionalMethod;
use crate::{menus, CONFIG};
use crate::config::{DVCFlags, DVCVariables};
use crate::randomizer::skill::{get_highest_priority, MADDENING_POOL};
use crate::utils::{get_random_and_remove, get_rng};

const RANKS: [&str; 4] = ["S", "A", "B", "C"];
pub struct BondRingSetting;
impl ConfigBasicMenuItemSwitchMethods for BondRingSetting {
    fn init_content(_this: &mut ConfigBasicMenuItem) {
        if !DVCVariables::is_main_menu() {
            GameVariableManager::make_entry_norewind("BRRs", GameVariableManager::get_number(DVCVariables::BOND_RING_RATE));
        }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let v = DVCVariables::get_bond_ring_skill(true);
        let result = ConfigBasicMenuItem::change_key_value_b(v);
        if v != result {
            DVCVariables::set_bond_ring_skill(result, true);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        }
        let pad = get_instance::<Pad>();
        if ((DVCVariables::is_main_menu() && pad.npad_state.buttons.a() && !pad.old_buttons.a()) || (!DVCVariables::is_main_menu() && !pad.old_buttons.plus() && pad.npad_state.buttons.plus()))
            && DVCVariables::get_bond_ring_skill(false)
        {
            ConfigMenu::create_bind(this.menu);
            this.menu.close_anime_all();
            let config_menu = this.menu.proc.child.as_mut().unwrap().cast_mut::<ConfigMenu<ConfigBasicMenuItem>>();
            config_menu.get_class_mut().get_virtual_method_mut("OnDispose")
                .map(|method| method.method_ptr = crate::menus::submenu::open_anime_all_ondispose_to_dvc_main as _).unwrap();
            config_menu.get_class_mut().get_virtual_method_mut("BCall")
                .map(|method| method.method_ptr = bond_gauge_menu_b_call as _).unwrap();


            config_menu.full_menu_item_list.clear();
            for x in 0..4{
                let switch = ConfigBasicMenuItem::new_gauge::<BondRingGauge>( format!("{}-Rank Bond Ring Skill Rate", RANKS[x]));
                config_menu.add_item(switch);
            }
            for x in 0..4 {
                let item = &mut config_menu.full_menu_item_list[x as usize];
                item.index = x;
                BondRingGauge::set_help_text(*item, None);
                item.gauge_ratio = get_bond_rates(x as usize) as f32 * 0.01;
                item.update_text();
            }
            TitleBar::open_header("Draconic Vibe Crystal", "Bond Ring Skill Rate", "");
            return BasicMenuResult::se_cursor();
        }
        if !DVCVariables::is_main_menu() && pad.npad_state.buttons.a() && !pad.old_buttons.a() {
            let s = DVCVariables::get_bond_ring_skill(false);
            let n = DVCVariables::get_bond_ring_skill(true);
            if s != n {
                let message = if n { "Enable Bond Ring Skill Randomization?" }
                else { "Disable Bond Ring Skill Randomization? Requires Save/Reload" };
                YesNoDialog::bind::<BondRingSettingConfirm>(this.menu, message, "Do it!", "Nah");
                return BasicMenuResult::se_cursor();
            }
        }
        BasicMenuResult::new()
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        this.command_text =
            if DVCVariables::is_main_menu() { if DVCVariables::get_bond_ring_skill(false) { "Enable" } else { "Disable" } }
            else {
                let s = DVCVariables::get_bond_ring_skill(false);
                let n = DVCVariables::get_bond_ring_skill(true);
                println!("Set: {} New: {}", s, n);
                    match (s, n) {
                        (true, true) => "Enable",
                        (true, false) => "Disable*",
                        (false, true) => "Enable*",
                        (false, false) => "Disable",
                    }
            }.into();
    }

    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) {
        let s = DVCVariables::get_bond_ring_skill(false);
        this.help_text =
        if DVCVariables::is_main_menu() {
            if s { "Bond ring skills will be randomized. (Press A to set rates)" }
            else { "Bond ring skills will not be randomized." }
        }
        else {
            let n = DVCVariables::get_bond_ring_skill(true);
                match (s, n) {
                    (true, true) => "Bond ring skills are randomized. + to change rates.",
                    (true, false) => "Press to A to confirm. Requires Save/Reload.",
                    (false, true) => "Press to A to confirm. + to change rates.",
                    (false, false) => "Bond ring skills are not randomized."
                }
        }.into()
    }
}

pub struct BondRingGauge;
impl ConfigBasicMenuItemGaugeMethods for BondRingGauge {
    fn init_content(this: &mut ConfigBasicMenuItem) {
        this.update_text();
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let index = this.index as usize;
        let rate = get_bond_rates(index);
        let result = ConfigBasicMenuItem::change_key_value_i(rate, 0, 100, 5);
        if rate != result {
            set_bond_rates(index, result);
            this.gauge_ratio = result as f32 * 0.01;
            Self::set_help_text(this, None);
            this.update_text();
           BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }

    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let rate = get_bond_rates(this.index as usize);
        if rate == 0 { this.help_text = format!("No skill for bond rings rank {}.", RANKS[this.index as usize]).into(); }
        else { this.help_text = format!("{}% chance bond rings rank {} skills.", rate, RANKS[this.index as usize]).into(); }
    }
}
pub struct BondRingSettingConfirm;
impl TwoChoiceDialogMethods for BondRingSettingConfirm {
    extern "C" fn on_first_choice(_this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        DVCVariables::update_flag(DVCFlags::BondRing);
        randomize_bond_ring_skills();
        BasicMenuResult::new().with_close_this(true)
    }
}
pub struct BondRingRateChange;
impl TwoChoiceDialogMethods for BondRingRateChange {
    extern "C" fn on_first_choice(_this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::set_number(DVCVariables::BOND_RING_RATE, GameVariableManager::get_number("BRRs"));
        randomize_bond_ring_skills();
        BasicMenuResult::new().with_close_this(true).with_se_cursor(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::set_number("BRRs", GameVariableManager::get_number(DVCVariables::BOND_RING_RATE));
        BasicMenuResult::new().with_close_this(true).with_se_cancel(true)
    }
    extern "C" fn bcall_first(_this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::set_number("BRRs", GameVariableManager::get_number(DVCVariables::BOND_RING_RATE));
        BasicMenuResult::new().with_close_this(true).with_se_cancel(true)
    }
    extern "C" fn bcall_second(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::set_number("BRRs", GameVariableManager::get_number(DVCVariables::BOND_RING_RATE));
        BasicMenuResult::new().with_close_this(true).with_se_cancel(true)
    }
}

fn bond_gauge_menu_b_call(this: &mut ConfigSequence, _optional_method: OptionalMethod) -> BasicMenuResult {
    if GameVariableManager::get_number(DVCVariables::BOND_RING_RATE) != GameVariableManager::get_number("BRRs") && DVCVariables::get_bond_ring_skill(false) {
        YesNoDialog::bind::<BondRingRateChange>(this, "Update Bond Ring Skills to the new Rates?", "Do it!", "Revert Changes.");
        this.get_class_mut().get_virtual_method_mut("BCall").map(|m| m.method_ptr = menus::utils::close_this_with_cancel as _);
    }
    BasicMenuResult::new().with_close_this(true).with_se_cancel(true)
}

fn get_bond_rates(index: usize) -> i32 {
    if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().get_bond_ring_rates()[index] }
    else {
        let v = GameVariableManager::get_number("BRRs");
        (v >> (8*index)) & 127
    }
}
fn set_bond_rates(index: usize, value: i32) {
    if DVCVariables::is_main_menu() { CONFIG.lock().unwrap().set_bond_ring_rate(index, value); }
    else {
        let v = GameVariableManager::get_number("BRRs");
        let v = (v & !(0xFF << (8*index))) | (value << (8*index));
        GameVariableManager::set_number("BRRs", v);
    }
}
pub fn randomize_bond_ring_skills(){
    let ring_list = RingData::get_list_mut().unwrap();
    let ranks = [3, 2, 1, 0];
    if !GameVariableManager::exist(DVCVariables::BOND_RING_RATE) {
        let ranks_rate: [i32; 4] = CONFIG.lock().unwrap().get_bond_ring_rates();
        let var = ranks_rate[0] | (ranks_rate[1] << 8) | (ranks_rate[2] << 16) | (ranks_rate[3] << 24);
        GameVariableManager::make_entry_norewind(DVCVariables::BOND_RING_RATE, var);
    }
    if !DVCVariables::get_bond_ring_skill(false) { return; }
    let var = GameVariableManager::get_number(DVCVariables::BOND_RING_RATE);
    let ranks_rate: [i32; 4] = [
        var & 100,
        (var >> 8) & 100,
        (var >> 16) & 100,
        (var >> 24) & 100,
    ];
    let rng_rings = get_rng();
    ring_list.iter_mut().for_each(|ring| { ring.get_equip_skills().clear(); } );
    for y in 0..4 {
        let current_rank = ranks[y as usize];
        let odds = ranks_rate[y as usize];
        if odds == 0 { continue; }
        let mut pool = MADDENING_POOL.lock().unwrap().clone();
        ring_list.iter_mut()
            .filter(|ring| ring.rank == current_rank && rng_rings.get_value(100) < odds )
            .for_each(|ring|{
                let equip_skills = ring.get_equip_skills();
                let mut skill_count = 0;
                let mut skill_odds = odds;
                while rng_rings.get_value(100) < skill_odds  && skill_count < 4 {
                    if let Some(skill) = get_random_and_remove(&mut pool, rng_rings).and_then(|i| SkillData::try_index_get(get_highest_priority(i))) {
                        equip_skills.add_skill(skill, 6, 0);
                        skill_count += 1;
                    }
                    else { break; }  // no more skills
                    skill_odds = 1/ ( 1 + skill_count + y)* skill_odds + (10 - y)*current_rank;
                }
            }
            );
    }
}
