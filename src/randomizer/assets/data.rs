use accessory::change_accessory;

use crate::utils::{dlc_check, str_contains};
use super::*;
use std::sync::Mutex;
static mut NAME_SET: bool  = false;
pub static UNIQUE_JOB_DATA: Mutex<Vec<UniqueJobAssets>> = Mutex::new(Vec::new());
pub static NAME_DATA: Mutex<NameData> = Mutex::new( NameData{female: Vec::new(), male: Vec::new(), act_replace: Vec::new() } );
pub static WEAPON_ASSET: Mutex<Vec<WeaponAsset>> = Mutex::new(Vec::new());
pub static mut HEAD_DATA: HeadData = HeadData{male_head: Vec::new(), female_head: Vec::new(), hair: Vec::new(),  acc_list: Vec::new(), skin: Vec::new() };
pub struct WeaponAsset {
    pub iid_index: i32,
    pub right_hand: String,
    pub left_hand: String,
}
impl WeaponAsset {
    pub fn new(line: String) -> Self {
        let values: Vec<_> = line.split_whitespace().collect();
        let right_hand = values[1];
        let left_hand = if values.len() == 3 { values[2] } else { "none" };
        let index = ItemData::get(values[0]).unwrap().parent.index;

        Self {
            iid_index: index, 
            right_hand: right_hand.to_string(),
            left_hand: left_hand.to_string()
        }
    }
}
pub struct UniqueJobAssets {
    pub jid: String,
    pub gender: i32,
    pub weapon_mask: i32,
    pub act_type: i32, 
    pub act_prefix: String,
    pub act_suffix: String,
    pub rig: String,
}
pub struct AccessoryAssets {
    pub index: i32,
    pub gender: i32,
    pub asset: String,
    pub is_head: bool,
}
impl UniqueJobAssets {
    pub fn new(line: String) -> Self {
        let values: Vec<_> = line.split_whitespace().collect();
        let jid = values[0].to_string();
        let gender = values[1].parse::<i32>().unwrap();
        if let Some(job) = JobData::get_mut(jid.as_str()) {
            let flag = job.get_flag();
            if dlc_check() && job.jid.contains("_E") {  //Avoiding adding FX enemy classes if FX isn't available
                if gender == 2 { flag.value |= 4;}
                else { flag.value |= 16; }
                flag.value |= 2;
                if flag.value & 20 == 20 { flag.value -= 20; }
            }
            else {
                if gender == 2 { flag.value |= 4;}
                else { flag.value |= 16; }
                flag.value |= 2;
                if flag.value & 20 == 20 { flag.value -= 20; }
            }
        }
        Self {
            jid: values[0].to_string(),
            gender: values[1].parse::<i32>().unwrap(),
            weapon_mask: values[2].parse::<i32>().unwrap(),
            act_type: values[3].parse::<i32>().unwrap(),
            act_prefix: values[4].to_string(),
            act_suffix: values[5].to_string(),
            rig: if values.len() > 6 { values[6] } else { "none" }.to_string()
        }
    }
}

impl AccessoryAssets {
    pub fn new(index: i32, gen: i32, asset: String, is_head: bool) -> Self {
        Self { index: index, gender: gen, asset: asset.clone(), is_head: is_head }
    }

}


pub struct NameData {
    pub female: Vec<i16>,
    pub male: Vec<i16>,
    pub act_replace: Vec<(String, String, i32)>,   // PersonIndex, Act, WeaponType
}

impl NameData {
    pub fn get_len(&self) -> usize {
        return self.female.len() + self.male.len();
    }
    pub fn add(&mut self, line: String) {
        let values: Vec<_> = line.split_whitespace().collect();
        let pid = values[0].to_string();

        if let Some(person) = PersonData::get(pid.as_str()) {
            let person_index = person.parent.index as i16;
            if person.gender == 1 { 
                if !self.male.iter().any(|&x| x == person_index) { self.male.push(person_index); }
            }
            else if person.gender == 2 { 
                if !self.female.iter().any(|&x| x == person_index) { self.female.push(person_index); }
            }
            else { return; }
            let mut counter = 1 as usize;
            loop {
                if counter >= values.len() { break; }
                if counter + 1 > values.len() { break; }
                let act = values[counter].to_string();
                let weapon = values[counter + 1].parse::<i32>().unwrap();
                let mpid = person.get_name().unwrap().to_string();
                self.act_replace.push( (mpid.clone(), act.clone(), weapon));
                counter += 2;
            }
        }
    } 
    pub fn get_random_person(&self, female: bool) -> &'static PersonData {
        let rng = Random::get_game();
        let index = 
            if female { self.female[ rng.get_value( self.female.len() as i32 ) as usize ] }
            else { self.male[rng.get_value(  self.male.len() as i32 ) as usize ]};
        return PersonData::try_index_get(index as i32).unwrap();
    }
}


pub struct HeadData {
    pub male_head: Vec<u16>,
    pub female_head: Vec<u16>,
    pub hair: Vec<(u16, bool)>,
    pub acc_list: Vec<(u16, i32, String)>,
    pub skin: Vec<SkinData>,
}

pub struct SkinData {
    pub id: u16,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub scale: [f32; 14],
    pub is_unique: bool,
    pub body: String,
    pub voice: String,
    pub rig: String,
}

impl SkinData {
    pub fn new() -> Self {
        Self {
            id: 0, r: 0, g: 0, b: 0,
            scale: [0.0; 14],
            is_unique: false,
            body: String::new(),
            voice: String::new(),
            rig: String::new(),
        }
    }
}


impl HeadData {
    pub fn reset_head_list(&mut self) {}
    pub fn add(&mut self, line: String){
        let values: Vec<_> = line.split_whitespace().collect();
        println!("Parsing {}", line);
        if values[0].contains("body"){
            let id = values[1].parse::<u16>().unwrap();
            if let Some(found) = self.skin.iter_mut().find(|s| s.id == id ) {
                found.is_unique = true;
                found.body = values[2].to_string();
                found.voice = values[3].to_string();
                if values.len() > 4 { found.rig = values[4].to_string(); }
            }
            else {
                let mut skin = SkinData::new();
                skin.id = id;
                skin.is_unique = true;
                skin.body = values[2].to_string();
                skin.voice = values[3].to_string();
                if values.len() > 4 { skin.rig = values[4].to_string(); }
                self.skin.push(skin);
            }
        }
        else if values[0].contains("acc") && values.len() >= 4 {
            if !values[3].is_empty() {
                let id = values[1].parse::<u16>().unwrap();
                match values[2] {
                    "h" => { self.acc_list.push( (id, 0, values[3].to_string() ) ); }
                    "s2" => { self.acc_list.push( (id, 1, values[3].to_string() ) ); }
                    "h2" => { self.acc_list.push( (id, 2, values[3].to_string() ) ); }
                    _ => { return; }
                }
            }
        }
        else {
            let id = values[0].parse::<u16>().unwrap();
            let is_acc = values[1].parse::<u16>().unwrap() == 1;
            let hair_id = values[2].parse::<u16>().unwrap();
    
            if id % 100 >= 50 || id == 303 { // Females are cx50+ and Rosado
                if !self.female_head.iter().any(|&f| f == id) { self.female_head.push(id); }         
            } 
            else {  // Males for cx00 to cx49
                if !self.male_head.iter().any(|&m| m == id) { self.male_head.push(id); }
            }
            if !self.hair.iter().any(|&h| h.0 == hair_id) { self.hair.push( (hair_id, is_acc) );  } 
            if values.len() == 17 && !self.skin.iter().any(|s| s.id == id) {
                let mut skin = SkinData::new();
                skin.id = id;
                for x in 0..14 { skin.scale[x] = values[3+x].parse::<f32>().unwrap();  }
                self.skin.push( skin );
            }
            else if values.len() >= 20 && !self.skin.iter().any(|s| s.id == id) {
                let mut skin = SkinData::new();
                skin.id = id;
                skin.r =  values[3].parse::<u8>().unwrap();
                skin.g =  values[4].parse::<u8>().unwrap();
                skin.b =  values[5].parse::<u8>().unwrap();
                for x in 0..14 { skin.scale[x] = values[6+x].parse::<f32>().unwrap();  }
                self.skin.push( skin );
            }
        }

    }
    pub fn replace_by_rng(&self, unit: &Unit, result: &mut AssetTableResult) {
        if unit.person.gender > 2 || unit.person.gender == 0 { return; }
        if GameVariableManager::get_number("G_GenericMode") & 1 == 0 { return; }
        let rng = Random::instantiate().unwrap();
        rng.ctor( unit.drop_seed as u32 );
        let male = unit_dress_gender(unit) == 1;
        let head = 
            if male { self.male_head[ rng.get_value( self.male_head.len() as i32 ) as usize] }
            else { self.female_head[ rng.get_value( self.female_head.len() as i32 ) as usize] };
        
        let hair = if head >= 800 || head == 504 {
            if male { 
                let hairs_m: Vec<_> = self.hair.iter().filter(|h| (h.0 % 100) < 50).collect();
                hairs_m[ rng.get_value( hairs_m.len() as i32) as usize ].0
             }
            else { 
                let hairs_f: Vec<_> = self.hair.iter().filter(|h| (h.0 % 100) >= 50).collect();
                hairs_f[ rng.get_value( hairs_f.len() as i32) as usize ].0
            }
        }
        else { head };
        let head_str = Self::pad_zeros(head);
        result.head_model = concat_string!("uHead_c", head_str).into();
        let hair_number = Self::pad_zeros(hair);

        if let Some(f_hair) = self.hair.iter().find(|h| h.0 == hair ) {   //Accessory
            if f_hair.1 {
                let model = concat_string!("uAcc_spine2_Hair", hair_number);
                result.hair_model = "uHair_null".into();
                accessory::change_accessory(result.accessory_list, model.as_str(), "c_spine1_jnt");
            }
            else {
                let hair = concat_string!("uHair_h", hair_number);
                result.hair_model = hair.into();
            }
        }
        if let Some(skin) = self.skin.iter().find(|s| s.id == head ) {
            if skin.r != 0 && skin.g != 0 && skin.b != 0 {
                result.unity_colors[2].r = skin.r as f32 / 255.0;
                result.unity_colors[2].g = skin.g as f32 / 255.0; 
                result.unity_colors[2].b  = skin.g as f32 / 255.0;
            }
            for x in 0..9 {  if skin.scale[x] > 0.0 { result.scale_stuff[x] = skin.scale[x]; } }
            for x in 9..12 { if skin.scale[x+2] > 0.0 { result.scale_stuff[x] = skin.scale[x+2]; } }
            if skin.scale[9] > 0.0 { result.scale_stuff[12] = skin.scale[9]; }
            if skin.scale[10] > 0.0 { result.scale_stuff[13] = skin.scale[10]; }

            if skin.is_unique { // AOC_Inf
                let number = if skin.voice == "_Blank" { head - 7 } else { head };
                if let Some(voice) = result.sound.voice {
                    if !skin.voice.is_empty() { result.sound.voice = Some ( skin.voice.clone().into() ); }
                }
                if skin.body.len() > 2 { 
                    let body = concat_string!("uBody_", skin.body, "_c", head_str);
                    // println!("Body Model: {} replacing {}", body, result.dress_model);
                    result.dress_model = body.clone().into(); 
                }

                if skin.rig.len() > 3 { result.body_model = concat_string!("uRig_", skin.rig).into(); }    //Rig
                //println!("Rig: {}", result.body_model);
                change_accessory(result.accessory_list, "null", "c_spine2_jnt");
                change_accessory(result.accessory_list, "null", "c_head_loc");
            }
        }
        self.acc_list.iter().filter(|acc| acc.0 == head && !acc.2.is_empty() )
            .for_each(|acc|{
                if acc.1 == 0 || acc.1 == 2  {
                    let locator = if acc.1 == 0 { "c_head_loc" } else { "c_head2_loc"};
                    let model = concat_string!("uAcc_head_", acc.2, head_str);
                    change_accessory(result.accessory_list, model.as_str(), locator);
                }
            }
        );
    }
    fn pad_zeros(number: u16) -> String {
        if number < 10 { format!("00{}", number ) }
        else if number  < 100 { format!("0{}", number ) }
        else { format!("{}", number )}
    }
}



#[unity::class("Combat", "AnimSetDB")]
pub struct AnimSetDB{
    pub parent: StructBaseFields,
    pub name: &'static Il2CppString,
}
impl Gamedata for AnimSetDB {}

pub fn add_animation_unique_classes() {
    let current_count = AssetTable::get_count() as usize;
    if unsafe { ASSET_SIZE } == 0 { 
        unsafe { ASSET_SIZE = current_count };
        println!("Un-added AssetTable has {} entries", current_count);
        let assets = include_str!("data/unique.txt").lines();
        let mut unique_table = UNIQUE_JOB_DATA.lock().unwrap();
        assets.into_iter().for_each(|line|{
            let new_line = line.to_string();
            unique_table.push(UniqueJobAssets::new(new_line));
        });
    }
}

pub fn add_names() {
    if IS_GHAST { return; }
    if unsafe { NAME_SET } { return; }

    let names = include_str!("data/names.txt").lines();
    let mut name_table = NAME_DATA.lock().unwrap();
    names.into_iter().for_each(|line|{
        let new_line = line.to_string();
        name_table.add(new_line);
    });
    unsafe { NAME_SET = true };
    return; 
}
pub fn add_weapon_assets() {
    let mut weapons = WEAPON_ASSET.lock().unwrap();
    if weapons.len() > 0 { return; }
    let weapons_data = include_str!("data/Items.txt").lines();
    weapons_data.into_iter().for_each(|line|{
        let new_line = line.to_string();
        weapons.push(WeaponAsset::new(new_line));
    });
}
pub fn add_head_data() {
    let mut head_stuff = unsafe { &mut HEAD_DATA };
    if head_stuff.male_head.len() > 0 { return; }
    let data = include_str!("data/heads.txt").lines();
    data.into_iter().for_each(|line|{
        let head_data = line.to_string();
        head_stuff.add(head_data);
    });
    head_stuff.male_head.push(504);
}

pub fn initalize_asset_data(){
    add_head_data();
    add_weapon_assets();
    add_animation_unique_classes();
    add_names();
}

pub fn get_dragon_stone_actor(item: &ItemData, job: &JobData) -> (Option<&'static PersonData>, f32) {
    if str_contains(item.iid, "IID_チキ") && item.flag.value & 128 == 0 { return (PersonData::get("PID_E001_Boss_竜化"), 1.0); }    //Tiki
    let i_item = item.iid.to_string();

    match i_item.as_str() {
        "IID_氷のブレス"|"IID_氷塊" => { return (PersonData::get("PID_遭遇戦_異形飛竜"), 1.0); },   //Corrupted Wyvern
        "IID_炎塊"|"IID_火のブレス" => { return (PersonData::get("PID_M011_異形竜"), 1.0); }  //Corrupted Wyrm
        _ => {},
    }
    let jid = job.jid.to_string();
    if jid == "JID_裏邪竜ノ娘" { (PersonData::get("PID_エル_竜化"), 0.4) }
    else if jid ==  "JID_裏邪竜ノ子" { (PersonData::get("PID_ラファール_竜化"), 0.4) }
    else { (None, 0.0) }
}
