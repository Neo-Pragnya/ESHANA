//! Aṣṭakūṭa (eight-fold) compatibility from two sidereal Moon longitudes, with the tables of
//! `rulesets/matching.yaml`. Partner `a` takes the role classical tables give the groom, `b` the bride.

use crate::jyotish::{lord_of, natural_relation};
use crate::packs::{self, VashyaGroup};
use crate::vedic;
use serde_json::{json, Value};

fn vashya(sign: usize, deg: f64) -> String {
    match &packs::matching().vashya_groups[sign] {
        VashyaGroup::One(g) => g.clone(),
        VashyaGroup::Split(v) => v[usize::from(deg >= 15.0)].clone(),
    }
}

fn idx(order: &[String], name: &str) -> usize {
    order.iter().position(|x| x == name).unwrap_or(0)
}

/// Score the eight kūṭas. Returns each kūṭa with both partners' attributes, the score and the maximum.
#[must_use]
pub fn ashtakoota(moon_a: f64, moon_b: f64) -> Value {
    let m = packs::matching();
    let (sa, sb) = (vedic::nakshatra_of(moon_a), vedic::nakshatra_of(moon_b));
    let (ra, rb) = (crate::jyotish::sign_of(moon_a), crate::jyotish::sign_of(moon_b));
    let (da, db) = (moon_a.rem_euclid(30.0), moon_b.rem_euclid(30.0));
    let mut kootas = vec![];
    let mut push = |name: &str, score: f64, a: Value, b: Value, note: &str| {
        kootas.push(json!({"koota": name, "score": score, "max": m.max[name], "a": a, "b": b, "note": note}));
    };

    // 1 varna
    let (va, vb) = (m.varna_rank[ra], m.varna_rank[rb]);
    push(
        "varna",
        if va >= vb { 1.0 } else { 0.0 },
        json!(m.varna_names[&va]),
        json!(m.varna_names[&vb]),
        "1 when a's varna rank is at least b's",
    );
    // 2 vashya
    let (ga, gb) = (vashya(ra, da), vashya(rb, db));
    let t = &m.vashya_table;
    push(
        "vashya",
        t.scores[idx(&t.order, &ga)][idx(&t.order, &gb)],
        json!(ga),
        json!(gb),
        "group table",
    );
    // 3 tara
    let count = |from: usize, to: usize| (to + 27 - from) % 27 + 1;
    let good = |c: usize| !m.tara_bad_remainders.contains(&(c % 9));
    let (ca, cb) = (count(sb.index, sa.index), count(sa.index, sb.index));
    let tara = 1.5 * f64::from(u8::from(good(ca))) + 1.5 * f64::from(u8::from(good(cb)));
    push(
        "tara",
        tara,
        json!({"count_from_b": ca}),
        json!({"count_from_a": cb}),
        "1.5 per direction when the remainder of 9 is not 3, 5 or 7",
    );
    // 4 yoni
    let (ya, yb) = (&m.yoni[sa.index], &m.yoni[sb.index]);
    push(
        "yoni",
        m.yoni_matrix[idx(&m.yoni_order, ya)][idx(&m.yoni_order, yb)],
        json!(ya),
        json!(yb),
        "yoni matrix",
    );
    // 5 graha maitri
    let (la, lb) = (lord_of(ra), lord_of(rb));
    let maitri = if la == lb {
        m.graha_maitri["same_lord"]
    } else {
        let mut pair = [natural_relation(la, lb), natural_relation(lb, la)];
        pair.sort_unstable();
        let key = match pair {
            ["friend", "friend"] => "friend_friend",
            ["friend", "neutral"] => "friend_neutral",
            ["neutral", "neutral"] => "neutral_neutral",
            ["enemy", "friend"] => "friend_enemy",
            ["enemy", "neutral"] => "neutral_enemy",
            _ => "enemy_enemy",
        };
        m.graha_maitri[key]
    };
    push(
        "graha_maitri",
        maitri,
        json!(la),
        json!(lb),
        "relationship of the Moon-sign lords",
    );
    // 6 gana
    let (gna, gnb) = (&m.gana[sa.index], &m.gana[sb.index]);
    let gt = &m.gana_table;
    push(
        "gana",
        gt.scores[idx(&gt.order, gna)][idx(&gt.order, gnb)],
        json!(gna),
        json!(gnb),
        "gana table",
    );
    // 7 bhakoot
    let dist = |from: usize, to: usize| (to + 12 - from) % 12 + 1;
    let pair = [dist(ra, rb), dist(rb, ra)];
    let dosha = m.bhakoot_dosha_pairs.contains(&pair);
    push(
        "bhakoot",
        if dosha { 0.0 } else { 7.0 },
        json!(ra),
        json!(rb),
        "0 for 2/12, 5/9 and 6/8 sign distances",
    );
    // 8 nadi
    let (na, nb) = (&m.nadi[sa.index], &m.nadi[sb.index]);
    push(
        "nadi",
        if na == nb { 0.0 } else { 8.0 },
        json!(na),
        json!(nb),
        "0 when both share a nadi",
    );

    let total: f64 = kootas.iter().map(|k| k["score"].as_f64().unwrap_or(0.0)).sum();
    json!({
        "kootas": kootas,
        "total": total,
        "max": 36,
        "a_moon": {"nakshatra": sa.name, "pada": sa.pada, "sign": ra},
        "b_moon": {"nakshatra": sb.name, "pada": sb.pada, "sign": rb},
        "nadi_dosha": na == nb,
        "bhakoot_dosha": dosha,
        "rule_version": packs::rule_id(&m.id, &m.version),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_moons() {
        // Same nakshatra and sign: same nadi (0), bhakoot 1/1 (7), tara count 1 both ways (3), same yoni (4),
        // same lord (5), same gana (6), same vashya (2), varna equal (1) = 28.
        let r = ashtakoota(100.0, 100.0);
        assert_eq!(r["total"], 28.0);
        assert_eq!(r["nadi_dosha"], true);
    }

    #[test]
    fn scores_never_exceed_maxima() {
        for a in (0..360).step_by(7) {
            for b in (0..360).step_by(11) {
                let r = ashtakoota(f64::from(a) + 0.3, f64::from(b) + 0.7);
                for k in r["kootas"].as_array().unwrap() {
                    assert!(k["score"].as_f64().unwrap() <= k["max"].as_f64().unwrap() + 1e-9);
                }
                assert!(r["total"].as_f64().unwrap() <= 36.0);
            }
        }
    }
}
