use unity::prelude::*;
use crate::{
    randomizer::job::JOB_HASH,
    utils::dlc_check,
};
use engage::{
    mess::Mess,
    gamedata::{assettable::{AssetTable, AssetTableResult}, JobData, Gamedata},
};
use concat_string::concat_string;
use super::super::animation::WEP_PRE;
pub struct UniqueJobAssets {
    pub job_hash: i32,
    pub gender: i32,
    pub weapon_mask: i32,
    pub act_type: i32, 
    pub act_prefix: String,
    pub act_suffix: String,
    pub rig: String,
    pub alt_act: String,
    pub alt_weapon_mask: i32,
}

impl UniqueJobAssets {
    pub fn new(line: String) -> Self {
        let values: Vec<_> = line.split_whitespace().collect();
        let jid = values[0].to_string();
        let gender = values[1].parse::<i32>().unwrap();
        let mut hash = 0;
        if let Some(job) = JobData::get_mut(jid.as_str()) {
            hash = job.parent.hash;
            let flag = job.get_flag();
            if dlc_check() && job.jid.contains("_E") {  //Avoiding adding FX enemy classes if FX isn't available
                if gender == 2 { flag.value |= 4;}
                else { 
                    flag.value |= 16; 
                    if flag.value & 4 != 0 { flag.value -= 4; }
                }
                flag.value |= 2;
                if flag.value & 20 == 20 { flag.value -= 20; }
            }
            else {
                if gender == 2 { flag.value |= 4;}
                else { 
                    flag.value |= 16; 
                    if flag.value & 4 != 0 { flag.value -= 4; }
                }
                // flag.value |= 2;
                if flag.value & 20 == 20 { flag.value -= 20; }
            }
        }
        let mask2;
        let act2;
        let rig;

        match values.len() {
            8 => {
                mask2 = values[7].parse::<i32>().unwrap();
                act2 = values[6].to_string();
                rig = "none".to_string();
            }
            7 => {
                mask2 = 0;
                act2 = "-".to_string();
                rig = values[6].to_string();
            }
            _ => {
                mask2 = 0;
                act2 = "-".to_string();
                rig = "none".to_string();
            }
        };
        Self {
            job_hash: hash,
            gender: values[1].parse::<i32>().unwrap(),
            weapon_mask: values[2].parse::<i32>().unwrap(),
            act_type: values[3].parse::<i32>().unwrap(),
            act_prefix: values[4].to_string(),
            act_suffix: values[5].to_string(),
            rig: rig,
            alt_act: act2,
            alt_weapon_mask: mask2,
        }
    }
}

pub struct CustomJobAssets {
    pub job_hash: i32,
    pub body_model_m: String,
    pub dress_model_m: String,
    pub body_model_f: String,
    pub dress_model_f: String,
    pub ride_dress_model: String,
    pub uride_model: String,
    pub oride_model: String,
    pub uas_anim: String,
    pub body_anims_m: Vec<(String, i32)>,
    pub body_anims_f: Vec<(String, i32)>,
    pub gender: i32,
}

impl CustomJobAssets {
    pub fn new(jobhash: i32) -> Self {
        CustomJobAssets{ 
            job_hash: jobhash,
            body_model_m: String::new(), 
            body_model_f: String::new(), 
            dress_model_m: String::new(), 
            dress_model_f: String::new(),
            ride_dress_model: String::new(),
            uride_model: String::new(),
            oride_model: String::new(),
            uas_anim: String::new(),
            body_anims_m: Vec::new(),
            body_anims_f: Vec::new(),
            gender: 0,
        }
    }
    pub fn replace(&self, result: &mut AssetTableResult, gender: i32, mode: i32, kind: i32, engaged: bool) {
        if mode == 2  {
            if result.dress_model.contains("Swd0A") {
                if gender == 1 && !self.dress_model_m.is_empty() { result.dress_model = self.dress_model_m.as_str().into(); }
                else if gender == 2 && !self.dress_model_f.is_empty() { result.dress_model = self.dress_model_f.as_str().into(); }
            }
        }
        else if mode == 1 {
            if result.body_model.contains("oBody_Swd0A"){
                if gender == 1 && !self.body_model_m.is_empty() { result.body_model = self.body_model_m.as_str().into();  }
                else if gender == 2 && !self.body_model_f.is_empty() { result.body_model = self.body_model_f.as_str().into();  }
            }
        }
        if !engaged {
            if mode == 2 {
                if !self.uride_model.is_empty() { result.ride_model = self.uride_model.as_str().into(); }
                if !self.ride_dress_model.is_empty() { result.ride_dress_model = self.ride_dress_model.as_str().into(); }
                if kind >= 0 {
                    if gender == 1 {
                        self.body_anims_m.iter().filter(|body| body.1 == kind).for_each(|anim|{
                            result.body_anims.add(Il2CppString::new_static(anim.0.as_str()));
                        });
                    }
                    else if gender == 2 {
                        self.body_anims_f.iter().filter(|body| body.1 == kind).for_each(|anim|{
                            result.body_anims.add(Il2CppString::new_static(anim.0.as_str()));
                        });
                    }
                }
                if !self.uride_model.is_empty() { result.ride_model = self.uride_model.as_str().into(); }
                if !self.ride_dress_model.is_empty() { result.ride_dress_model = self.ride_dress_model.as_str().into(); }
            }
            else {
                if !self.oride_model.is_empty() {  result.ride_model = self.oride_model.as_str().into(); }
                if !self.uas_anim.is_empty() { 
                    result.body_anim = Some(self.uas_anim.as_str().into()); 
                    if self.uas_anim.contains("DM") {
                        result.ride_anim = Some(self.uas_anim.replace("DM", "DR").into());
                        result.scale_stuff[18] = 0.5;
                    }
                    else if self.uas_anim.contains("DF") {
                        result.ride_anim = Some(self.uas_anim.replace("DF", "DR").into());
                        result.scale_stuff[18] = 0.5;
                    }
                    else if self.uas_anim.contains("BM") {
                        result.ride_anim = Some(self.uas_anim.replace("BM", "BR").into());
                    }
                    else if self.uas_anim.contains("BF") {
                        result.ride_anim = Some(self.uas_anim.replace("BF", "BR").into());
                    }
                    else if self.uas_anim.contains("FM") {
                        result.ride_anim = Some(self.uas_anim.replace("FM", "FR").into());
                        result.scale_stuff[18] = 0.5;
                    }
                    else if self.uas_anim.contains("FF") {
                        result.ride_anim = Some(self.uas_anim.replace("FF", "FR").into());
                        result.scale_stuff[18] = 0.5;
                    }
                    else if self.uas_anim.contains("CM") {
                        result.ride_anim = Some(self.uas_anim.replace("CM", "CR").into());
                    }
                    else if self.uas_anim.contains("CF") {
                        result.ride_anim = Some(self.uas_anim.replace("CF", "CR").into());
                    }
                }
            }
        }
    }
}
pub fn get_animation_unique_classes() -> Vec<UniqueJobAssets> {
    let mut data: Vec<UniqueJobAssets> = Vec::new();
    include_str!("data/unique.txt").lines().into_iter()
        .for_each(|line|{
            let new_line = line.to_string();
            data.push(UniqueJobAssets::new(new_line));
        }
    );
    println!("Initialized unique class asset data");
    data
}

pub fn get_custom_class_assets() -> Vec<CustomJobAssets> {
    let custom_list: Vec<_> = JobData::get_list().unwrap().iter()
        .filter(|job| !JOB_HASH.iter().any(|h| *h == job.parent.hash))
        .collect();

    let asset_table: Vec<_> = AssetTable::get_list().unwrap().iter()
        .filter(|entry| entry.conditions.as_ref().is_some_and(|f| 
            f.iter().any(|con| con.contains("JID_")))).collect();
    let anim_list = super::super::AnimSetDB::get_list().unwrap();
    let mut list: Vec<CustomJobAssets> = Vec::new();
    custom_list.iter().for_each(|job|{
        let jid = job.jid;
        let mut custom_asset = CustomJobAssets::new(job.parent.hash);
        let mut filled = false;
        let mut gender = 0;
        asset_table.iter()
            .filter(|entry| 
                entry.conditions.as_ref().is_some_and(|array| 
                    array.iter().any(|con| *con == jid)
                )
            )
            .for_each(|entry|{
                if let Some(dress) = entry.dress_model {
                    if dress.contains("F_c") { 
                        custom_asset.dress_model_f = dress.to_string(); 
                        gender |= 2;
                        filled = true;
                    }
                    else if dress.contains("M_c") { 
                        custom_asset.dress_model_m = dress.to_string();
                        gender |= 1;
                        filled = true
                    }
                }
                if let Some(body) = entry.body_model {
                    if body.contains("F_c") {
                        custom_asset.body_model_f = body.to_string(); 
                        filled = true;
                    }
                    else if body.contains("M_c") { 
                        custom_asset.body_model_f = body.to_string(); 
                        filled = true;
                    }
                }
                if let Some(ride) = entry.ride_model {
                    if ride.contains("oBody") { custom_asset.oride_model = ride.to_string(); filled = true; }
                    else if ride.contains("uRig") { custom_asset.uride_model = ride.to_string();  filled = true; }
                }
                if let Some(ride) = entry.ride_dress_model {
                    if ride.contains("uBody") { custom_asset.ride_dress_model = ride.to_string(); filled = true; }
                }
                if let Some(body_anim) = entry.body_anim {
                    println!("Body: {}", body_anim);
                    let body = body_anim.to_string();
                    if body_anim.contains("UAS_") { custom_asset.uas_anim = body; }
                    else if body_anim.contains("M-#_") {  
                        let pos = body.find("#");
                        let s1 = body.split_at(pos.unwrap());
                        for x in 0..9 {
                            let weapon = concat_string!(s1.0, WEP_PRE[x]);
                            if let Some(anim) = anim_list.iter().find(|x| x.name.contains(weapon.as_str())) {
                                custom_asset.body_anims_m.push( (anim.name.to_string(), x as i32));
                            }
                        }
                    }
                    else if body_anim.contains("M-") {  
                        for x in 0..9 {
                            if body_anim.contains(WEP_PRE[x]) {
                                custom_asset.body_anims_m.push( (body_anim.to_string(), x as i32 ));
                                break;
                            }
                        }
                    }
                    else if body_anim.contains("F-#_") {  
                        let pos = body.find("#");
                        let s1 = body.split_at(pos.unwrap());
                        for x in 0..9 {
                            let weapon = concat_string!(s1.0, WEP_PRE[x]);
                            if let Some(anim) = anim_list.iter().find(|x| x.name.contains(weapon.as_str())) {
                                custom_asset.body_anims_f.push( (anim.name.to_string(), x as i32 ));
                            }
                        }
                    }
                    else if body_anim.contains("F-") {  
                        for x in 0..9 {
                            if body_anim.contains(WEP_PRE[x]) {
                                custom_asset.body_anims_f.push( (body_anim.to_string(), x as i32 ));
                                break;
                            }
                        }
                    }
                }
            }
        );
        if filled {
            custom_asset.gender = gender;
            println!("Custom Class {} Assets have been added", Mess::get_name(jid));
            list.push(custom_asset);
        }
    });
    println!("Custom Class Assets added: {}", list.len());
    list
}