// Every table of the game (makers, upgrades, shops, bosses, pets, story...) as exported from the web version.
use crate::big::{Big, b};
use serde::Deserialize;
use serde_json::Value;
use std::sync::OnceLock;

pub static RAW: &str = include_str!("../assets/data.json");
pub static MISSIONS: &str = include_str!("../assets/missions.json");

fn de_big<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Big, D::Error> {
    let v = Value::deserialize(d)?;
    Ok(match v { Value::Number(n) => b(n.as_f64().unwrap_or(0.0)), Value::String(s) => crate::big::bs(s.trim_start_matches("D:")), _ => Big::ZERO })
}
fn de_eff<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<(String, Value)>, D::Error> {
    let v = Value::deserialize(d)?;
    Ok(v.as_array().map(|a| a.iter().filter_map(|p| { let p = p.as_array()?; Some((p.first()?.as_str()?.to_string(), p.get(1).cloned().unwrap_or(Value::Null))) }).collect()).unwrap_or_default())
}

#[derive(Deserialize, Clone)] pub struct Gen { pub n: String, #[serde(deserialize_with = "de_big")] pub c: Big, pub r: f64, #[serde(deserialize_with = "de_big")] pub p: Big }
#[derive(Deserialize, Clone)] pub struct Cond { pub t: String, #[serde(default)] pub i: usize, pub n: f64 }
#[derive(Deserialize, Clone)] pub struct Upg { pub id: String, pub cat: String, pub col: f64, pub row: f64, pub n: String, pub d: String, #[serde(deserialize_with = "de_big")] pub cost: Big, #[serde(deserialize_with = "de_eff")] pub e: Vec<(String, Value)>, #[serde(default)] pub req: Vec<String>, #[serde(default)] pub cond: Option<Cond> }
#[derive(Deserialize, Clone)] pub struct Perk { pub id: String, pub n: String, pub d: String, pub max: u32 }
#[derive(Deserialize, Clone)] pub struct Shop { pub id: String, pub n: String, pub d: String, #[serde(deserialize_with = "de_big")] pub c: Big, #[serde(deserialize_with = "de_eff")] pub e: Vec<(String, Value)>, #[serde(default)] pub req: Vec<String>, #[serde(default)] pub col: f64, #[serde(default)] pub row: f64 }
#[derive(Deserialize, Clone)] pub struct Res { pub id: String, pub n: String, pub d: String, #[serde(deserialize_with = "de_eff")] pub e: Vec<(String, Value)>, pub tier: usize, #[serde(deserialize_with = "de_big")] pub c: Big }
#[derive(Deserialize, Clone)] pub struct Art { pub id: String, pub r: usize, pub n: String, pub d: String, #[serde(deserialize_with = "de_eff")] pub e: Vec<(String, Value)>, #[serde(default)] pub secret: bool }
#[derive(Deserialize, Clone)] pub struct Chal { pub id: String, pub n: String, pub d: String, pub rw: String }
#[derive(Deserialize, Clone)] pub struct Boss { pub n: String, #[serde(deserialize_with = "de_big")] pub hp: Big, pub i: usize }
#[derive(Deserialize, Clone)] pub struct Pet { pub id: String, pub n: String, pub kind: String, pub r: usize, pub col: String, pub d: String, pub b: (String, f64), pub bd: String }
#[derive(Deserialize, Clone)] pub struct Dim { pub id: String, pub n: String, pub res: String, pub cost: f64, pub col: String, pub mech: String, pub eff: String, #[serde(default)] pub cond: Option<(String, f64)>, #[serde(default)] pub tra: u32 }
#[derive(Deserialize, Clone)] pub struct Law { pub id: String, pub n: String, pub b: String, pub d: String }
#[derive(Deserialize, Clone)] pub struct DGen { pub c: f64, pub r: f64, pub p: f64 }
#[derive(Deserialize, Clone)] pub struct Badge { pub id: String, pub n: String, pub spr: String, pub what: String, pub th: Vec<f64> }
#[derive(Deserialize, Clone)] pub struct Chapter { pub n: String, pub who: String, pub syn: String, #[serde(default)] pub intro: Vec<(String, String)>, #[serde(default)] pub mid: Vec<(String, String)>, #[serde(default)] pub outro: Vec<(String, String)> }
#[derive(Deserialize, Clone)] pub struct Mission { pub t: String, pub k: String, pub a: Value, pub n: f64 }
#[derive(Clone)] pub struct Event { pub k: String, pub n: String, pub d: String, pub w: f64, pub col: String }
#[derive(Deserialize, Clone)] pub struct Rew { pub n: String, pub d: String, pub spr: String }
#[derive(Deserialize, Clone)] pub struct Ach { pub id: String, pub n: String, pub d: String, #[serde(default)] pub h: bool }
#[derive(Deserialize, Clone)] pub struct Change { pub v: String, pub d: String, pub n: Vec<String> }

pub struct Data {
    pub gens: Vec<Gen>, pub upg: Vec<Upg>, pub perks: Vec<Perk>, pub supg: Vec<Shop>, pub aupg: Vec<Shop>, pub vupg: Vec<Shop>, pub uupg: Vec<Shop>, pub inf: Vec<Shop>, pub perm: Vec<Shop>,
    pub res: Vec<Res>, pub arts: Vec<Art>, pub chal: Vec<Chal>, pub bosses: Vec<Boss>, pub events: Vec<Event>, pub pets: Vec<Pet>, pub dims: Vec<Dim>, pub laws: Vec<Law>,
    pub dgen: Vec<DGen>, pub dgen_n: Vec<String>, pub badges: Vec<Badge>, pub btier: Vec<(String, String)>, pub chapters: Vec<Chapter>, pub missions: Vec<Vec<Mission>>,
    pub chars: Vec<(String, String, String)>, pub rew: Vec<Rew>, pub ach: Vec<Ach>, pub rar: Vec<String>, pub rar_w: Vec<f64>, pub egg_time: Vec<f64>, pub egg_w: Vec<f64>,
    pub pet_stage: Vec<String>, pub chal_goal: Vec<Big>, pub suf: Vec<String>, pub ucats: Vec<(String, String)>, pub scale_name: Vec<(String, String)>, pub abil: Vec<(String, String, f64, f64, String)>,
    pub ldesc: Value, pub tos: String, pub tips: Vec<String>, pub changelog: Vec<Change>, pub story_m: f64, pub ult_ip: f64, pub gen_spr: Vec<String>, pub art_spr: Value, pub stats_names: Vec<String>,
}

fn arr<T: for<'a> Deserialize<'a>>(v: &Value, k: &str) -> Vec<T> { serde_json::from_value(v[k].clone()).unwrap_or_else(|e| panic!("table {k}: {e}")) }

pub fn data() -> &'static Data {
    static D: OnceLock<Data> = OnceLock::new();
    D.get_or_init(|| {
        let v: Value = serde_json::from_str(RAW).expect("data.json");
        let ev = v["EVENTS"].as_object().unwrap().iter().map(|(k, e)| Event { k: k.clone(), n: e["n"].as_str().unwrap().into(), d: e["d"].as_str().unwrap().into(), w: e["w"].as_f64().unwrap(), col: e["col"].as_str().unwrap().into() }).collect();
        let chars = v["CHARS"].as_object().unwrap().iter().map(|(k, c)| (k.clone(), c["n"].as_str().unwrap_or("").into(), c["role"].as_str().unwrap_or("").into())).collect();
        let abil = v["ABIL"].as_object().unwrap().iter().map(|(k, a)| (k.clone(), a["n"].as_str().unwrap().into(), a["cd"].as_f64().unwrap(), a["dur"].as_f64().unwrap_or(0.0), a["d"].as_str().unwrap().into())).collect();
        let pairs = |k: &str| -> Vec<(String, String)> { v[k].as_object().map(|o| o.iter().map(|(a, b)| (a.clone(), b.as_str().unwrap_or("").to_string())).collect()).unwrap_or_default() };
        Data {
            gens: arr(&v, "GENS"), upg: arr(&v, "UPG"), perks: arr(&v, "PERKS"), supg: arr(&v, "SUPG"), aupg: arr(&v, "AUPG"), vupg: arr(&v, "VUPG"), uupg: arr(&v, "UUPG"), inf: arr(&v, "INF"), perm: arr(&v, "PERM"),
            res: arr(&v, "RES"), arts: arr(&v, "ARTS"), chal: arr(&v, "CHAL"), bosses: arr(&v, "BOSSES"), events: ev, pets: arr(&v, "PETS"), dims: arr(&v, "DIMS"), laws: arr(&v, "LAWS"),
            dgen: arr(&v, "DGEN"), dgen_n: arr(&v, "DGEN_N"), badges: arr(&v, "BADGES"), btier: arr(&v, "BTIER"), chapters: arr(&v, "CHAPTERS"), missions: serde_json::from_str(MISSIONS).unwrap(),
            chars, rew: arr(&v, "REW"), ach: arr(&v, "ACH"), rar: arr(&v, "RAR"), rar_w: arr(&v, "RAR_W"), egg_time: arr(&v, "EGG_TIME"), egg_w: arr(&v, "EGG_W"),
            pet_stage: arr(&v, "PET_STAGE"), chal_goal: v["CHAL_GOAL"].as_array().unwrap().iter().map(|x| crate::big::bs(x.as_str().unwrap_or("1"))).collect(), suf: arr(&v, "SUF"),
            ucats: arr(&v, "UCATS"), scale_name: pairs("SCALE_NAME"), abil,
            ldesc: v["LDESC"].clone(), tos: v["TOS_HTML"].as_str().unwrap_or("").into(), tips: arr(&v, "LD_TIPS"), changelog: arr(&v, "CHANGELOG"), story_m: v["STORY_M"].as_f64().unwrap(), ult_ip: v["ULT_IP"].as_f64().unwrap(),
            gen_spr: arr(&v, "GEN_SPR"), art_spr: v["ART_SPR"].clone(), stats_names: v["STATS"].as_array().unwrap().iter().map(|s| s[0].as_str().unwrap_or("").to_string()).collect(),
        }
    })
}
pub const NG: usize = 25;
pub fn dim_idx(id: &str) -> usize { data().dims.iter().position(|d| d.id == id).unwrap_or(0) }
