use unity::{prelude::*, il2cpp::object::Array};
use engage::{
    menu::{
        BasicMenuResult, 
        config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}
    },
    gamevariable::*,
    gameuserdata::*,
    hub::access::*,
    random::*,
    mess::*,
    gamedata::{*, item::*, dispos::*, unit::*},
};

use crate::emblem::*;
use crate::random::*;
use crate::deploy;
use super::CONFIG;
use crate::item;
pub static mut RAND_PERSONS: [i32; 82] = [0; 82];
pub const PIDS : &[&str] = &["PID_リュール", "PID_ヴァンドレ", "PID_クラン", "PID_フラン", "PID_アルフレッド", "PID_エーティエ", "PID_ブシュロン", "PID_セリーヌ", "PID_クロエ", "PID_ルイ", "PID_ユナカ", "PID_スタルーク", "PID_シトリニカ", "PID_ラピス", "PID_ディアマンド", "PID_アンバー", "PID_ジェーデ", "PID_アイビー", "PID_カゲツ", "PID_ゼルコバ", "PID_フォガート", "PID_パンドロ", "PID_ボネ", "PID_ミスティラ", "PID_パネトネ", "PID_メリン", "PID_オルテンシア", "PID_セアダス", "PID_ロサード", "PID_ゴルドマリー", "PID_リンデン", "PID_ザフィーア", "PID_ヴェイル", "PID_モーヴ", "PID_アンナ", "PID_ジャン", "PID_エル", "PID_ラファール", "PID_セレスティア", "PID_グレゴリー", "PID_マデリーン"];
pub const RINGS: [&str; 19] = ["Marth", "Siglud", "Celica", "Micaiah", "Roy", "Leaf", "Lucina", "Lin", "Ike", "Byleth", "Kamui", "Eirik", "Edelgard", "Tiki", "Hector", "Veronica", "Senerio", "Camilla", "Chrom" ];
pub static mut SET: i32 = 0;

pub fn randomize_person() {
    change_hub_dispos(true);
    if !GameVariableManager::get_bool("G_Random_Recruitment") {
        unsafe { 
            for i in 0..41 { 
                RAND_PERSONS[i as usize] = i; 
                RAND_PERSONS[41 + i as usize] = i; 
                let string = format!("G_R_{}",PIDS[i as usize]);
                GameVariableManager::make_entry_str(&string, PIDS[i as usize]);
            } 
        }
    }
    else {
        unsafe {
            let mut emblem_list_size: i32 = 36;
            if has_content(0, None) { emblem_list_size = 41; }
            for i in 0..41 { 
                RAND_PERSONS[i as usize] = i; 
                RAND_PERSONS[41 + i as usize] = i;
                let string = format!("G_R_{}",PIDS[i as usize]);
                GameVariableManager::make_entry_str(&string, PIDS[i as usize]);
            }
            let rng = Random::instantiate().unwrap();
            let seed = GameVariableManager::get_number("G_Random_Seed") as u32;
            rng.ctor(seed);
            let mut emblem_count: i32 = 1;
            let mut set_emblems: [bool; 41] = [false; 41];
            set_emblems[0] = true;
            while emblem_count < emblem_list_size {
                let index = rng.get_value(emblem_list_size);
                if index >= emblem_list_size { continue; }
                if !set_emblems[index as usize] {
                    let string = format!("G_R_{}",PIDS[emblem_count as usize]);
                    GameVariableManager::set_string(&string, PIDS[index as usize]);
                    RAND_PERSONS[ emblem_count as usize ] = index;
                    RAND_PERSONS[ ( index + 41 ) as usize ] = emblem_count;
                    println!("{} -> {}", PersonData::get(PIDS[ emblem_count as usize]).unwrap().get_name().unwrap().get_string().unwrap(),
                    PersonData::get(PIDS[ index as usize]).unwrap().get_name().unwrap().get_string().unwrap());
                    set_emblems[ index as usize ] = true;
                    emblem_count += 1;
                }
            }
        }
        change_hub_dispos(false);
    }
}
pub fn find_pid_replacement(pid: &String, reverse: bool) -> Option<&str>{
    let mut index = 0;
    for z in PIDS { // Vander
        if *z == pid {
            unsafe {
                if reverse { return Some(PIDS[ RAND_PERSONS [ index + 41 ] as usize ]); }
                else { return Some(PIDS[ RAND_PERSONS [ index ] as usize ]); }
            }
        }
        index += 1;
    }
    index = 0;
    for z in deploy::EMBLEM_GIDS {
        if *z == pid {
            unsafe {
                if reverse { return Some(deploy::EMBLEM_GIDS [ RANDOMIZED_INDEX [ index + 19] as usize ]); }
                else { return Some(deploy::EMBLEM_GIDS[ RANDOMIZED_INDEX [ index as usize] as usize ]); }
            }
        }
        index += 1;
    }
    return None;
}
pub fn pid_to_mpid(pid: &String) -> String {
    return PersonData::get(&pid).unwrap().get_name().unwrap().get_string().unwrap();
}
pub fn change_hub_dispos(revert: bool) {
    let t_list = HubDisposData::get_array_mut().expect("Me");
    for x in 0..t_list.len() {
        for y in 0..t_list[x].len() {
            let aid = t_list[x][y].get_aid();
            if aid.is_some() { 
                let pid = aid.unwrap().get_string().unwrap();
                let new_pid = find_pid_replacement(&pid, revert);
                if new_pid.is_some() { 
                    let n_pid = new_pid.unwrap();
                  //  println!("Hub {} {} -> {} (reverted: {}", x,  pid_to_mpid(&pid),  pid_to_mpid(&n_pid.to_string()), revert);
                    t_list[x][y].set_aid(n_pid.into());
                 }
            }
        }
    }
}
pub fn change_map_dispos() {
    let t_list = DisposData::get_array_mut().expect("Me");
    for x in 0..t_list.len() {
        for y in 0..t_list[x].len() {
            let aid = t_list[x][y].get_pid();
            if aid.is_some() && ( ( t_list[x][y].get_flag().value & 8 ) == 0 && t_list[x][y].get_force() == 0 ) {
                let pid = aid.unwrap().get_string().unwrap();
                let mut index = 0;
                //Resetting to Normal 
                for z in PIDS {
                    if *z == pid {
                        unsafe {
                            let new_pid = PIDS[ RAND_PERSONS [ 41 + index ] as usize ].into();
                            t_list[x][y].set_pid(new_pid);
                            break;
                        }
                    }
                    index += 1;
                }
                index = 0;
                // New One
                for z in PIDS {
                    if *z == pid {
                        unsafe {
                            let new_pid = PIDS[ RAND_PERSONS [ index ] as usize ].into();
                            t_list[x][y].set_pid(new_pid);
                            break;
                        }
                    }
                    index += 1;
                }
            }
        }
    }
}
pub fn get_low_class_index(this: &PersonData) -> usize {
    let apt = this.get_aptitude();
    if apt.value & 2 == 2 { return 0;  }
    else if apt.value & 4 == 4 { return 1; }
    else if apt.value & 8 == 8 { return 2; }

    let apt2 = this.get_sub_aptitude();
    if apt2.value & 2 == 2 { return 0; }
    else if apt2.value & 4 == 4 { return 1; }
    else if apt2.value & 8 == 8 { return 2; }
    return 0;
}

pub fn switch_person(person: &PersonData) -> &'static PersonData {
    if !GameVariableManager::get_bool("G_Random_Recruitment") { return PersonData::get(&pid).unwrap(); }
    let pid = person.pid.get_string().unwrap();
    let var_str = format!("G_R_{}", pid);
    let new_pid = GameVariableManager::get_string(&var_str);
    unsafe {
        if is_null_empty(new_pid, None) { return PersonData::get(&pid).unwrap(); }
    }
    let new_person = PersonData::get(&new_pid.get_string().unwrap());
    if new_person.is_some() {
        return new_person.unwrap(); 
    }
    else {
        return PersonData::get(&pid).unwrap(); 
    }
}

pub fn switch_person_reverse(person: &PersonData) -> &'static PersonData {
    let pid = person.pid.get_string().unwrap();
    unsafe {
        let mut index: usize = 0;
        for x in PIDS {
            if *x == pid {
                return PersonData::get(PIDS[ RAND_PERSONS [ 41 + index ] as usize ] ).unwrap();
            }
            index += 1;
        }
    }
    return PersonData::get(&pid).unwrap();
}

pub fn is_player_unit(person: &PersonData) -> bool {
    let pid = person.pid.get_string().unwrap();
    for x in PIDS {
        if *x == pid { return true; }
    }
    return false;
}
#[unity::from_offset("App", "Unit", "set_Person")]
pub fn unit_set_person(this: &Unit, person: &PersonData, method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "SetSelectedWeaponFromOriginalAptitude")]
fn unit_set_select_weapon_from_original_aptitude(this: &Unit, mask: &WeaponMask, method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "AddAptitudeFromWeaponMask")]
fn unit_add_apt_from_weapon_mask(this: &Unit, method_info: OptionalMethod);

// done in Unit$$CreateImpl1
pub fn fixed_unit_weapon_mask(this: &mut Unit){
    this.original_aptitude.value = this.person.get_aptitude().value;
    this.aptitude.value = this.original_aptitude.value | this.person.get_sub_aptitude().value;
    this.selected_weapon_mask.value = 0;
    this.update_weapon_mask();
    this.set_select_weapon_from_original_aptitude(this.original_aptitude);
    this.update_weapon_mask();
    this.add_aptitude_from_weapon_mask();
}
#[unity::hook("App", "Unit", "CreateImpl2")]
pub fn unit_create_impl_2_hook(this: &mut Unit, method_info: OptionalMethod){
    // if Alear or not a playable unit, behave normally
    call_original!(this, method_info);
    if this.person.pid.get_string().unwrap() == "PID_リュール" || !is_player_unit(this.person) {
        return;
    }
    if GameVariableManager::get_bool("G_Random_Recruitment"){
        let mut person = this.get_person();
        let mut new_person = switch_person(person);
      //  let mut current_rating = deploy::get_unit_rating(this);
        if GameUserData::get_sequence() == 5  ||  GameUserData::get_sequence() == 4  {  // Hub & Kizuna: person is already the correct person
            println!("Hub/Kizuna Recruitment");
            new_person = this.get_person();
            this.set_person( switch_person_reverse(person) );
            person = this.get_person();
        }
        // if randomized to the same person
        if new_person.pid.get_string().unwrap() == person.pid.get_string().unwrap() {
            if GameVariableManager::get_bool("G_Random_Job") {
                item::unit_change_to_random_class(this);
                fixed_unit_weapon_mask(this);
                adjust_unit_items(this);
            }
            return;
        }
        //let class1 = person.get_job().unwrap().name.get_string().unwrap();
        //let class2 = new_person.get_job().unwrap().name.get_string().unwrap();
        println!("{} -> {}",  person.get_name().unwrap().get_string().unwrap(), new_person.get_name().unwrap().get_string().unwrap());
        let is_low = person.get_job().unwrap().is_low();
        let is_new_low = new_person.get_job().unwrap().is_low();

        let current_level = person.get_level() as i32;
        let mut current_internal_level = person.get_internal_level() as i32;
        if current_internal_level == 0 && !is_low { current_internal_level = 20; }

        //let new_level = new_person.get_level() as i32;
        //let new_internal_level = new_person.get_internal_level() as i32;

        if is_low {
            if current_level > 20 { //Old Unit is in a special class so new unit needs to be promoted
                if is_new_low && new_person.get_job().unwrap().has_high() {    // new unpromoted unit can promoted 
                    //let target_level = current_level;
                    let level = current_level - 20;
                    let new_job = &new_person.get_job().unwrap().get_high_jobs()[0];
                    this.set_job(new_job);
                    this.auto_grow_capability( level, current_level);
                    call_original!(this, method_info);
                    this.set_level( level );
                    this.set_internal_level( 20 );
                }
                else if is_new_low && !new_person.get_job().unwrap().has_high() {   // special -> special
                    this.set_job(new_person.get_job().unwrap());
                    this.auto_grow_capability( current_level, current_level);
                    call_original!(this, method_info);
                    this.set_level( current_level );
                    this.set_internal_level( 0 );
                }
                else {  // special -> high
                    this.set_job(new_person.get_job().unwrap());
                    this.auto_grow_capability( current_level-20, current_level);
                    call_original!(this, method_info);
                    this.set_level( current_level - 20 );
                    this.set_internal_level( 20 );
                }
            }
            if is_new_low { // base or special class lvl < 20 -> base class
                this.set_job(new_person.get_job().unwrap());
                this.auto_grow_capability( current_level, current_level);
                call_original!(this, method_info);
                this.set_level( current_level );
                this.set_internal_level( 0 );
            }
            else {
                let new_job_list = new_person.get_job().unwrap().get_low_jobs();
                if new_job_list.len() == 3 {
                    let index = get_low_class_index(new_person);
                    this.set_job(&new_job_list[index as usize]);
                }
                else {  this.set_job(&new_job_list[0]); }
                this.auto_grow_capability(current_level, current_level);
                call_original!(this, method_info);
                this.set_level(current_level);
                this.set_internal_level(0);
            }
        }
        else {  // Promoted
            if is_new_low { // new unit has a base class
                if new_person.get_job().unwrap().has_high() {   // base class -> 1st promoted class
                    this.set_job(&new_person.get_job().unwrap().get_high_jobs()[0]);
                    this.auto_grow_capability(current_level, current_level + 20);
                    call_original!(this, method_info);
                    this.set_level(current_level);
                    this.set_internal_level(current_internal_level);
                }
                else { // Promoted -> Special
                    this.set_job(new_person.get_job().unwrap());
                    let total_level = current_internal_level + current_level;
                    this.auto_grow_capability(total_level, 20+current_level);
                    call_original!(this, method_info);
                    this.set_level(total_level);
                    this.set_internal_level(0);
                    this.set_level( ( person.get_level() + person.get_internal_level() as u8 ).into() );
                }
            }
            else {  // Promoted -> Promoted
                this.set_job(new_person.get_job().unwrap());
                this.auto_grow_capability(current_level, current_level + 20);
                call_original!(this, method_info);
                this.set_level(current_level);
                this.set_internal_level( current_internal_level );
            }
        }
        this.set_person(new_person);    // change person
        fixed_unit_weapon_mask(this);   // fixed weapon mask due to class changes
    }
    if GameVariableManager::get_bool("G_Random_Job") {
        item::unit_change_to_random_class(this);
        fixed_unit_weapon_mask(this);
    }
    if GameVariableManager::get_bool("G_Random_Recruitment") ||  GameVariableManager::get_bool("G_Random_Job") {  adjust_unit_items(this);  }
}
 #[unity::hook("App", "Unit", "CreateFromDispos")]
 pub fn create_from_dispos_hook(this: &mut Unit, data: &DisposData, method_info: OptionalMethod) {
    unsafe {
        if !is_null_empty(data.get_gid(), None) {
            let string = format!("G_R_{}", data.get_gid().get_string().unwrap());
            let new_gid = GameVariableManager::get_string(&string);
            if !is_null_empty(new_gid, None) { data.set_gid(new_gid); }
        }
    }
    call_original!(this, data, method_info);
    if !is_player_unit(this.person) { return; }
    if GameVariableManager::get_bool("G_Random_Recruitment") ||  GameVariableManager::get_bool("G_Random_Job") { 
        if GameVariableManager::get_bool("G_Random_Job") {
            item::unit_change_to_random_class(this);
            fixed_unit_weapon_mask(this);
        }
        adjust_unit_items(this); 
    }
 }

pub fn adjust_unit_items(unit: &Unit) {
    let job = unit.get_job();
    let mut weapon_mask = job.get_weapon_mask().value | unit.selected_weapon_mask.value;
    if weapon_mask == 0 {
        weapon_mask = unit.selected_weapon_mask.value;
    }
    let list_count = unit.item_list.get_count();
    let name =  unit.person.get_name().unwrap().get_string().unwrap();
    if list_count == 0 {
        return;
    }
    let mut slot = 0;
    let mut weapon_mask_array: [i32; 4] = [0; 4];
    let mut weapon_level: [i32; 4] = [0; 4];
    for x in 1..9 {
        if x == 7 { continue; }
        if weapon_mask & (1 << x) != 0 {
            weapon_mask_array[slot as usize] =  weapon_mask & (1 << x);
            weapon_level[slot as usize] = job.get_max_weapon_level(x);
            println!("Job has weapon type: {}, max level: {}", x, weapon_level[slot as usize]);
            slot += 1;
        }
        if slot == 4 { break; }
    }
    let n_weapons = slot;
    slot = 0;
    for x in 0..list_count+3 {
        let item = unit.item_list.get_item(x);
        if item.is_some() {
            let weapon = &item.unwrap();
            let kind = weapon.item.get_kind(); 
            if kind > 8 || kind == 0 { continue; }
            if kind == 7 { continue; }
            let rank = weapon.item.get_weapon_level();
            println!("{}: Weapon Mask {} & {} (kind = {}, rank {} ) = {} for {} ", name, weapon_mask, 1 << kind, kind, rank, weapon_mask & ( 1 <<  kind ), weapon.item.name.get_string().unwrap());
            if weapon_mask & ( 1 <<  kind ) == 0 {
                // For Veyle
                if name == "MPID_Veyre" && weapon_mask_array[slot] == 64 {
                    if slot == 0 { weapon.ctor(ItemData::get("IID_オヴスキュリテ").unwrap());  }
                    else if slot == 1 { weapon.ctor(ItemData::get("IID_ミセリコルデ").unwrap());  }
                }
                else {
                    if slot < n_weapons {
                        item::replace_weapon(weapon, weapon_mask_array[slot as usize], weapon_level[slot as usize]);
                        if n_weapons > 1 { slot += 1; }
                    }
                    else {
                        item::replace_weapon(weapon, weapon_mask_array[slot - 1 as usize], weapon_level[slot - 1 as usize]);
                    }
                }
            }
        }
    }

    item::adjust_staffs(unit);
    unsafe { unit_update_auto_equip(unit, None); }
}

#[unity::from_offset("App", "Unit", "UpdateStateWithAutoEquip")]
pub fn unit_update_auto_equip(this: &Unit, method_info: OptionalMethod);

pub struct RandomPersonMod;
impl ConfigBasicMenuItemSwitchMethods for RandomPersonMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().random_recruitment);
        if CONFIG.lock().unwrap().random_recruitment != result {
            CONFIG.lock().unwrap().random_recruitment = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_recruitment { this.help_text = "Characters will be recruited in a random order.".into(); }
        else { this.help_text = "Standard recruitment order.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_recruitment { this.command_text = "Random".into();  }
        else { this.command_text = "Standard".into(); }
    }
}

#[unity::hook("App", "Mess", "GetImpl")]
pub fn mess_get_impl_hook(label: Option<&Il2CppString>, is_replaced: bool, method_info: OptionalMethod) -> &'static Il2CppString {
    let result = call_original!(label, is_replaced, method_info);
    unsafe {
        if label.is_some() {
            let mess_label = label.unwrap().get_string().unwrap();
            if mess_label == "MID_RULE_M006_LOSE" {
                return replace_str(result, Mess::get("MPID_Yunaka"), Mess::get(PersonData::get( PIDS [ RAND_PERSONS[10] as usize ]).unwrap().get_name().unwrap()), None);
            }
            if mess_label == "MID_RULE_M015_WIN" {
                return replace_str(result, Mess::get("MPID_Seadas"), Mess::get(PersonData::get( PIDS [ RAND_PERSONS[27] as usize ]).unwrap().get_name().unwrap()), None);
            }
            if mess_label == "MID_RULE_M015_LOSE" {
                return replace_str(result, Mess::get("MPID_Seadas"), Mess::get(PersonData::get( PIDS [ RAND_PERSONS[27] as usize ]).unwrap().get_name().unwrap()), None);
            }
            if mess_label == "MID_TUT_NAVI_M015_ESCAPE" {
                return replace_str(result, Mess::get("MPID_Seadas"), Mess::get(PersonData::get( PIDS [ RAND_PERSONS[27] as usize ]).unwrap().get_name().unwrap()), None);
            }
            if string_start_with(label.unwrap(), "MTID_Ring_".into(), None) {
                for x in 1..12 {
                    let tid_label = format!("MTID_Ring_{}",  RINGS[x as usize ]);
                    if mess_label == tid_label { return Mess::get(format!("MGID_Ring_{}", RINGS[ RANDOMIZED_INDEX[ x as usize ] as usize ])); }
                }
            }
            if string_start_with(label.unwrap(), "MID_TUT_NAVI_M022_GET_".into(), None){
                if GameVariableManager::get_number("G_Emblem_Mode") != 0 {
                    let mock_text = call_original!(Some("MID_TUT_NAVI_M022_GET_Siglud".into()), is_replaced, method_info);
                    for x in RINGS {
                        if str_contains(label.unwrap(), x) {
                            let new_ring = format!("MGID_Ring_{}", x);
                            return replace_str(mock_text, Mess::get("MGID_Ring_Siglud"),  Mess::get(new_ring), None);
                        }
                    }
                }
            }
        }
    }
    return result;
}
#[skyline::hook(offset=0x01e56900)]
pub fn cmd_info_ctor_hook(this: u64, func: u64, cmd_name: &Il2CppString, args: &mut Array<&Il2CppString>, method_info: OptionalMethod) {
    for i in 0..args.len() {
        let arg = args[i].get_string().unwrap();
        let mut index = 0;
        for z in PIDS { // Vander
            if *z == arg {
                unsafe {
                    args[i] = PIDS[ RAND_PERSONS [ index ] as usize ].into();
                    break;
                }
            }
            index += 1;
        }
    }
    call_original!(this, func, cmd_name, args, method_info);
}

#[no_mangle]
extern "C" fn person_rng() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_switch::<RandomPersonMod>("Recruitment Order") } 
pub fn install_rng_person() { cobapi::install_global_game_setting(person_rng); }
