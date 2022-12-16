//! 家系図を生成するためのライブラリ
use rand::{Rng, SeedableRng};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// 確率を表すデータ構造
/// f32やf64を使うとEqトレイトやHashトレイトが使えなくなったり、
/// マイナス値の扱いなどを考えないといけないので
/// これを作ることでうまく隠蔽する
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Probability {
  base: i64,
  value: i64,
}
impl Probability {
  pub fn make_f64(self) -> f64 {
    let v = self.value % self.base;
    (v as f64) / (self.base as f64)
  }
  pub fn make_f32(self) -> f32 {
    let v = self.value % self.base;
    (v as f32) / (self.base as f32)
  }
  pub fn make(value: i64, base: i64) -> Self {
    Probability { value, base }
  }
}

/// 世代を表す数値
/// 一旦i128を上限としておくが、多倍長整数を使うかもしれない
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Generation {
  i: u64,
}
impl Generation {
  pub fn u64_to_generation(i: u64) -> Self {
    Generation { i }
  }
  pub fn gen_one() -> Self {
    Generation { i: 1 }
  }
  pub fn increment(&self) -> Self {
    let i = self.i;
    Generation { i: i + 1 }
  }
}

/// 登場する人間にそれぞれつけられるユニークなID
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PersonalID {
  id: String,
}
impl PersonalID {
  pub fn str_to_id(id: &str) -> Self {
    PersonalID { id: id.to_string() }
  }
}

// /// 身体的性別を表す
// #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
// enum Sex {
//   Male,
//   Female,
//   Other,
// }

/// 年月日を表す
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Date {
  year: u64,
  month: u8,
  day: u8,
}
impl Date {
  pub fn gen(y: u64, m: u8, d: u8) -> Self {
    Date {
      year: y,
      month: m,
      day: d,
    }
  }
}

/// 生年月日・死亡年月日・性別など、様々な情報を格納する
/// ここにある情報をもとに家族関係を生成していく
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PersonalInfo {
  pub id: PersonalID,
  // もしかしたら性別は使わないかも
  // でも見てるときの面白さ的にはあった方が良いかな
  // /// 身体的性別を表す
  // /// 子供を作ることができるかどうかに関して判定をするために使用する
  // sex: Sex,
  // 後で実装！！！！！
  // /// 生年月日
  //date_of_birth: Date,
  // 後で実装！！！！！
  // /// 死亡年月日
  //date_of_death: Date,
  /// 子供の数
  /// 合計特殊出生率などにより決定する
  pub number_of_children: u8,
  /// パートナーの数
  /// 基本1でたまに0、2以上の扱いについては今後の課題とする
  pub number_of_partners: u8,
}

/// パートナーに関する情報
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PartnerInfo {
  pub id: PersonalID,
  // /// パートナーとなった年月日
  // data_of_start : Date,
  // /// パートナーではなくなった年月日
  // data_of_end : Option<Date>,
}

/// 家族関係を表す
/// これを一番最初の人ののPersonalIDから辿っていけば家系図が作れる
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FamilyRelationship {
  /// 親子関係を表す
  /// 親のidをそれぞれ保持するが、パートナーの方のIDはなくてもよい
  Child(Generation, PersonalID, Option<PersonalID>, PersonalInfo),
  /// パートナー関係を表す
  Partner(PersonalID, PartnerInfo),
  /// 大本となる人のデータ
  Original(PersonalInfo),
}

/// 家族関係生成時に参照される設定値
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Config {
  /// シード値
  pub seed: String,
  /// 大本となる一番最初の人間
  pub original: PersonalInfo,
  /// 平均寿命
  /// これと乳幼児死亡率をもとに死亡年月日を生成する
  pub average_life: i8,
  /// 生まれた子供が5歳までに死亡する確率である乳幼児死亡率
  /// これと平均寿命をもとに死亡年月日を生成する
  pub infant_death_rate: Probability,
  /// 合計特殊出生率
  pub total_fertility_rate: Probability,
  /// パートナーができる確率
  /// 様々な要因が考えられるが、ここではそれらを区別しない
  pub probability_of_having_partner: Probability,
}

/// 家系図情報を格納するリストを生成する
pub fn make_familly_list(config: &Config, generation_max: Generation) -> Vec<FamilyRelationship> {
  let seed = make_hash_value(config);
  let mut rand_rng = rand_pcg::Pcg64::seed_from_u64(seed);
  let origin = FamilyRelationship::Original(config.original.clone());
  let mut lst = vec![origin];
  let mut generation = Generation::gen_one();
  while generation < generation_max {
    // 最後の世代のリスト
    // これを基に、最初にカップルを作り、そのあとに新しい世代を生成する
    let g = generation.clone();
    let last_generation_lst = lst.iter().filter_map(|r| filter_map_generation(r, &g));
    generation = generation.increment();
    // ここでいろいろと生成する
    let mut new_generation_lst = last_generation_lst
      .map(|info| {
        // 生まれたときにパートナーの数と子供の数は決まっているので、それを基にいろいろと生成する
        // まずは乱数のシード値を新しく生成する
        let seed = make_hash_value(&info);
        let mut info_rand_rng = rand_pcg::Pcg64::seed_from_u64(seed);
        // パートナーを生成する
        let mut partner_lst = vec![];
        for _ in 0..info.number_of_partners {
          // 相手のinfoのhash値にランダムな値をくっつけてIDを作る
          let random_str = info_rand_rng.gen::<u32>().to_string();
          let seed_str = rand_rng.gen::<u8>().to_string();
          let id = PersonalID::str_to_id(
            &(make_hash_value(&info).to_string() + "-" + &random_str + "#" + &seed_str),
          );
          let partner_info = PartnerInfo { id };
          partner_lst.push((info.clone().id, partner_info))
        }
        // 子供を生成する
        let mut children_lst = vec![];
        for _ in 0..info.number_of_children {
          // 片方の親を特定する
          // 一定確率で片親がいないようにもする（これはconfigで指定させたい）
          let other_parent_info_opt = if random_bool(&mut rand_rng, 0.1) || partner_lst.is_empty() {
            None
          } else {
            Some(&partner_lst[info_rand_rng.gen::<usize>() % partner_lst.len()].1)
          };
          // 親のinfoのhash値にランダムな値をくっつける
          let other_parent_hash_value = other_parent_info_opt
            .map(|info| make_hash_value(&info).to_string())
            .unwrap_or_else(String::new);
          let random_str = info_rand_rng.gen::<u32>().to_string();
          let seed_str = rand_rng.gen::<u8>().to_string();
          let id = PersonalID::str_to_id(
            &(make_hash_value(&info).to_string()
              + "+"
              + &other_parent_hash_value
              + "+"
              + &random_str
              + "#"
              + &seed_str),
          );
          let number_of_children = 1;
          let number_of_partners = 1;
          let personal_info = PersonalInfo {
            id,
            number_of_children,
            number_of_partners,
          };
          let other_parent_id_opt = other_parent_info_opt.map(|info| info.clone().id);
          children_lst.push(FamilyRelationship::Child(
            generation.clone(),
            info.clone().id,
            other_parent_id_opt,
            personal_info,
          ))
        }
        let mut l = partner_lst
          .iter()
          .map(|(id, info)| FamilyRelationship::Partner(id.clone(), info.clone()))
          .collect::<Vec<_>>();
        l.append(&mut children_lst);
        l
      })
      .collect::<Vec<Vec<FamilyRelationship>>>()
      .concat();
    lst.append(&mut new_generation_lst)
  }
  lst
}

fn make_hash_value<T>(t: &T) -> u64
where
  T: Hash,
{
  let mut s = DefaultHasher::new();
  t.hash(&mut s);
  s.finish()
}

fn filter_map_generation(r: &FamilyRelationship, g: &Generation) -> Option<PersonalInfo> {
  match r {
    FamilyRelationship::Child(g2, _, _, info) if g == g2 => Some(info.clone()),
    FamilyRelationship::Original(info) if g == &Generation::gen_one() => Some(info.clone()),
    _ => None,
  }
}

fn random_bool(rand_rng: &mut rand_pcg::Lcg128Xsl64, f: f32) -> bool {
  rand_rng.gen::<f32>() < f
}
