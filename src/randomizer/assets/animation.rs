use crate::utils::str_contains;
use crate::enums;
use super::*;
static mut NAME_SET: bool  = false;
pub fn add_animation_by_name_jid(mpid: String, jid: String, gender: &str) {
    let list = AssetTable::get_list_mut().unwrap();
    let mut x = 1000;
    while x < list.len() {
        let asset_entry = &mut list[x];
        if asset_entry.conditions.is_none() { x += 1; continue; }
        let mut job_conditions: [i32;2] = [-1; 2];
        let conditions = asset_entry.conditions.as_ref().unwrap(); 
        for y in 0..conditions.len() {
            let con = conditions[y].get_string();
            if con.is_err() { continue; }
            let con_str = con.unwrap();
            if con_str == mpid { job_conditions[0] = y as i32;  }
            if con_str == jid {  job_conditions[1] = y as i32; }
        }
        if job_conditions[0] == -1 || job_conditions[1] == -1 { x += 1; continue;  }
        unsafe {
            let new_asset_table_entry = AssetTable::instantiate().unwrap();
            asset_table_ctor(new_asset_table_entry, None); 
            asset_table_on_build(new_asset_table_entry, None);
            let new_conditions = Array::<&Il2CppString>::new_specific( conditions.get_class(), conditions.len()+1).unwrap();
            for y in 0..conditions.len() {
                let con = conditions[y].get_string();
                if con.is_err() { 
                    new_conditions[y] = conditions[y];
                    continue; 
                }
                else if con.unwrap() == mpid {
                    if mpid == "MPID_Lueur" {  new_conditions[y] = "!MPID_Lueur".into(); }
                    else {  new_conditions[y] = gender.into();  }
                }
                else { new_conditions[y] = conditions[y]; }
            }
            new_conditions[ conditions.len() ] = format!("!{}", mpid).into(); 
            //new_asset_table_entry.conditions = Some(new_conditions);
            asset_table_set_conditions(new_asset_table_entry, new_conditions, None);
            new_asset_table_entry.left_hand = asset_entry.left_hand;
            new_asset_table_entry.right_hand = asset_entry.right_hand;
            new_asset_table_entry.magic = asset_entry.magic;
            new_asset_table_entry.ride_model = asset_entry.ride_model;
            new_asset_table_entry.ride_dress_model = asset_entry.ride_dress_model;
            new_asset_table_entry.ride_anim = asset_entry.ride_anim;
            new_asset_table_entry.body_model = asset_entry.body_model;
            new_asset_table_entry.body_anim = asset_entry.body_anim;
            new_asset_table_entry.dress_model = asset_entry.dress_model;
            new_asset_table_entry.hair_model = None;
            let mode = asset_entry.mode;
            new_asset_table_entry.preset_name = None;
            new_asset_table_entry.mode = mode;
            for x2 in 0..16 {  new_asset_table_entry.scale_stuff[x2] = 0.0; }
            for x2 in 16..19 {  new_asset_table_entry.scale_stuff[x2] = asset_entry.scale_stuff[x2]; }
            list.insert( x as i32, new_asset_table_entry);
            x += 2;
        }
    }
}
pub fn add_body_animation_for_classes(jid: &String, weapon: &str, is_female: bool, body_anim: &str) {
    let list = AssetTable::get_list_mut().unwrap();
    let mut x = 750;
    while x < list.len() {
        let asset_entry = &mut list[x];
        if asset_entry.conditions.is_none() { x += 1; continue; }
        if asset_entry.body_anim.is_none() { x+= 1; continue; }
        if asset_entry.body_anim.unwrap().get_string().unwrap() != body_anim { x += 1; continue; }
        let new_conditions = Array::<&Il2CppString>::new_specific( asset_entry.conditions.as_ref().unwrap().get_class(), 3).unwrap();
        new_conditions[0] = jid.into();
        new_conditions[1] = weapon.into();
        new_conditions[2] = if is_female { "女装" } else { "男装" }.into();
        unsafe {
            let new_asset_table_entry = AssetTable::instantiate().unwrap();
            asset_table_ctor(new_asset_table_entry, None); 
            asset_table_on_build(new_asset_table_entry, None);
            for x2 in 0..19 {  new_asset_table_entry.scale_stuff[x2] = 0.0; }
            asset_table_set_conditions(new_asset_table_entry, new_conditions, None);
            new_asset_table_entry.body_anim = Some(body_anim.into());
            let mode = asset_entry.mode;
            new_asset_table_entry.preset_name = None;
            new_asset_table_entry.mode = mode;
            x += 1;
            list.insert( x as i32, new_asset_table_entry);
            println!("Added {} for mode {} at position {}",  body_anim, mode, x);
            return;
        }
    }
}

pub fn add_animation_unique_classes() {
    unsafe {
        let current_count = AssetTable::get_count() as usize;
        if ASSET_SIZE == 0 { 
            ASSET_SIZE = current_count;
            println!("Un-added AssetTable has {} entries", current_count);
         }
        else if current_count > ASSET_SIZE  {
             return; 
        }
    }
    println!("Attempting to add animations to unique classes");
    let upper_limit = if crate::utils::dlc_check() { 41 } else { 36 };
    for x in 0..upper_limit {
        let person = PersonData::get(PIDS[x as usize]).unwrap();
        let job = person.get_job().unwrap();
        let flag = job.get_flag();
        if !(flag.value & 1 == 1 && flag.value & 2 == 0) { continue; }
        let mpid = person.get_name().unwrap().get_string().unwrap();
        let jid = job.jid.get_string().unwrap();
        let gender = if person.get_gender() == 1 { "男装" } else { "女装" };
        println!("Finding {} for gender {}", Mess::get(job.name).get_string().unwrap(), person.get_gender() );
        add_animation_by_name_jid(mpid.clone(), jid, gender);
        if job.get_max_level() > 20 || job.is_high() { continue; }
        let high_job = job.get_high_jobs();
        if high_job.len() == 0 { continue;}
        add_animation_by_name_jid(mpid.clone(), high_job[0].jid.get_string().unwrap(), gender);
    }
    let mf = "JID_裏邪竜ノ子 | JID_蛮族 | JID_ティラユール下級 | JID_ティラユール | JID_アクスファイター| JID_ベルセルク| JID_ウォーリアー| JID_アーチャー| JID_スナイパー| JID_マージ| JID_セイジ | JID_モンク | JID_シーフ | JID_ダンサー".to_string();
    let cav_jids = "JID_グレートナイト | JID_ロイヤルナイト | JID_ソードナイト | JID_ランスナイト | JID_アクスナイト | JID_パラディン | JID_クピードー下級 | JID_クピードー | JID_アヴニール下級 | JID_アヴニール | JID_アヴニール_E | JID_クピードー_E".to_string();
    let armor_jids = "JID_ジェネラル | JID_ソードアーマー | JID_ランスアーマー | JID_アクスアーマー".to_string();
    add_body_animation_for_classes(&"JID_ダンサー".to_string(), "踊り", false, "Dnc0AM-No1_c000_N");

    add_body_animation_for_classes(&cav_jids, "魔道書", true, "Mag2BF-#_c000_M");
    add_body_animation_for_classes(&cav_jids, "魔道書", true, "UAS_Mag2BF");

    add_body_animation_for_classes(&cav_jids, "魔道書", false, "Mag2BM-#_c000_M");
    add_body_animation_for_classes(&cav_jids, "魔道書", false, "UAS_Mag2BM");

    add_body_animation_for_classes(&cav_jids, "弓", false, "Bow2BM-#_c000_L");  // Cav bow male
    add_body_animation_for_classes(&cav_jids, "弓", false, "UAS_Bow2BM");
    
    add_body_animation_for_classes(&cav_jids, "弓", true, "Bow2BF-#_c000_L");   // Cav bow female
    add_body_animation_for_classes(&cav_jids, "弓", true, "UAS_Bow2BF");

    // Armor
    add_body_animation_for_classes(&armor_jids, "魔道書", true, "Mag0AF-#_c000_M");
    add_body_animation_for_classes(&armor_jids, "魔道書", false, "Mag0AM-#_c000_M");
    add_body_animation_for_classes(&armor_jids, "魔道書", true, "UAS_Mag0AF");  
    add_body_animation_for_classes(&armor_jids, "魔道書", false, "UAS_Mag0AM");

    // Flier magic
    let fliers1 = "JID_ソードペガサス | JID_ランスペガサス | JID_アクスペガサス".to_string();
    add_body_animation_for_classes(&fliers1, "魔道書 | 弓", true, "Slp0EF-Mg1_c351_M");  
    add_body_animation_for_classes(&fliers1, "魔道書 | 弓", true, "UAS_Slp1EF");
    // Wyvern Knight Magic for Female
    add_body_animation_for_classes(&"JID_ドラゴンナイト".to_string(), "魔道書 | 弓", true, "Lnd1DF-Mg1_c350_M");
    add_body_animation_for_classes(&"JID_ドラゴンナイト".to_string(), "魔道書 | 弓", true, "UAS_Lnd1DF");
    add_body_animation_for_classes(&"JID_ドラゴンナイト".to_string(), "魔道書 | 弓", false, "Wng2DM-Lc1_c000_L");
    // Griffin

    add_body_animation_for_classes(&"JID_グリフォンナイト".to_string(), "魔道書 | 弓", true, "Wng1FF-#_c000_N");
    add_body_animation_for_classes(&"JID_グリフォンナイト".to_string(), "魔道書 | 弓", false, "Wng1FM-#_c000_N");

    add_body_animation_for_classes(&mf, "剣", false, "Swd1AM-#_c000_N");
    add_body_animation_for_classes(&mf, "剣", false, "UAS_Swd1AM");
    add_body_animation_for_classes(&mf, "剣", true, "Swd1AF-#_c000_N");
    add_body_animation_for_classes(&mf, "剣", true, "UAS_Swd1AF");


    let list = AssetTable::get_list_mut().unwrap();
    unsafe { asset_table_on_completed_end(list[0], None); }
    println!("AssetTable has {} entries after asset additions", AssetTable::get_count());
}

#[skyline::hook(offset=0x01bb2430)]
pub fn asset_table_result_setup_hook(this: &mut AssetTableResult, mode: i32, unit: &Unit, equipped: &ItemData, conditions: &Array<&'static Il2CppString>, method_info: OptionalMethod) -> &'static mut AssetTableResult {
    let result = call_original!(this, mode, unit, equipped, conditions, method_info);

    if GameVariableManager::get_number("G_Random_God_Mode") & 2 == 0 || !crate::utils::can_rand() { return result; }
    let state = unsafe { unit_god_get_state(unit, None) };
    if state == 2  { // && unit.force.unwrap().force_type == 0
        let engage_attack = unsafe { get_engage_attack(unit, None) };
        if engage_attack.is_none() { return result; } 
        let list = unsafe { get_body_anims(result, None) };
        let sid = engage_attack.unwrap().sid;
        let mut emblem_index = 50;
        let mut animation_index = 50;
        let mut old_engage = 50;
        // Finding the Engage Animation Index in Body Animations
        for x in 0..list.len() {
            let animation = ENGAGE_PREFIX.iter().position(|&y| str_contains(list[x], y));
            if animation.is_some() {
                animation_index = x;
                old_engage = animation.unwrap();
                break;
            }
        }
        // Get New Engage Attack
        for x in 0..20 {
            let e_sid = format!("SID_{}エンゲージ技", EMBLEM_ASSET[x as usize]);
            if crate::utils::str_contains(sid, &e_sid) {
                emblem_index = x; 
                break;
            }
            if x == 12 && crate::utils::str_contains(sid, "SID_三級長エンゲージ") {
                emblem_index = x;   // Three Houses Engage Attack
                break;
            }
            if crate::utils::str_contains(sid, "SID_三級長エンゲージ技＋") {
                emblem_index = 20;  // Houses Unite+
                break;
            }
        }
        println!("Found Engage Attack {} and Old Emblem: {}", emblem_index, old_engage);
        if animation_index == 50 && list.len() > 0 { animation_index = list.len() -1; }
        if emblem_index == 50 { return result; }
        if emblem_index == old_engage { return result; }
        let gender = 
            if unit.edit.is_enabled() {  unit.edit.gender }  // Alear
            else { unit.person.get_gender() }; // Everyone Else 
        let gender_str = 
            if gender == 2 || unit.person.get_flag().value & 32 != 0 && gender == 1 { "F" }
            else if unit.person.get_flag().value & 32 != 0 && gender == 2 || gender == 1 { "M"}
            else { "M" };
        let mpid = unit.person.get_name().unwrap().get_string().unwrap();
        result.ride_model = "".into();
        result.ride_dress_model = "".into();

        let new_engage_animation: &Il2CppString;
        let mut accessory_list = unsafe { asset_table_result_accessory_list(result, None) };

        // Remove Accessories
        match old_engage {
            5 => {  //Leif  Remove Weapons
                result.left_hand = "".into();
                result.right_hand = "".into();
            }
            7|10 => {  // Lyn Remove Bow, Corrin remove Dragon Fang
                result.right_hand = "".into();
            },
            12 => { // Edelgard Remove Weapons
                result.left_hand = "".into();
                result.right_hand = "".into();
                clear_accessory_from_list(&mut accessory_list, "uWep_Ax20");
                clear_accessory_from_list(&mut accessory_list, "uWep_Lc21");
                clear_accessory_from_list(&mut accessory_list, "uWep_Bw16s-Bw");
            }
            15 => {
                clear_accessory_from_list(&mut accessory_list, "uAcc_Event_SummonStone");
                clear_accessory_from_list(&mut accessory_list, "uAcc_Event_SummonStoneB");
            }
            _ => {},
        }

        match emblem_index {    //Marth, Roy, Leif, Lucina, Ike, Byleth, Dragon Blast
            0|4|5|6|8|9|19 => { 
                new_engage_animation = format!("{}1A{}-Sw1_c000_N", ENGAGE_PREFIX[ emblem_index as usize], gender_str).into(); 
                if emblem_index == 5 {
                    result.right_hand = "uWep_Sw42R".into();
                    result.left_hand = "uWep_Sw42L".into();
                    result.trail = "cEff_EngageA_Swd_00".into();
                }
            }
            2|15|16 => { 
                new_engage_animation = format!("{}1A{}-Mg1_c000_N", ENGAGE_PREFIX[ emblem_index as usize], gender_str).into();  //Celica / Veronica / Soren 
                if emblem_index == 2 { result.right_hand = "uWep_Mg00".into(); }  
                if emblem_index == 15 {  add_accessory_to_list(accessory_list, "uAcc_Event_SummonStone", "uAcc_Event_SummonStoneB");  }
            }
            1 => {  //Sigurd
                result.ride_model = "uRig_HorsR".into();
                result.ride_dress_model = "uBody_Sig0BR_c531".into();
                new_engage_animation = format!("Sig1B{}-Sw1_c000_N", gender_str).into();
            }
            3|13 => {  //Micaiah and Tiki
                if ( mpid == "MPID_Lueur" && gender == 1 ) || mpid == "MPID_Boucheron" || mpid == "MPID_Bonet" || mpid == "MPID_Vandre" || mpid == "MPID_Mauve" {
                    new_engage_animation = "Mic1AM-Mg1_c001_N".into();
                }
                else if mpid == "MPID_Jean" || mpid == "MPID_Staluke" || mpid == "MPID_Clan" { new_engage_animation = "Mic1AM-Mg1_c501_N".into(); }
                else if mpid == "MPID_Saphir" { new_engage_animation = "Mic1AF-Mg1_c254_N".into(); }
                else { new_engage_animation = format!("Mic1A{}-Mg1_c000_N", gender_str).into(); }
            }
            7 => { 
                new_engage_animation = format!("Lyn1A{}-Bw1_c000_L", gender_str).into();
                result.right_hand = "uWep_Ft00".into();
            }   // Lyn 
            10 => { 
                new_engage_animation = format!("Cor1A{}-Ft1_c000_N", gender_str).into(); 
                result.left_hand = "uWep_Ft02".into();
            }  // Corrin
            11 => {
                if mpid == "MPID_Jean" { new_engage_animation = "Eir1AM-Sw1_c103_N".into(); }
                else if mpid == "MPID_Anna" { new_engage_animation = "Eir1AF-Sw1_c552_N".into(); }
                else { new_engage_animation = format!("Eir1A{}-Sw1_c000_N", gender_str).into(); }
            }   // Eirika
            12 => { 
                new_engage_animation = format!("Thr1A{}-Ax1_c000_N", gender_str).into(); 
                result.left_hand = "uWep_Ax30L".into();
                result.right_hand = "uWep_Ax30R".into();
                result.trail = "cEff_EngageA_Swd_00".into();
                add_accessory_to_list(accessory_list, "reserve1_loc", "uWep_Ax20");
                add_accessory_to_list(accessory_list, "reserve2_loc", "uWep_Lc21");
                add_accessory_to_list(accessory_list, "reserve3_loc", "uWep_Bw16s-Bw");
            } //Houses Unite   //Tiki
            14 => { new_engage_animation = format!("Hec1A{}-Ax1_c000_N", gender_str).into(); } // Hector
            17 => { //Camilla 
                result.ride_model = "uRig_DragR".into();
                result.ride_dress_model = "uBody_Cmi0DR_c561".into(); 
                new_engage_animation = format!("Cmi1D{}-Ax1_c000_N", gender_str).into();
            }
            18 => {     //Chrom
                if mpid == "MPID_Jean" {  new_engage_animation = "Chr1AM-Sw1_c103_N".into(); }
                else if mpid == "MPID_Mauve" || mpid == "MPID_Boucheron" { new_engage_animation = "Chr1AM-Sw1_c502_N".into(); }
                else if mpid == "MPID_Anna" { new_engage_animation = "Chr1AF-Sw1_c552_N".into(); }
                else if ( mpid == "MPID_Lueur" && gender == 2 )  || mpid == "MPID_Chloe" || mpid == "MPID_Jade" || mpid == "MPID_Ivy" || mpid == "MPID_Merin" || mpid == "MPID_Saphir" {
                    new_engage_animation = "Chr1AF-Sw1_c254_N".into();
                }
                else if mpid == "MPID_El" || mpid == "MPID_Selestia" { new_engage_animation = "Chr1AF-Sw1_c254_N".into(); }
                else { new_engage_animation = format!("Chr1A{}-Sw1_c000_N", gender_str).into(); }
            }
            20 => { new_engage_animation = format!("Thr2A{}-Ax1_c000_N", gender_str).into(); }  // Houses Unite+
            _ => { return result; }
        }
    // overwrite old engage attack animation
        for x in animation_index..list.len() { list[x as usize] = unsafe {copy_str(new_engage_animation, None) }; }
    }
    // Chapter 22 Dark Emblems
    if crate::utils::str_contains(unit.person.pid, "PID_M022_紋章士") {
        let pid = unit.person.pid.get_string().unwrap();
        for x in EMBLEM_ASSET {
            let pid2 = format!("PID_M022_紋章士_{}", x);
            if pid == pid2 {
                let gid = GameVariableManager::get_string(&format!("G_R_GID_{}", x)).get_string().unwrap();
                for y in 12..19 {
                    if gid == format!("GID_{}", EMBLEM_ASSET[y]) {
                        let god = GodData::get(&format!("GID_E006_敵{}", EMBLEM_ASSET[y])).unwrap();
                        return asset_table_result_god_setup(this, mode+10, Some(god), true, conditions, method_info);
                    }
                }
                let god = GodData::get(&gid).unwrap();
                return asset_table_result_god_setup(this, mode+10, Some(god), true, conditions, method_info);
            }
        }
    }
    result
}
//Adding Accessories to AssetTable Result's AssetTableAccessoryList 
fn add_accessory_to_list(list: &mut List<AssetTableAccessory>, model: &str, location: &str) {
    let accessory_class = Il2CppClass::from_name("App", "AssetTable").unwrap().get_nested_types().iter().find(|x| x.get_name() == "Accessory").unwrap();
    let new_accessory = Il2CppObject::<AssetTableAccessory>::from_class( accessory_class ).unwrap();
    new_accessory.model = Some(model.into() );
    new_accessory.locator = Some(location.into());
    unsafe { try_add_accessory_list(list, new_accessory, None); }
}
fn clear_accessory_from_list(list: &mut List<AssetTableAccessory>, model: &str) {
    for x in 0..list.len() {
        if list[x].model.is_none() { continue; }
        let model1 = list[x].model.unwrap().get_string().unwrap();
        if model1 == model { 
            list[x].model = None;
            list[x].locator = None;
        }
    }
}

pub fn add_mpid_condition(search: String, mpid_add: String, match_condition: bool) {
    let list = AssetTable::get_list_mut().unwrap();
    for x in 500..list.len() {
        let asset_entry = &mut list[x];
        if asset_entry.conditions.is_none() { continue; }
        let conditions: &mut Array<&Il2CppString> = asset_entry.conditions.as_mut().unwrap();
        if conditions.len() > 1  {continue; } 
        let condition_str = conditions[0].get_string().unwrap();
        if !match_condition { 
            if !str_contains( conditions[0], search.as_str()) { continue; }
        }
        else { if condition_str != search { continue; } }

        let new_condition = format!("{} | {}", condition_str, mpid_add);
        conditions[0] = new_condition.clone().into();
        println!("New Condition at {}: {}", x, new_condition);
    }
}

pub fn add_names() {
    // GID with MPID
    if unsafe { NAME_SET } { return; }
    for x in 0..19 {
        let search = format!("AID_Person_{}", enums::EMBLEM_ASSET[x as usize]);
        let mpid = format!("MPID_{}", enums::RINGS[x as usize]);
        add_mpid_condition(search, mpid, false);
    }
    add_mpid_condition("AID_Person_ディミトリ".to_string(), "MPID_Dimitri".to_string(), false);
    add_mpid_condition("AID_Person_クロード".to_string(), "MPID_Claude".to_string(), false);
    add_mpid_condition("AID_Person_ルフレ".to_string(), "MPID_Reflet".to_string(), false);
    add_mpid_condition("PID_M026_ソンブル_人型".to_string(), "MPID_Sombre".to_string(), false);
    add_mpid_condition("PID_武器屋".to_string(), "MPID_WeaponShop".to_string(), false);
    add_mpid_condition("PID_道具屋".to_string(), "MPID_ItemShop".to_string(), false);
    add_mpid_condition("PID_アクセ屋".to_string(), "MPID_AccessoriesShop".to_string(), false);
    add_mpid_condition("PID_錬成屋".to_string(), "MPID_BlackSmith".to_string(), false);

    let list = PersonData::get_list().unwrap();
    for x in 2..1000 {
        let gender = list[x].get_gender();
        if gender != 1 && gender != 2 { continue; }
        if list[x].get_asset_force() == 0 { continue; }
        if list[x].get_name().is_none() { continue; }
        if list[x].get_job().is_none() { continue; }
        let pid_str = list[x].pid.get_string().unwrap();
        if pid_str == "PID_ヴェイル_フード" { continue; }
        let name = list[x].get_name().unwrap().get_string().unwrap();
        if name == "MPID_Morph" || name == "MPID_Phantom" || name == "MPID_Veyre" { continue; }
        if name == "MPID_FileneSoldier" || name == "MPID_BrodiaSoldier" { continue; }
        if name == "MPID_IrcionSoldier" || name == "MPID_SolumSoldier" { continue; }
        if name == "MPID_Hide" || name == "MPID_SombreDragon" || name == "MPID_Il_E006" { continue; }
        let pid = list[x].pid.get_string().unwrap();
        add_mpid_condition(pid, name, true);
    }
    let list = AssetTable::get_list_mut().unwrap();
    unsafe { asset_table_on_completed_end(list[0], None); }
    unsafe { NAME_SET = true };
    println!("Finish Adding MPIDs for Name Randomization");
}


#[skyline::hook(offset=0x01bb4290)]
pub fn asset_table_result_setup_person_hook(this: &AssetTableResult, mode: i32, 
    person: Option<&PersonData>, job: Option<&JobData>, equipped: &ItemData, conditions: &Array<&Il2CppString>, method_info: OptionalMethod) -> &'static mut AssetTableResult {
        if person.is_none() && job.is_some() {
            let jid = job.unwrap().jid.get_string().unwrap();
            if jid == "JID_裏邪竜ノ娘" {
                let new_person = PersonData::get("PID_エル_竜化");
                let result = call_original!(this, mode, new_person, job, equipped, conditions, method_info);
                result.scale_stuff[0] = 0.4;
                return result;
            }
            if jid == "JID_裏邪竜ノ子" {
                let new_person = PersonData::get("PID_ラファール_竜化");
                let result = call_original!(this, mode, new_person, job, equipped, conditions, method_info);
                result.scale_stuff[0] = 0.4;
                return result;
            }
        }
        call_original!(this, mode, person, job, equipped, conditions, method_info)
    }
