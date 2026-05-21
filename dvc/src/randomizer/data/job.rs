use engage::{
    gamedata::{Gamedata, JobData, job::BattleStyles, skill::{SkillData, SkillDataCategorys}},
    random::Random,
    unit::{Gender, Unit}
};
use crate::{randomizer::{data::GameData, job::reclass::ClassTier}, DVCVariables};
use crate::config::DVCFlags;

pub struct Job {
    pub hash: i32,
    pub monster: bool,
    pub tier: ClassTier,
    pub gender: Gender,
    pub selectable: i32,
    pub n_selectable: i32,

    pub grow: [i8; 11],
    pub limit: [u8; 11],
    pub style: i32,
    pub attr: i32,
    pub cc: Vec<i32>,
}
impl Job {
    pub fn from_job(job_data: &JobData) -> Self {
        let mut grow: [i8; 11] = [0; 11];
        let mut limit = [0; 11];
        let job_grow = job_data.get_diff_grow();
        let job_limit = job_data.get_limit();
        let hash = job_data.parent.hash;
        let tier = ClassTier::from_job(job_data);
        let monster = job_data.weapons.iter().enumerate().any(|(k, v)| k > 0 && k < 9 && *v > 0);
        let style = job_data.style;
        let attr =  job_data.attrs;
        for x in 0..11 {
            grow[x] = job_grow[x];
            limit[x] = job_limit[x];
        }
        if job_data.parent.index == 0 || job_data.jid.to_string().starts_with("JID_紋章士_"){
            Self {
                monster, tier, hash, gender: Gender::None, n_selectable: 0, selectable: 0,
                cc: vec![], grow, limit, style, attr,
            }
        }
        else {
            let mut n_selectable = 0;
            let selectable = job_data.get_selectable_weapon_mask(&mut n_selectable).map(|w| w.value).unwrap_or(0);
            let flag = job_data.get_flag().value;
            let gender = if flag & 16 != 0 { Gender::Male } else if flag & 4 != 0 { Gender::Female } else { Gender::None };
            let mut cc = vec![];
                if tier == ClassTier::Base { cc = job_data.get_high_jobs().iter().map(|j| j.parent.hash).collect(); }
                else if tier == ClassTier::Promoted {
                    cc = JobData::get_list().unwrap()
                        .iter().filter(|s| s.get_high_jobs().iter().any(|v| v.parent.hash == hash))
                        .map(|v| v.parent.hash)
                        .collect();
                }
                Self {
                    monster, tier, gender, hash, n_selectable, selectable,
                    cc, grow, limit, style, attr,
                }
        }
    }
    pub fn get(&self) -> &'static mut JobData { JobData::try_get_hash_mut(self.hash).unwrap() }
    pub fn match_aptitude(&self, apt: i32) -> i32 {
        if self.n_selectable == 0 {
            if self.get().weapons.iter().enumerate().any(|v| (1 << v.0) & apt != 0 && *v.1 == 1) { 150 }
            else { 0 }
        }
        else {
            let mut count = 0;
            for x in 1..9 {
                if ((apt & (1 << x)) & self.selectable) != 0 { count += 1; }
            }
            if count < self.n_selectable { 0 }
            else { count * 100 }
        }
    }
    pub fn update_cap(&self, enable: bool) {
        let job = self.get();
        let base = job.get_base();
        let limit = job.get_limit();
        if enable {
            for x in 0..10 { limit[x] = 120 + base[x]; }
            limit[10] = 99;
        }
        else {
            limit.iter_mut().zip(self.limit.iter()).for_each(|(l, r)| { *l = *r; })
        }
    }
    pub fn update_style(&self, style: Option<i32>){
        let job = self.get();
        if let Some(style) = style { job.style = style; }
        else { job.style = self.style; }
    }
    pub fn update_attr(&self, random: Option<&'static Random>) {
        let job = self.get();
        if let Some(random) = random.as_ref() {
            let mut attr = 0;
            let mut rate = 50;
            while random.get_value(100) < rate {
                attr = 2 << random.get_value(5);
                rate = rate >> 1;
            }
            job.attrs = attr;
        }
        else { job.attrs = self.attr; }
    }
}
pub struct JobDataBase { pub jobs: Vec<Job>, }
impl JobDataBase {
    pub fn init() -> Self {
        Self { jobs: JobData::get_list().unwrap().iter().map(|j| Job::from_job(j)).collect() }
    }
    pub fn get_by_hash(&self, hash: i32) -> Option<&Job> { self.jobs.iter().find(|j| j.hash == hash) }
    pub fn get_reclass_job(&self, unit: &Unit, job: &JobData, class_tier: ClassTier) -> Option<&'static mut JobData> {
        let hash = job.parent.hash;
        let j = self.jobs.iter().find(|j| j.hash == hash)?;
        let apt = unit.get_person().aptitude.value; 
        let sub_apt = unit.get_person().sub_aptitude.value;
        j.cc.iter()
            .flat_map(|v| self.get_by_hash(*v).filter(|j| j.tier == class_tier))
            .map(|j| (j, j.match_aptitude(apt)))
            .max_by(|a, b| a.1.cmp(&b.1))
            .map(|a| a.0.get())
            .or_else(||
                j.cc.iter()
                 .flat_map(|v| self.get_by_hash(*v).filter(|j| j.tier == class_tier))
                 .map(|j| (j, j.match_aptitude(sub_apt)))
                 .max_by(|a, b| a.1.cmp(&b.1))
                 .map(|a| a.0.get())
            )
    }
    pub fn get() -> &'static Self { &GameData::get().job_db }
    pub fn update_styles(&self) {
        let style = DVCVariables::BattleStyles.get_value();
        let rng = crate::utils::get_rng();
        match style {
            1 => { self.jobs.iter().for_each(|job| { job.update_style(Some(rng.get_value(8) + 1)); }); },
            2 => { self.jobs.iter().for_each(|job| { job.update_style(Some(0)); }); },
            _ => { self.jobs.iter().for_each(|j| { j.update_style(None); }); }
        }
        JobData::get_list_mut().unwrap().iter().for_each(|job|{
            job.mask_skills.clear();
            if let Some(array) = job.skills {
                array.iter().flat_map(|sid| SkillData::get(*sid))
                    .for_each(|skill| { job.mask_skills.add_skill(skill, SkillDataCategorys::Job, 0); });
            }
            if let Some(array) = BattleStyles::get_skills2(job.style) {
                array.iter().flat_map(|sid| SkillData::get(*sid)).for_each(|skill| { job.mask_skills.add_skill(skill, SkillDataCategorys::Job, 0); });
            }
        })
    }
    pub fn update_attr(&self) {
        let rng = if DVCFlags::RandomClassAttrs.get_value() { Some(crate::utils::get_rng()) } else { None };
        self.jobs.iter().for_each(|j| j.update_attr(rng));
    }
    pub fn update_caps(&self) {
        let enable_cap = DVCFlags::MaxStatCaps.get_value();
        self.jobs.iter().for_each(|j|{j.update_cap(enable_cap); });
    }
}