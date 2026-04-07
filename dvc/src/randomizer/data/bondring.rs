use engage::gamedata::Gamedata;
use engage::gamedata::ring::RingData;
use engage::gamedata::skill::{SkillData, SkillDataCategorys};
use crate::config::DVCFlags;
use crate::randomizer::data::SkillsList;
use crate::randomizer::data::sync::get_highest_priority;
use crate::randomizer::{get_data_read, Randomizer};
use crate::utils::get_rng;

pub struct BondRingData {
    pub hash: i32,
    pub skills: SkillsList,
    pub stats: [i8; 11],
}

impl BondRingData {
    pub fn from_data(ring_data: &RingData) -> Self {
        let mut stats = [0; 11];
        for x in 0..11 { stats[x] = ring_data.enhance[x]; }
        Self {
            hash: ring_data.parent.hash, stats,
            skills: SkillsList::from_skill_array(ring_data.get_equip_skills()),
        }
    }
    pub fn reset(&self) {
        if let Some(data) = RingData::try_get_hash_mut(self.hash).as_mut() {
            for x in 0..11 { data.enhance[x] = self.stats[x]; }
            self.skills.set_skill_array(data.get_equip_skills());
        }
    }
}

pub fn randomize_bond_ring_skills() {
    let ring_list = RingData::get_list_mut().unwrap();
    let ranks = [3, 2, 1, 0];
    let ranks_rate = crate::DeploymentConfig::get().get_bond_ring_rates();
    let none = ranks_rate.iter().filter(|x| **x > 0).count() == 0;
    let data = get_data_read();
    println!("Bond Rings Rates: S: {} A: {} B: {} C: {}: Bond Ring: {}", ranks_rate[0], ranks_rate[1], ranks_rate[2], ranks_rate[3], DVCFlags::BondRing.get_value());
    if DVCFlags::Initialized.get_value() { data.bond_ring.iter().for_each(|r| { r.reset(); }); }
    if !DVCFlags::BondRing.get_value() || none { return; }
    if DVCFlags::BondRing.get_value() {
        let rng_rings = get_rng();
        ring_list.iter_mut().for_each(|ring| { ring.get_equip_skills().clear(); });
        for y in 0..4 {
            let current_rank = ranks[y as usize];
            let odds = ranks_rate[y as usize];
            if odds == 0 { continue; }
            let mut pool = data.skill_pool.pool.clone();
            ring_list.iter_mut()
                .filter(|ring| ring.rank == current_rank)
                .for_each(|ring| {
                    let equip_skills = ring.get_equip_skills();
                    let mut skill_count = 0;
                    let mut skill_odds = odds;
                    while rng_rings.get_value(100) < skill_odds && skill_count < 4 {
                        if let Some(skill) = pool.get_remove(rng_rings).and_then(|s| SkillData::try_get_hash(s)).map(|s| get_highest_priority(s)) {
                            let highest = get_highest_priority(skill);
                            if highest.name.is_none() { continue; }
                            equip_skills.add_skill(highest, SkillDataCategorys::Ring, 0);
                            skill_count += 1;
                        } else { break; }  // no more skills
                        skill_odds = (1 / (1 + skill_count + y)) * skill_odds + (10 - y) * current_rank;
                    }
                });
        }
    }
    if DVCFlags::RingStats.get_value() {
        let rng_rings = get_rng();
        RingData::get_list_mut().unwrap().iter_mut().for_each(|ring| {
            for x in 0..11 { ring.enhance[x] = 0; }
            let mut s = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 10];
            let mut order = vec![];
            if let Some(ss) = s.get_remove(rng_rings) { order.push(ss); } // stat 1
            if let Some(ss) = s.get_remove(rng_rings) { order.push(ss); } // stat 2
            for _ in 0..ring.rank+1 {
                let count = order.len() as i32 + 1;
                if count >= 5 { break; }
                if rng_rings.get_value(count) == 0 {
                    if let Some(ss) = s.get_remove(rng_rings) { order.push(ss); }
                }
            }
            if rng_rings.get_value(4) == 0 { order.push(9); }
            order.iter().for_each(|s|{
                let stat =
                    match *s {
                        0|9 => 2 + 2*rng_rings.get_value(2+ring.rank),
                        10 => { 1 + if rng_rings.get_value(3) == 0 { ring.rank } else { 0 } },
                        _ => 1 + (ring.rank / 2) + rng_rings.get_value(2+ring.rank),
                    };
                ring.enhance[*s as usize] = stat as i8;
                // println!("{}: {}", CapabilityDefinition::get_name(*s), stat);
            });
        });
    }
}
