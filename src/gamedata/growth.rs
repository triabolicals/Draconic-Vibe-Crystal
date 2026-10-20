use engage::app::capabilitybase_1::*;
use super::*;
use crate::utils::to_opt;

pub struct GrowthData {
    person_stats: [Vec<u8>; 10],
    job_stat: [Vec<i8>; 10],
    person_grow: Vec<(i32, [u8; 10])>,
    job_grow: Vec<[i8; 10]>,
    job_limit: Vec<[u8; 10]>,
    person_cap: Vec<(i32, [i8; 10])>,
}
impl GrowthData {
    pub fn new() -> GrowthData{
        let mut person_stats = [const { Vec::new() }; 10];
        let mut job_stat = [const { Vec::new() }; 10];
        let person_grow =
            PersonData::get_list().iter().filter(|x| !x.get_grow().is_zero() )
                .map(|x|{
                    let grow = x.get_grow();
                    let mut g = [0; 10];
                    for x in 0..10 {
                        let v = grow.get_item(x as i32);
                        g[x] = v;
                        let s = if v > 100 { 20 } else { v/5 } + 1;
                        person_stats[x].push(s);
                    }
                    (x.index(), g)
                }).collect();
        let person_cap =
            PersonData::get_list().iter().filter(|x| !x.get_limit().is_zero() )
                .map(|x|{
                    let mut g = [0; 10];
                    let limit = x.get_limit();
                    for x in 0..10 { g[x] = limit.get_item(x as i32); }
                    (x.index(), g)
                }).collect();

        let job_grow =
            JobData::get_list().iter().map(|x|{
                let mut g = [0; 10];
                let diff = x.get_diff_grow();
                for x in 0..10 {
                    let v = diff.get_item(x as i32);
                    g[x] = v;
                    let s = if v > 25 { 5 } else if v < -25 { -5 } else { v/5 };
                    job_stat[x].push(s);
                }
                g
            }).collect();
        let job_limit =
            JobData::get_list().iter().map(|x|{
                let mut g = [0; 10];
                let diff = x.get_limit();
                for x in 0..10 { g[x] = diff.get_item(x as i32); }
                g
            }).collect();
        Self{ person_grow, job_stat, job_grow, person_stats, person_cap, job_limit }
    }
    pub fn get_starting_class_key(person_data: PersonData) -> String { format!("G_JG_{}", person_data.get_pid()) }
    pub fn set_temp_key(person_data: PersonData, value: i32) {
        let key = format!("TJG_{}", person_data.get_pid());
        if !GameVariableManager::is_exist(key.as_str()) { GameVariableManager::make_entry_norewind(key.as_str(), value); }
        else { GameVariableManager::set_number(key.as_str(), value); }
    }
    pub fn set_starting_class(unit: Unit) { Self::set_starting_class_2(unit.get_person(), unit.get_job()); }
    pub fn set_starting_class_2(person_data: PersonData, job_data: JobData) { Self::set_starting_class_3(person_data, job_data.hash()); }
    pub fn set_starting_class_3(person_data: PersonData, hash: i32) {
        let key = Self::get_starting_class_key(person_data);
        if !GameVariableManager::is_exist(key.as_str()) { GameVariableManager::make_entry_norewind(key.as_str(), hash); }
        else { GameVariableManager::set_number(key.as_str(), hash); }
    }
    pub fn get_starting_class(unit: Unit) -> Option<JobData> {
        let key = Self::get_starting_class_key(unit.get_person());
        if GameVariableManager::is_exist(key.as_str()) {
            let job = JobData::try_get_from_hash(GameVariableManager::get_number(key.as_str()));
            if job.is_null() { None } else { Some(job) }
        }
        else { None }
    }
    pub fn is_magic_class(job: JobData) -> bool {
        (job.weapons().get(6) == 1 || job.weapons().get(7) == 1) &&
            job.weapon_levels().iter().enumerate().map(|(i, v)| (i, v.value))
                .max_by(|i1, i2| i1.1.partial_cmp(&i2.1).unwrap()).is_some_and(|v| v.0 == 6 || v.0 == 7)
    }
    pub fn adaptive_grow(person: PersonData) {
        if !DVCFlags::AdaptiveGrowths.get_value() { return; }
        let key = Self::get_starting_class_key(person);
        let job = JobData::try_get_from_hash(GameVariableManager::get_number(key.as_str()));
        let is_magic = if job.is_null() { Self::is_magic_class(person.get_job()) } else { Self::is_magic_class(job) };
        let grow = person.get_grow();
        let (p_str, p_mag) = (grow.get_item(1), grow.get_item(6));
        if is_magic == (p_mag < p_str) {
            grow.set_item(1, p_mag);
            grow.set_item(6, p_str);
        }
    }
    pub fn calculate_growth(&self, hash: i32, mode: i32) -> [u8; 10] {
        let rng = DVCVariables::create_hash_rng(hash, 2);
        let mut out = [0u8; 10];
        match mode {
            1|2 => {
                let cap_range = if mode != 2 { (500, 250) } else { (2550, 5 )};
                let mut total = 0;
                while total < cap_range.1 || total > cap_range.0 {
                    total = 0;
                    for i in 0..10 {
                        let v = 5 * self.person_stats[i].get_random_element(rng).map(|u| *u).unwrap();
                        total += v as i32;
                        out[i] = v;
                    }
                }
            }
            3 => {
                if let Some(data) = self.person_grow.iter().find(|x| x.0 == hash){
                    let mut grows: Vec<u8> = data.1.iter().map(|&x| x).collect();
                    grows.remove(9);
                    for i in 0..9 {
                        if let Some(v) = grows.get_remove(rng) { out[i] = v; }
                    }
                    out[9] = data.1[9];
                }
            }
            4 => {
                for i in 0..10 {
                    let rn = rng.get_value_2(32);
                    let mut v = (rng.get_value_2(11) as u8) * 5;
                    for i in 1..5 { if rn < (16 >> i) { v += 50; } }
                    out[i] = v;
                }
            }
            _ => {
                if let Some(data) = self.person_grow.iter().find(|x| x.0 == hash){
                    for i in 0..10 { out[i] = data.1[i]; }
                }
            }
        }
        out
    }
    pub fn update_personal_grow(&self) {
        let grow_mode = DVCVariable::PersonalGrowthMode.get_value();
        match grow_mode {
            1|2|4 => {
                self.person_grow.iter().for_each(|x| {
                    let ran_grow = self.calculate_growth(x.0, grow_mode);
                    let person = PersonData::try_get_2(x.0);
                    let person_grow = person.get_grow();
                    for i in 0..10 { person_grow.set_item(i as i32, ran_grow[i]); }
                    Self::adaptive_grow(person);
                });
            }
            3 => {
                self.person_grow.iter().for_each(|x| {
                    let mut grows: Vec<u8> = x.1.iter().map(|&x| x).collect();
                    grows.remove(9);
                    let person = PersonData::try_get_2(x.0);
                    let grow = person.get_grow();
                    let rng = DVCVariables::create_hash_rng(person.hash(), 2);
                    for i in 0..9 {
                        if let Some(v) = grows.get_remove(rng) { grow.set_item(i, v); }
                    }
                    Self::adaptive_grow(person);
                });
            }
            _ => {
                self.person_grow.iter().for_each(|x|{
                    let person = PersonData::try_get_2(x.0);
                    let grow = person.get_grow();
                    for i in 0..10 { grow.set_item(i as i32, x.1[i]); }
                    Self::adaptive_grow(person);
                })
            }
        }
    }
    pub fn update_job_diff(&self){
        if DVCFlags::RandomClassGrowth.get_value() {
            JobData::get_list().iter().for_each(|j|{
                let diff_grow = j.get_diff_grow();
                let rng = DVCVariables::create_hash_rng(j.hash(), 2);
                for i in 0..10 {
                    let v = 5 * self.job_stat[i].get_random_element(rng).map(|u| *u).unwrap();
                    diff_grow.set_item(i as i32, v);
                }
            });
        }
        else {
            JobData::get_list().iter().zip(self.job_grow.iter()).for_each(|(j,x)| {
                let diff = j.get_diff_grow();
                for i in 0..10{ diff.set_item(i as i32, x[i]); }
            })
        }
    }
    pub fn update_personal_caps(&self) {
        if DVCFlags::PersonalCaps.get_value() {
            self.person_cap.iter().for_each(|x|{
                if let Some(person) = to_opt(PersonData::try_get_2(x.0)) {
                    let rng = DVCVariables::create_hash_rng(person.hash(), 2);
                    let limit = person.get_limit();
                    for x in 0..8 {
                        let v = rng.get_value_2(11) - 5;
                        limit.set(x, v as i8);
                    }
                }
            });
        }
        else {
            self.person_cap.iter().for_each(|xx|{
                if let Some(person) = to_opt(PersonData::try_get_2(xx.0)){
                    let limit = person.get_limit();
                    for x in 0..10 { limit.set_item(x as i32, xx.1[x]); }
                }
            });
        }
    }
    pub fn update_job_cap(&self) {
        if DVCFlags::MaxStatCaps.get_value() {
            JobData::get_list().iter().for_each(|j| {
                let limit = j.get_limit();
                let base = j.get_base();
                for i in 0..10 { limit.set_item(i, 120 + base.get(i)); }
            });
        }
        else {
            JobData::get_list().iter().zip(self.job_limit.iter()).for_each(|(j, x)| {
                let limit = j.get_limit();
                for i in 0..10 { limit.set_item(i, x[i as usize] ); }
            });
        }
    }
    pub fn update(&self) {
        self.update_personal_grow();
        self.update_personal_caps();
        self.update_job_diff();
        self.update_job_cap();
    }
    pub fn reset(&self) {
        JobData::get_list().iter().zip(self.job_limit.iter()).for_each(|(j, x)| {
            let limit = j.get_limit();
            for i in 0..10 { limit.set(i, x[i as usize]); }
        });
        self.person_cap.iter().for_each(|xx|{
            if let Some(person) = to_opt(PersonData::try_get_2(xx.0)){
                let limit = person.get_limit();
                for x in 0..10 { limit.set(x, xx.1[x as usize]); }
            }
        });
        JobData::get_list().iter().zip(self.job_grow.iter()).for_each(|(j,x)| {
            let diff = j.get_diff_grow();
            for i in 0..10{ diff.set_item(i as i32,x[i]); }
        });
        self.person_grow.iter().for_each(|x|{
            let person = PersonData::try_get_2(x.0);
            let grow = person.get_grow();
            for i in 0..10 { grow.set(i as i32, x.1[i]); }
            Self::adaptive_grow(person);
        });
    }
}