fn main() {
  let config = mftree::Config {
    seed: "puripuri2100".to_string(),
    original: mftree::PersonalInfo {
      id: mftree::PersonalID::str_to_id("original"),
      // 子供の数
      // この値はまだ使っていない
      number_of_children: 1,
      // パートナーの数
      // この値はまだ使っていない
      number_of_partners: 1,
    },
    // 平均寿命
    // この値はまだ使っていない
    average_life: 78,
    // 生まれた子供が5歳までに死亡する確率である乳幼児死亡率
    // この値はまだ使っていない
    infant_death_rate: mftree::Probability::make(5, 1000),
    // 合計特殊出生率
    // この値はまだ使っていない
    total_fertility_rate: mftree::Probability::make(105, 100),
    // パートナーができる確率
    // この値はまだ使っていない
    probability_of_having_partner: mftree::Probability::make(70, 100),
  };
  let max = mftree::Generation::u64_to_generation(5);
  let l = mftree::make_familly_list(&config, max);
  println!("{:?}", l);
}
