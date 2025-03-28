use unity::prelude::*;
use super::*;
use engage::gamedata::{assettable::{AssetTable, AssetTableResult}, Gamedata};
use super::super::animation::WEP_PRE;

#[derive(Copy, Clone, PartialEq)]
pub enum Mount {
    None,
    Cav,
    Wolf,
    Pegasus,
    Griffin,
    Wyvern,
}


pub struct JobAssetSets {
    pub job_hash: i32,
    pub mode: i32,
    pub mount: Mount,
    pub unique: bool,
    pub dragon_stone: bool,
    pub cannon: bool,
    pub gender_flag: i32,
    pub entries: Vec<i32>,
}

impl JobAssetSets {
    pub fn get_dress(&self, gender: Gender, is_morph: bool) -> Option<&'static Il2CppString> {
        let gen = if gender == Gender::Male { "M_c" } else { "F_c" };
        let morph = "_c70";
        if is_morph {
            if let Some(entry) = self.entries.iter().flat_map(|&index| AssetTable::try_index_get(index))
            .find(|entry| entry.dress_model.is_some_and(|x|{ let xt = x.to_string();  xt.contains(gen) && xt.contains(morph) })) {
                return entry.dress_model;
            }
        }
        if let Some(entry) = self.entries.iter().flat_map(|&index| AssetTable::try_index_get(index))
        .find(|entry| entry.dress_model.is_some_and(|x| x.to_string().contains(gen))){    //uBody_xxx#G_c
            return entry.dress_model;
        }
        return None;
    }
    pub fn get_ride_dress(&self, is_morph: bool) -> Option<&'static Il2CppString> {
        let morph = "_c70";
        if is_morph {
            if let Some(entry) = self.entries.iter().flat_map(|&index| AssetTable::try_index_get(index))
            .find(|entry| entry.ride_dress_model.is_some_and(|x| x.to_string().contains(morph) )) {
                return entry.ride_dress_model;
            }
        }
        if let Some(entry) = self.entries.iter().flat_map(|&index| AssetTable::try_index_get(index))
        .find(|entry| entry.ride_dress_model.is_some_and(|x| !x.to_string().contains(morph) )){
            return entry.ride_dress_model;
        }
        return None;
    }
    pub fn get_body_rig(&self, gender: Gender) -> Option<&'static Il2CppString> {
        let gen = if gender == Gender::Male { "M_c" } else { "F_c" };
        if let Some(entry) = self.entries.iter().flat_map(|&index| AssetTable::try_index_get(index))
        .find(|entry| entry.mode == 2 && entry.body_model.is_some() && entry.dress_model.is_some_and(|x| x.contains(gen))) {
            return entry.body_model;
        }
        return None;
    }
    pub fn get_obody(&self, gender: Gender, is_morph: bool) -> Option<&'static Il2CppString> {
        let morph = "_c70";
        let gen = if gender == Gender::Male { "M_c" } else { "F_c" };
        if is_morph {
            if let Some(entry) = self.entries.iter().flat_map(|&index| AssetTable::try_index_get(index))
            .find(|entry| self.mode == 1 && entry.body_model.is_some_and(|x|{ let xt = x.to_string();  xt.contains(gen) && xt.contains(morph) })) {
                return entry.body_model;
            }
        }
        if let Some(entry) = self.entries.iter().flat_map(|&index| AssetTable::try_index_get(index))
        .find(|entry| self.mode == 1 && entry.body_model.is_some_and(|x|{ let xt = x.to_string();  xt.contains(gen) && !xt.contains(morph)})){
            return entry.body_model;
        }
        return None;
    }
    pub fn get_ride_rig(&self) -> Option<&'static Il2CppString> {
        if let Some(entry) = self.entries.iter().flat_map(|&index| AssetTable::try_index_get(index))
        .find(|entry| entry.mode == 2 && entry.ride_model.is_some()) { return entry.ride_model;  }
        return None;
    }
    pub fn get_ride_obody(&self, is_morph: bool) -> Option<&'static Il2CppString> {
        let morph = "_c70";
        if is_morph {
            if let Some(entry) = self.entries.iter().flat_map(|&index| AssetTable::try_index_get(index))
            .find(|entry| entry.mode == 1 && entry.ride_model.is_some_and(|x| x.to_string().contains(morph))) { 
                return entry.ride_model 
            }
        }
        if let Some(entry) = self.entries.iter().flat_map(|&index| AssetTable::try_index_get(index))
        .find(|entry|  entry.mode == 1 && entry.ride_model.is_some_and(|x|  !x.to_string().contains(morph) )){
            return entry.ride_model;
        }
        return None;
    }
    pub fn get_acc(&self, gender: Gender, mode: i32, locator: &str) -> Option<&'static AssetTableAccessory> {
        let gen_str = create_anim_type(self.mount, gender);
        self.entries.iter().flat_map(|&index| AssetTable::try_index_get(index))
            .filter(|entry| entry.mode == mode)
            .flat_map(|entry| entry.accessory_list.list.iter())
            .find(|acc| 
                acc.model.is_some_and(|model| model.to_string().contains(gen_str.as_str())) && 
                acc.locator.is_some_and(|loc| loc.to_string() == locator)
            )
            .map(|v| &**v)
    }
    pub fn get_body_anims(&self, result: &mut AssetTableResult, kind: i32, gender: Gender, is_morph: bool) {
        let search = create_anim_type(self.mount, gender);
        if self.mode == 1 {
            if let Some(a) = self.entries.iter().flat_map(|&i| AssetTable::try_index_get(i))
            .find(|entry| entry.body_anim.is_some_and(|x| x.to_string().contains(search.as_str())) ){
                result.body_anim = a.body_anim;
                result.body_anims.add(Il2CppString::new_static(a.body_anim.unwrap().to_string()));
                if a.ride_anim.is_some() { result.ride_anim = a.ride_anim };
            }
            return;
        }
        let gen = if gender == Gender::Male { "M-"} else { "F-" };
        let engage = AssetTableStaticFields::get_condition_index("エンゲージ技");
        let dragonstone = AssetTableStaticFields::get_condition_index("竜石");
        let engage2 = AssetTableStaticFields::get_condition_index("エンゲージ中");
        let bullet = AssetTableStaticFields::get_condition_index("弾丸");
        // let weapon_kind_index = SEARCH_LIST.get().unwrap().weapon_conditions[kind as usize];
        let mut custom_made = false;
        self.entries.iter().flat_map(|&i| AssetTable::try_index_get(i))
            .filter(|entry| weapon_condition_met(entry, kind) && 
                !has_condition(entry, engage) && !has_condition(entry, dragonstone) && !has_condition(entry, engage2) &&
                !has_condition(entry, bullet) &&
                entry.body_anim.is_some_and(|x| x.to_string().contains(gen))
            )
            .for_each(|entry|{
                // println!("Entry Added: {} (Line: {}) Kind: {}", entry.parent.index, 90 + entry.parent.index, kind);
                let body_anim = entry.body_anim.unwrap().to_string();
                // println!("Body_Anim: {}", body_anim.as_str());
                if body_anim.contains("-#") {
                    let new_body = body_anim.replace("#", WEP_PRE[kind as usize]).replace("_c", "1_c");
                    if super::super::animation::anim_exists(new_body.as_str()) {
                        result.body_anims.add(Il2CppString::new_static(new_body));
                    }
                    else if !custom_made {
                        let new_anim = super::super::animation::create_anim_from_mount(self.mount, gender, kind);
                        result.body_anims.add(Il2CppString::new_static(new_anim));
                        custom_made = true;
                    }
                }
                else { 
                    result.body_anims.add(Il2CppString::new_static(body_anim));
                }
                if entry.right_hand.is_some_and(|str| str.to_string().contains("00")) { result.right_hand = entry.right_hand.unwrap(); }
                if entry.left_hand.is_some_and(|str| str.to_string().contains("00")) { result.left_hand = entry.left_hand.unwrap(); }
            }
        );
    }
    pub fn get_map_wing_scaling(&self) -> Option<f32> {
        if self.mode == 2 { return None; }
        if let Some(entry) = self.entries.iter().flat_map(|&index| AssetTable::try_index_get(index)).find(|entry|  entry.mode == 1 && entry.scale_stuff[18] > 0.10)
        {
            return Some(entry.scale_stuff[18]);
        }
        else { return None; }
    }
    pub fn get_map_all_scaling(&self) -> Option<f32> {
        if self.mode == 2 { return None; }
        if let Some(entry) = self.entries.iter().flat_map(|&index| AssetTable::try_index_get(index)).find(|entry|  entry.mode == 1 && entry.scale_stuff[16] > 0.10)
        {
            return Some(entry.scale_stuff[16]);
        }
        else { return None; }
    }
}

pub fn get_job_entries(table: &mut JobAssetSets, mode: i32, jid: &Il2CppString) -> bool {
    let asset_table_sf = AssetTableStaticFields::get();
    let jid_index  = AssetTableStaticFields::get_condition_index(jid);
    let male_con = AssetTableStaticFields::get_condition_index("男装");
    let female_con = AssetTableStaticFields::get_condition_index("女装");

    table.entries.clear();
    table.mode = mode;
    let mut female = false;
    let mut male = false;
    asset_table_sf.search_lists[mode as usize].iter()
        .filter(|entry| entry.condition_indexes.list.iter().any(|s| s.iter().any(|&x| x == jid_index)))
            //entry.condition_indexes.list.iter().flat_map(|s| s.iter().any(|&index| jid_index == index)))
        .for_each(|entry|{
            if table.mount == Mount::None { table.mount = determine_mount(entry); }
            male |= has_condition(entry, male_con);
            female |= has_condition(entry, female_con);
            if entry.dress_model.is_some_and(|x| x.to_string().contains("M_c")) { table.gender_flag |= 1; }
            if entry.dress_model.is_some_and(|x| x.to_string().contains("F_c")) { table.gender_flag |= 2; }
            table.entries.push(entry.parent.index);
        }
    );
    table.unique = !(male && female);
    table.entries.len() > 0
}

pub fn determine_mount(entry: &AssetTable) -> Mount {
    let ride =   
    if entry.mode == 1 { 
        if entry.ride_anim.is_some() { entry.ride_anim.unwrap().to_string() }
        else if entry.ride_model.is_some() {  entry.ride_model.unwrap().to_string() }
        else if entry.body_anim.is_some() { entry.body_anim.unwrap().to_string() }
        else { String::new() }
    }
    else {
        if entry.ride_anim.is_some() { entry.ride_anim.unwrap().to_string() }
        else if entry.ride_dress_model.is_some() { entry.ride_dress_model.unwrap().to_string() }
        else if entry.body_anim.is_some() {  entry.body_anim.unwrap().to_string() }
        else if entry.dress_model.is_some() { entry.dress_model.unwrap().to_string() }
        else { String::new() }
    };

    if ride.contains("BM") || ride.contains("BF") || ride.contains("BR") { return Mount::Cav; }
    if ride.contains("CM") || ride.contains("CF") || ride.contains("CR") { return Mount::Wolf; }
    if ride.contains("DM") || ride.contains("DF") || ride.contains("DR") { return Mount::Wyvern; }
    if ride.contains("EF") || ride.contains("ER")  { return Mount::Pegasus; }
    if ride.contains("FM") || ride.contains("FM") || ride.contains("FR") { return Mount::Griffin; }

    return Mount::None;
}   
pub fn determine_mount_str(ride: &str) -> Mount {
    if ride.contains("BM") || ride.contains("BF") || ride.contains("BR") { return Mount::Cav; }
    if ride.contains("CM") || ride.contains("CF") || ride.contains("CR") { return Mount::Wolf; }
    if ride.contains("DM") || ride.contains("DF") || ride.contains("DR") { return Mount::Wyvern; }
    if ride.contains("EF") || ride.contains("ER")  { return Mount::Pegasus; }
    if ride.contains("FM") || ride.contains("FM") || ride.contains("FR") { return Mount::Griffin; }
    return Mount::None;
}
pub fn determine_gender_str(ride: &str) -> Gender {
    if ride.contains("AT") { return Gender::Other; }
    if ride.contains("BM") || ride.contains("CM") || ride.contains("FM") || ride.contains("AM") || ride.contains("DM") { return Gender::Male; }
    if ride.contains("BF") || ride.contains("CF") || ride.contains("FF") || ride.contains("AF") || ride.contains("DF") || ride.contains("EF") { return Gender::Female; }
    return Gender::None;
}

pub fn weapon_condition_met(entry: &AssetTable, kind: i32) -> bool {
    let weapon_condition_index = &SEARCH_LIST.get().unwrap().weapon_conditions;
    let selected_weapon = weapon_condition_index[kind as usize];
    let not_selected_weapon = selected_weapon + 0xFFF;
    if entry.condition_indexes.list.iter().any(|search| search.iter().any(|&i| i == not_selected_weapon)) { return false; } // If not weapon kind
    if entry.condition_indexes.list.iter().any(|search| search.iter().any(|&i| i == selected_weapon)) { return true; }  // If Weapon kind
    !entry.condition_indexes.list.iter().any(|search| search.iter().any(|&lhs| weapon_condition_index.iter().any(|&rhs| lhs == rhs && selected_weapon != rhs ))) // if other weapon kinds 
}

pub fn create_anim_type(mount: Mount, gender: Gender) -> String {
    match (mount, gender) {
        (Mount::None, Gender::Male) =>  { "AM" }
        (Mount::None, Gender::Female) => { "AF" }
        (Mount::Cav, Gender::Male) =>  { "BM" }
        (Mount::Cav, Gender::Female) => { "BF" }
        (Mount::Wolf, Gender::Male) =>  { "CM" }
        (Mount::Wolf, Gender::Female) => { "CF" }
        (Mount::Wyvern, Gender::Male) =>  { "DM" }
        (Mount::Wyvern, Gender::Female) => { "DF" }
        (Mount::Pegasus, Gender::Female) =>  { "EF" }
        (Mount::Griffin, Gender::Male) => { "FM"}
        (Mount::Griffin, Gender::Female) =>{ "FM"}
        (_, Gender::Other) => {"AT" }
        (_, _) => {"" }
    }.to_string()
}

