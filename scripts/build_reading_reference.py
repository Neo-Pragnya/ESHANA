#!/usr/bin/env python3
"""Write eshana/data/reading_reference.json: reading notes for the studio's yoga and special-point pages.

These are paraphrases of what classical texts and common practice say about each yoga, dosha and special point:
what it is, what it is traditionally associated with, how practitioners judge its strength, and what weakens or
cancels it. They are interpretation (epistemic: interpretation) with attribution, kept out of the engine; the studio
shows them beside the computed facts and labels them. Edit this script, not the JSON.
"""
from __future__ import annotations

import json
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "crates/eshana-python/python/eshana/data/reading_reference.json"

BPHS, PD, SAR, BJ, JP, JS = ("Brihat Parashara Hora Shastra", "Phaladeepika", "Saravali", "Brihat Jataka",
                              "Jataka Parijata", "Jaimini Upadesha Sutras")

MAHAPURUSHA_READ = [
    "The yoga graha's own strength decides how much of the result shows: check its ṣaḍbala ratio and viṃśopaka.",
    "Kendra placement in the rāśi chart forms the yoga; the same graha well placed in the navāṃśa (not debilitated, "
    "ideally own or exalted there) is read as confirming it.",
    "Combustion, a lost planetary war, or affliction by malefics (conjunction or aspect) reduces it.",
    "Results are expected mainly in the daśā and antardaśā of the yoga graha.",
]

YOGAS = {
    "ruchaka": dict(
        summary="One of the five Mahāpuruṣa yogas, formed by Mars strong in a kendra.",
        effects=["courage, drive and physical vigour", "leadership in demanding or competitive fields",
                 "a commanding, energetic presence"],
        read=MAHAPURUSHA_READ, sources=[BPHS, PD, SAR]),
    "bhadra": dict(
        summary="One of the five Mahāpuruṣa yogas, formed by Mercury strong in a kendra.",
        effects=["intelligence, eloquence and skill with words and numbers", "learning, trade and analysis",
                 "a youthful, adaptable nature"],
        read=MAHAPURUSHA_READ, sources=[BPHS, PD, SAR]),
    "hamsa": dict(
        summary="One of the five Mahāpuruṣa yogas, formed by Jupiter strong in a kendra.",
        effects=["wisdom, good judgement and ethical standing", "respect from teachers and the learned",
                 "inclination to counsel, teaching or law"],
        read=MAHAPURUSHA_READ, sources=[BPHS, PD, SAR]),
    "malavya": dict(
        summary="One of the five Mahāpuruṣa yogas, formed by Venus strong in a kendra.",
        effects=["refinement, comfort and appreciation of beauty", "harmony in relationships and an attractive manner",
                 "success through arts, design, luxury or hospitality"],
        read=MAHAPURUSHA_READ, sources=[BPHS, PD, SAR]),
    "shasha": dict(
        summary="One of the five Mahāpuruṣa yogas, formed by Saturn strong in a kendra.",
        effects=["endurance, discipline and organising ability", "authority over groups or institutions",
                 "steady rise through persistence"],
        read=MAHAPURUSHA_READ, sources=[BPHS, PD, SAR]),
    "gaja_kesari": dict(
        summary="Jupiter in a kendra from the Moon: the 'elephant and lion' combination.",
        effects=["reputation and lasting goodwill", "intelligence used well, protection in difficulty",
                 "support from wise or influential people"],
        read=["Both Jupiter and the Moon should be strong: a waxing, unafflicted Moon and a Jupiter not debilitated or "
              "combust.", "Many readers also require Jupiter to be well placed from the lagna, not only from the Moon.",
              "The daśās of Jupiter and the Moon are when it is said to show."],
        cancel=["Jupiter debilitated, combust or in an enemy sign", "the Moon weak (near new Moon) or hemmed by malefics"],
        sources=[PD, SAR, BJ]),
    "budha_aditya": dict(
        summary="The Sun and Mercury in the same sign. Very common, since Mercury is never far from the Sun.",
        effects=["sharp intellect and communication", "skill in administration, accounts or scholarship"],
        read=["Because it occurs often, readers weigh it only when Mercury is not deeply combust (more than about 14° "
              "from the Sun is often asked for) and the sign and house are favourable.",
              "Its house shows where the intelligence is applied."],
        cancel=["Mercury combust or debilitated", "the pair afflicted by Saturn, Rāhu or Ketu"], sources=[PD, SAR]),
    "chandra_mangala": dict(
        summary="The Moon and Mars in the same sign.",
        effects=["enterprise and earning through effort or trade", "a forceful, sometimes restless emotional nature"],
        read=["The dignity of both grahas and the house occupied decide whether the drive is constructive.",
              "Read with the 2nd and 11th houses for its money side."], sources=[PD, SAR]),
    "guru_chandala": dict(
        summary="Jupiter with Rāhu (or Ketu) in the same sign, a combination classed among afflictions.",
        effects=["unconventional or questioning attitude to tradition and teachers",
                 "judgement that can be clouded or swayed, needing care in guidance and ethics"],
        read=["A strong, dignified Jupiter is read as resisting the node; a weak one as overcome by it.",
              "The degree distance between Jupiter and the node matters: close conjunctions are weighed more."],
        sources=["Common practice"]),
    "sunapha": dict(
        summary="A lunar yoga: a graha other than the Sun and the nodes in the 2nd from the Moon.",
        effects=["self-earned resources and a good name", "the qualities of the graha in the 2nd colour the result"],
        read=["Benefics in the 2nd from the Moon are read more favourably than malefics.",
              "The strength of that graha and of the Moon both matter."], sources=[BJ, SAR, PD]),
    "anapha": dict(
        summary="A lunar yoga: a graha other than the Sun and the nodes in the 12th from the Moon.",
        effects=["dignity, health and comfort", "the graha in the 12th from the Moon shapes the style of it"],
        read=["As with Sunapha, benefics give the gentler result and the graha's own strength scales it."],
        sources=[BJ, SAR, PD]),
    "durudhara": dict(
        summary="A lunar yoga: grahas (not the Sun or the nodes) on both sides of the Moon, in the 2nd and 12th from it.",
        effects=["support and resources, comfort and generosity", "combines the results of Sunapha and Anapha"],
        read=["Read the grahas on each side: benefics give ease, malefics on both sides can act like a hemming."],
        sources=[BJ, SAR, PD]),
    "kemadruma": dict(
        summary="A lunar condition: no graha (other than the Sun and the nodes) in the 2nd or 12th from the Moon.",
        effects=["periods of feeling unsupported or isolated", "fluctuating resources if the Moon is also weak"],
        read=["Classical texts list many cancellations, so it is rarely read alone.",
              "Check whether the Moon is strong and whether grahas occupy kendras from the lagna or the Moon."],
        cancel=["a graha in a kendra from the Moon or the lagna", "the Moon aspected by Jupiter or conjunct a benefic",
                "a strong, full Moon"], sources=[BJ, PD, SAR]),
    "raja_kendra_trikona": dict(
        summary="A Rāja yoga: the lord of a kendra joins the lord of a trikoṇa, linking effort (kendra) with "
                "fortune (trikoṇa).",
        effects=["rise in status, authority or recognition", "success in the areas of the houses involved"],
        read=["Stronger when both lords are dignified, unafflicted and placed in good houses.",
              "Most visible in the daśā and antardaśā of the lords taking part.",
              "Conjunction is the strongest link; mutual aspect and exchange of signs are also used by many readers."],
        cancel=["either lord debilitated, combust or in the 6th, 8th or 12th", "affliction by malefics"],
        sources=[BPHS, PD]),
    "dhana": dict(
        summary="A wealth yoga: a lord of the 2nd or 11th (income) joins a lord of the 1st, 5th or 9th.",
        effects=["accumulation of wealth and resources", "gains through the areas of the houses involved"],
        read=["The 2nd and 11th houses, their lords and the SAV bindus in those signs support the reading.",
              "Results come in the daśās of the lords involved."], cancel=["the lords weak or afflicted"],
        sources=[BPHS, PD]),
    "vipareeta_raja": dict(
        summary="A 'reversed' Rāja yoga: the lord of a dusthāna (6, 8, 12) placed in another dusthāna.",
        effects=["success that comes out of difficulty, crisis or the setbacks of others",
                 "resilience and unexpected turnarounds"],
        read=["The lord should be in a dusthāna without strongly influencing kendras or trikoṇas.",
              "Readers check the lagna and its lord are strong enough to benefit."],
        sources=[BPHS, "Uttara Kalamrita"]),
    "parivartana": dict(
        summary="Two grahas each sit in a sign ruled by the other, so each is read as also occupying its own sign.",
        effects=["the two houses and the two grahas work as a linked pair, each supporting the other's matters",
                 "results of one house are carried into the other"],
        read=["Readers judge it by which houses are exchanged: the Mahā, Khala and Dainya kinds below.",
              "Both grahas then behave as if in their own signs for strength, so their dignity is read with that in mind.",
              "The daśās of either graha bring the exchange into play."],
        sources=[PD, BPHS]),
    "maha_parivartana": dict(
        summary="An exchange between the lords of two good houses (not the 3rd, 6th, 8th or 12th).",
        effects=["wealth, standing and comforts tied to the two houses involved", "support from the people those houses show"],
        read=["Counted among the strongest exchanges when the lagna lord or a kendra or trikoṇa lord is one of the pair."],
        sources=[PD]),
    "khala_parivartana": dict(
        summary="An exchange that involves the 3rd house (and none of the 6th, 8th or 12th).",
        effects=["mixed results: gains through effort and initiative, with swings of fortune"],
        read=["The 3rd house is one of effort and courage, so results come through the person's own exertion."],
        sources=[PD]),
    "dainya_parivartana": dict(
        summary="An exchange that involves a dusthāna (the 6th, 8th or 12th house).",
        effects=["struggle, obstacles or loss in the matters of the other house involved",
                 "results that come after difficulty"],
        read=["Classical texts call it inauspicious; later readers note it can also give depth and endurance when the "
              "grahas are otherwise strong, much as a Vipareeta Rāja yoga does."],
        sources=[PD]),
    "lakshmi": dict(
        summary="The 9th lord (fortune) strong in its own or exaltation sign, in a kendra or trikoṇa.",
        effects=["prosperity, good fortune and refinement", "virtue and generosity"],
        read=["Many readers also ask for a strong lagna lord; the 9th lord's own strength scales the result.",
              "The daśā of the 9th lord is the time it shows most."], sources=[BPHS, PD]),
    "neecha_graha": dict(
        summary="A graha in its sign of debilitation.",
        effects=["the significations of that graha may need more effort or arrive late",
                 "the houses it rules and occupies can feel strained"],
        read=["Look for neecha-bhaṅga (cancellation): the lord of the debilitation sign or of the exaltation sign in a "
              "kendra from the lagna or the Moon, or the graha exalted in the navāṃśa, among other conditions.",
              "A cancelled debilitation is often read as a strong rise after early difficulty."],
        sources=[BPHS, PD, SAR]),
    "manglik_lagna": dict(
        summary="Kuja (Manglik) doṣa from the lagna: Mars in the 1st, 2nd, 4th, 7th, 8th or 12th.",
        effects=["read in matching for intensity or friction in marriage and partnership"],
        read=["ESHANA does not apply cancellations. Common ones: Mars in its own or exaltation sign, Mars aspected by "
              "Jupiter, both partners Manglik, or particular sign placements by region.",
              "Many readers check it from the lagna, the Moon and Venus and weigh the strongest."],
        sources=["Muhurta and matching texts", "Common practice"]),
    "manglik_moon": dict(
        summary="Kuja (Manglik) doṣa counted from the Moon.",
        effects=["as for the lagna version, but read through the mind and emotional life"],
        read=["Usually weighed less than the version from the lagna; cancellations as listed there."],
        sources=["Common practice"]),
    "manglik_venus": dict(
        summary="Kuja (Manglik) doṣa counted from Venus, the significator of marriage.",
        effects=["read for the relationship and marital life specifically"],
        read=["Usually weighed less than the version from the lagna; cancellations as listed there."],
        sources=["Common practice"]),
    "kala_sarpa": dict(
        summary="All seven grahas on one side of the Rāhu–Ketu axis.",
        effects=["read as a life that runs in intense cycles, with delays followed by sudden progress"],
        read=["A modern popular condition, not found in the major classical texts; treat with caution.",
              "Readers distinguish full and partial forms and the direction (from Rāhu or from Ketu)."],
        sources=["Modern practice"]),
    "grahana": dict(
        summary="The Sun or the Moon in the same sign as Rāhu or Ketu (the eclipse axis).",
        effects=["the luminary's significations (self, father and authority for the Sun; mind and mother for "
                 "the Moon) are read as clouded or intensified"],
        read=["Closeness in degrees matters: a node within a few degrees of the luminary is weighed most.",
              "A birth near an actual eclipse is read more strongly."], sources=["Common practice"]),
    "shrapit": dict(
        summary="Saturn and Rāhu in the same sign.",
        effects=["read as karmic delays or burdens in the matters of that house"],
        read=["A modern popular condition; check the dignity of Saturn and the degree distance to Rāhu."],
        sources=["Modern practice"]),
}

POINTS = {
    "lagna": dict(
        name="Lagna (Ascendant)",
        what="The rising degree of the zodiac at the birth place and moment.",
        why="It is the first house and the anchor of every house count; it stands for the body, the self and how "
            "life is met.",
        read=["Read the sign, its lord and where that lord sits: the lagna lord's strength is the chart's backbone.",
              "Grahas in the lagna and aspects on it colour the personality and health.",
              "The nakshatra and navāṃśa of the lagna degree refine the reading."],
        sources=[BPHS, PD]),
    "bhava_lagna": dict(
        name="Bhāva lagna",
        what="A special lagna that moves one sign every two hours from sunrise, starting at the Sun.",
        why="Used to judge the self and well-being through the day's elapsed time, as a check on the lagna.",
        read=["Compare it with the lagna: agreement strengthens a reading.",
              "The house it falls in from the lagna, and grahas with it, are noted."], sources=[BPHS]),
    "hora_lagna": dict(
        name="Horā lagna",
        what="A special lagna that moves one sign every hour (horā) from sunrise.",
        why="Read for wealth and resources.",
        read=["Its lord, and grahas in or aspecting it, are judged for financial strength.",
              "Rāja yogas involving the horā lagna are given weight for prosperity."], sources=[BPHS]),
    "ghati_lagna": dict(
        name="Ghaṭī lagna",
        what="A special lagna that moves one sign every ghaṭī (24 minutes) from sunrise.",
        why="Read for power, position and authority.",
        read=["Connections between the lagna, horā lagna and ghaṭī lagna (or their lords) are read as status."],
        sources=[BPHS]),
    "indu_lagna": dict(
        name="Indu lagna",
        what="A wealth lagna from the kalās of the 9th lords from the lagna and from the Moon, counted from the Moon.",
        why="Read for the flow of wealth over life.",
        read=["Benefics in or aspecting it, and a strong lord, are favourable; malefics there are read as effort.",
              "Daśās of grahas in the Indu lagna are watched for gains."], sources=["Jataka Parijata"]),
    "arudha_lagna": dict(
        name="Ārūḍha lagna (AL)",
        what="The image of the lagna: as far from the lagna lord as the lord is from the lagna, with exceptions.",
        why="Shows how the person is seen by the world: status, image and perception.",
        read=["Grahas in the 11th from it support the image; in the 12th from it, expenses and losses of status.",
              "Compare with the lagna: the lagna is the self, the AL the projection."], sources=[JS]),
    "upapada": dict(
        name="Upapada (UL)",
        what="The ārūḍha of the 12th house.",
        why="Read for marriage and the spouse.",
        read=["The 2nd from the upapada is read for the continuity of marriage.",
              "Benefics with it are favourable; its lord's placement adds detail."], sources=[JS]),
    "yogi_point": dict(
        name="Yogi point",
        what="Sun + Moon + 93°20′; a sensitive degree derived from the yoga (Sun–Moon sum) of the birth day.",
        why="Marks the source of fortune; its nakshatra lord is the yogi.",
        read=["The yogi graha's strength and placement are read as how easily good fortune comes.",
              "Transits over the yogi point, and the yogi's daśā, are watched as favourable."],
        sources=["Modern Jyotiṣa practice"]),
    "yogi": dict(
        name="Yogi graha",
        what="The lord of the nakshatra in which the yogi point falls.",
        why="Read as the graha that brings luck and support.",
        read=["A strong, well-placed yogi is favourable; its daśā is watched."], sources=["Modern Jyotiṣa practice"]),
    "duplicate_yogi": dict(
        name="Duplicate yogi",
        what="The lord of the sign in which the yogi point falls.",
        why="Acts like a second yogi.",
        read=["Read together with the yogi; when one graha is both, it is emphasised."],
        sources=["Modern Jyotiṣa practice"]),
    "avayogi": dict(
        name="Avayogi",
        what="The nakshatra lord of the point at yogi point + 186°40′.",
        why="Read as the graha that obstructs or drains.",
        read=["Its placement and daśā are watched for obstacles; a weak avayogi does less harm.",
              "When the avayogi is also a functional benefic, readers soften the reading."],
        sources=["Modern Jyotiṣa practice"]),
    "dagdha_rashis": dict(
        name="Dagdha (burnt) rāśis",
        what="Two signs made 'burnt' by the tithi of birth.",
        why="Houses and grahas in dagdha signs are read as weakened; their lords as less able to give results.",
        read=["Check which houses fall in these signs and which grahas sit there.",
              "Also used in muhūrta to avoid starting work in those signs."], sources=["Muhurta texts"]),
    "navamsha_64th": dict(
        name="64th navāṃśa",
        what="The navāṃśa 63 steps ahead of the Moon's or the lagna's: the one in the same position within the 8th sign.",
        why="Read as a sensitive point for health crises.",
        read=["Its lord and transits over it are watched; it is one factor among many, never read alone."],
        sources=["Traditional practice"]),
    "drekkana_22nd": dict(
        name="22nd dreshkāṇa (khara)",
        what="The dreshkāṇa 21 steps ahead of the lagna's or the Moon's: the one in the same position within the 8th sign.",
        why="Read as a sensitive point for danger and health; its lord is the 'khara' graha.",
        read=["The khara lord's daśā and transits over the point are watched; read with longevity factors."],
        sources=["Traditional practice"]),
    "graha_yuddha": dict(
        name="Graha yuddha (planetary war)",
        what="Two of Mars, Mercury, Jupiter, Venus or Saturn within 1° of longitude.",
        why="The loser is read as weakened in its significations; the winner as strengthened.",
        read=["ESHANA names the winner by greater northern ecliptic latitude; other conventions use brightness.",
              "Ṣaḍbala includes a yuddha component."], sources=[SAR, BPHS]),
    "kartari": dict(
        name="Kartari (hemming)",
        what="A sign hemmed by malefics (pāpa kartari) or benefics (śubha kartari) in the 2nd and 12th from it.",
        why="Pāpa kartari restricts the house or graha inside; śubha kartari protects it.",
        read=["Most important for the lagna, the Moon and the 7th house.",
              "The strength of the hemming grahas decides how much it matters."], sources=[JS, "Traditional practice"]),
    "chara_karakas": dict(
        name="Chara kārakas",
        what="Jaimini's movable significators: grahas ranked by degrees within their signs.",
        why="The ātmakāraka (highest degree) is the soul's significator; the others stand for minister, siblings, "
            "mother, children, relatives and spouse.",
        read=["The ātmakāraka's navāṃśa sign (kārakāṃśa) is central to Jaimini reading.",
              "Seven- and eight-kāraka schemes exist; ESHANA shows both."], sources=[JS, BPHS]),
    "karakamsa": dict(
        name="Kārakāṃśa",
        what="The navāṃśa sign of the ātmakāraka.",
        why="Read for the soul's direction, inclinations and spiritual path.",
        read=["Grahas in the kārakāṃśa and the 12th from it are read for spiritual tendency.",
              "Counted both in the navāṃśa and, by many, in the rāśi chart."], sources=[JS]),
    "arudha_padas": dict(
        name="Ārūḍha padas",
        what="The images of the twelve houses, computed like the ārūḍha lagna.",
        why="Show how the matters of each house appear in the world, as against their substance.",
        read=["A2 for visible wealth, A4 for property, A7 for partners and A10 for career status are the most used."],
        sources=[JS]),
    "upagrahas": dict(
        name="Upagrahas",
        what="Shadow points. Five are measured from the Sun (Dhūma, Vyatīpāta, Pariveṣa, Indracāpa, Upaketu). Gulika "
             "and Māndi are the lagna at set portions of the day or night.",
        why="Classical texts read them as subtle malefic influences on the houses and grahas they touch; Gulika and "
            "Māndi are the most used.",
        read=["Note the bhāva each falls in and any graha within a few degrees of it.",
              "Gulika or Māndi in the lagna, or joined to its lord, is traditionally weighed for health and obstacles.",
              "They modify a reading; they are not read on their own."],
        sources=[BPHS, PD]),
    "functional_nature": dict(
        name="Functional nature",
        what="Whether a graha acts as a benefic or malefic for this lagna, from the houses it rules.",
        why="Results depend on lordship as much as on natural benefic or malefic character.",
        read=["Lords of trikoṇas are generally favourable; lords of 3, 6, 8 and 11 trouble unless they also rule a "
              "trikoṇa; kendra lords become neutral (kendrādhipati doṣa for natural benefics).",
              "ESHANA states the rule it applies on the page."], sources=[BPHS, "Laghu Parashari"]),
}

# ── numerology: what each number and each role is commonly said to describe (paraphrased, attributed) ──────────
# Chaldean compound numbers 10–52 (paraphrased in our own words from Cheiro, Book of Numbers, 1926)
COMPOUND = {
    "10": "Rise and fall by one's own effort; a name that is remembered, for good or ill.",
    "11": 'Hidden trials and possible betrayal by others; faith is tested.',
    "12": 'Sacrifice for others; a sense of being used by those one serves.',
    "13": 'Upheaval and change; the end of one state and the start of another.',
    "14": 'Movement, trade and dealings with people; gains with risks from chance and rashness.',
    "15": 'Personal magnetism and persuasion; gifts and favours from others.',
    "16": 'A sudden fall from a height; plans that collapse unless carefully made.',
    "17": 'Rising above trials; a name that lasts.',
    "18": 'Conflict, deception and quarrels, sometimes within the family.',
    "19": 'Success, esteem and happiness; one of the most favoured numbers.',
    "20": 'An awakening or a call to a new purpose; plans that come late.',
    "21": 'Advancement after long effort; honour and elevation.',
    "22": 'Illusion and misplaced trust; a warning to judge people carefully.',
    "23": 'Help from superiors and protection; success in plans.',
    "24": 'Help from those in power and through partnerships, including marriage.',
    "25": 'Strength through experience; success after earlier trials.',
    "26": 'Losses through partnerships, advice or speculation.',
    "27": 'Authority and command; rewards from the mind and creative work.',
    "28": 'Gains that may be lost again through others; contradictions.',
    "29": 'Uncertainty, deception by others and trials in relationships.',
    "30": 'Thought and mental deduction; detachment from material ambition.',
    "31": 'Self-containment and isolation; a lonely number.',
    "32": "Like 5 and 23: influence over people, good fortune in plans if one keeps one's own counsel.",
    "33": 'No separate meaning of its own; it shares the influence of 24.',
    "34": 'No separate meaning of its own; it shares the influence of 25.',
    "35": 'No separate meaning of its own; it shares the influence of 26.',
    "36": 'No separate meaning of its own; it shares the influence of 27.',
    "37": 'Good friendships and happy partnerships, in love and in business.',
    "38": 'No separate meaning of its own; it shares the influence of 29.',
    "39": 'No separate meaning of its own; it shares the influence of 30.',
    "40": 'No separate meaning of its own; it shares the influence of 31.',
    "41": 'No separate meaning of its own; it shares the influence of 32.',
    "42": 'No separate meaning of its own; it shares the influence of 24.',
    "43": 'Revolution and upheaval; failure of plans.',
    "44": 'No separate meaning of its own; it shares the influence of 26.',
    "45": 'No separate meaning of its own; it shares the influence of 27.',
    "46": 'No separate meaning of its own; it shares the influence of 37.',
    "47": 'No separate meaning of its own; it shares the influence of 29.',
    "48": 'No separate meaning of its own; it shares the influence of 30.',
    "49": 'No separate meaning of its own; it shares the influence of 31.',
    "50": 'No separate meaning of its own; it shares the influence of 32.',
    "51": "Sudden advancement and strength; the warrior's number, with enemies made on the way.",
    "52": 'No separate meaning of its own; it shares the influence of 43.',
}

KUA = {
    "about": "The Kua (Gua) number of Eight Mansions feng shui, from the birth year (Chinese solar year from 4 February) and sex. It sorts people into an east group (1, 3, 4, 9) and a west group (2, 6, 7, 8), each with four favourable directions; the number is used to choose sitting, sleeping and facing directions.",
    "groups": {"east": "Favourable directions are east, south-east, north and south.", "west": "Favourable directions are west, north-west, south-west and north-east."},
}

NUM_SOURCES = ["Common Pythagorean practice (Balliett, Decoz)", "Chaldean practice (Cheiro)", "Indian numerology practice"]
NUMBERS = {
    "1": dict(key="beginnings, will, independence", strengths=["initiative", "leadership", "self-reliance"], challenges=["impatience", "stubbornness", "isolation"], planet="Sun"),
    "2": dict(key="partnership, sensitivity, balance", strengths=["diplomacy", "cooperation", "intuition"], challenges=["over-sensitivity", "indecision", "dependence"], planet="Moon"),
    "3": dict(key="expression, creativity, joy", strengths=["communication", "optimism", "imagination"], challenges=["scattered energy", "superficiality", "moodiness"], planet="Jupiter"),
    "4": dict(key="structure, work, foundations", strengths=["discipline", "reliability", "method"], challenges=["rigidity", "overwork", "resistance to change"], planet="Rahu (Uranus in Western practice)"),
    "5": dict(key="freedom, change, experience", strengths=["adaptability", "curiosity", "versatility"], challenges=["restlessness", "excess", "inconsistency"], planet="Mercury"),
    "6": dict(key="responsibility, care, harmony", strengths=["nurturing", "service", "aesthetic sense"], challenges=["over-responsibility", "worry", "control"], planet="Venus"),
    "7": dict(key="inquiry, inner life, wisdom", strengths=["analysis", "depth", "spiritual insight"], challenges=["aloofness", "secrecy", "doubt"], planet="Ketu (Neptune in Western practice)"),
    "8": dict(key="power, material mastery, karma", strengths=["ambition", "management", "endurance"], challenges=["materialism", "harshness", "delays"], planet="Saturn"),
    "9": dict(key="completion, compassion, humanity", strengths=["generosity", "idealism", "courage"], challenges=["impulsiveness", "resentment", "letting go"], planet="Mars"),
    "11": dict(key="master number: inspiration, illumination", strengths=["vision", "intuition", "inspiring others"], challenges=["nervous tension", "self-doubt"], planet="(reduces to 2)"),
    "22": dict(key="master number: the master builder", strengths=["large-scale practical vision", "organisation"], challenges=["pressure of potential", "overwhelm"], planet="(reduces to 4)"),
    "33": dict(key="master number: the master teacher", strengths=["compassionate service", "healing"], challenges=["self-sacrifice", "burden of care"], planet="(reduces to 6)"),
}
ROLES = {
    "growth": "The growth (or key) number: the first name alone. Read as the way a person tends to grow and the quality they keep returning to.",
    "intensity": "Intensity numbers: digits that occur in the full name more often than average. Read as traits that are pronounced, sometimes overdone.",
    "transit_physical": "The physical transit: the letter of the first name ruling this year. Read as the tone of practical and bodily matters for the year.",
    "transit_mental": "The mental transit: the letter of the middle name(s) ruling this year. Read as the tone of thinking and plans for the year.",
    "transit_spiritual": "The spiritual transit: the letter of the last name ruling this year. Read as the tone of inner life and values for the year.",
    "essence": "The essence number: the three transit letters added and reduced. Read as the theme of the year as a whole, alongside the personal year.",
    "life_path": "The main path and lessons of the life, from the full birth date: the most weighted number.",
    "expression": "Natural abilities and how they are expressed, from all the letters of the full name (also called destiny).",
    "soul_urge": "Inner motivation and what the heart wants, from the vowels (heart's desire).",
    "personality": "How others first see you, from the consonants.",
    "birthday": "A particular talent, from the day of birth.",
    "maturity": "What grows in importance in later life: life path plus expression.",
    "attitude": "First reactions and outlook: birth month plus day.",
    "balance": "How you steady yourself under stress, from the initials.",
    "hidden_passion": "The talent or drive most present in the name (the most frequent value).",
    "karmic_lessons": "Values absent from the name: qualities to develop in this life.",
    "subconscious_self": "Confidence in a crisis: nine minus the number of karmic lessons.",
    "rational_thought": "How you think things through: first name plus birth day.",
    "cornerstone": "How you approach things: the first letter of the first name.",
    "capstone": "How you finish things: the last letter of the first name.",
    "first_vowel": "The soul's instinctive response: the first vowel of the first name.",
    "pinnacles": "Four long periods of opportunity across life, each with its own number.",
    "challenges": "The obstacles to work through in each of the same four periods.",
    "life_cycles": "Three broad cycles of life: from the month, the day and the year of birth.",
    "personal_year": "The theme of the current year in a nine-year cycle.",
    "personal_month": "The theme of the month within the personal year.",
    "personal_day": "The tone of the day within the personal month.",
    "universal_year": "The collective theme of the calendar year.",
    "bridge_life_path_expression": "The distance between path and ability; a smaller number means they align more easily.",
    "bridge_soul_personality": "The distance between inner wish and outer image.",
    "planes_of_expression": "How the name's letters spread across physical, mental, emotional and intuitive ways of meeting life.",
    "inclusion": "How often each value appears in the name: strengths (many) and lessons (none).",
    "chaldean_name": "The Chaldean name number: the compound shows the hidden influence, the root the outward one.",
    "psychic": "The Chaldean psychic number, from the day of birth: how you see yourself.",
    "destiny": "The Chaldean destiny number, from the whole date: the course of events.",
    "mulank": "Mulank (driver or psychic number), from the day of birth: personality and daily nature, ruled by a planet.",
    "bhagyank": "Bhagyank (conductor or destiny number), from the whole date: fortune and the direction of life.",
    "name_number": "The name number (Chaldean values) in Indian practice, read for its friendship with mulank and bhagyank.",
    "lo_shu": "The Lo Shu grid of the birth date: numbers present show strengths, missing ones gaps, complete lines arrows.",
}
KARMIC_DEBT = {
    "13": "Debt of laziness from a past life, paid with steady work and focus.",
    "14": "Debt of freedom misused, paid with moderation and adaptability.",
    "16": "Debt of ego and broken bonds, paid through humility and rebuilding.",
    "19": "Debt of power misused, paid by learning to accept help and share.",
}
LESSONS = {
    "1": "initiative and confidence", "2": "patience and cooperation", "3": "self-expression", "4": "order and discipline",
    "5": "adaptability and courage to change", "6": "responsibility and care", "7": "inner study and trust",
    "8": "handling money and authority", "9": "compassion and letting go",
}
ARROWS = {
    "mental plane": "memory and intellect", "emotional plane": "feeling and sensitivity", "practical plane": "material and physical ability",
    "thought plane": "planning and ideas", "will plane": "determination and persistence", "action plane": "carrying things through",
    "golden plane (success)": "success through steady effort", "silver plane (property)": "property and stability",
}

# ── sahams (Tājika sensitive points): what each stands for and how it is used in a varṣaphala ─────────────────
# Written here in our own words from the saham chapters of the Tājika manuals; keyed by the engine's saham id
# (rulesets/auxiliary.yaml). `signifies` is the matter, `used_for` what a varṣaphala reader looks at it for.
TN, HR, RAMAN = ("Tājika Nīlakaṇṭhī (Nīlakaṇṭha, Saṃjñā Tantra, saham chapter)", "Hāyana Ratna (Balabhadra)",
                 "B. V. Raman, Varshaphal or the Hindu Progressed Horoscope")
SAHAM_SOURCES = [TN, HR, RAMAN]
SAHAM_GENERAL = dict(
    what="A saham (from the Arabic sahm, 'lot' or 'share') is a sensitive degree found by measuring the distance "
         "between two points and laying it off from a third, usually the lagna. Each saham stands for one matter of "
         "life. The Tājika school, which shaped the Indian annual chart, lists about fifty; ESHANA computes 37.",
    read=["Judge a saham mainly through its lord: a lord that is strong in pañcavargīya bala, well placed from the "
          "varṣa lagna (kendra or trikoṇa) and joined to or aspecting the saham is read as the matter prospering "
          "that year.",
          "A lord in the 6th, 8th or 12th, combust, debilitated or hit by malefics, or a saham occupied by malefics, "
          "is read as the matter meeting obstacles.",
          "The natal saham shows the promise; the same saham recomputed in the annual chart shows whether this year "
          "delivers it. A saham falling on, or aspected by, the year lord or the muntha is given weight.",
          "Night births reverse many formulas (A and B swap); the table marks those."],
    caution="Sahams refine a reading of the year; manuals warn against reading one in isolation from the varṣa "
            "lagna, the muntha and the year lord.",
)
SAHAMS = {
    "punya": ("Merit and good fortune: the store of past good deeds and the luck that follows from it.",
              "The general fortune of the year; the first saham checked, and an input to several others."),
    "vidya": ("Learning and knowledge, education and intellectual attainment.",
              "Studies, examinations, scholarship and teaching in the year."),
    "yasa": ("Fame and reputation: being known and spoken well of.",
             "Recognition, public standing and honours in the year."),
    "mitra": ("Friends, allies and well-wishers.",
              "Help from friends, new friendships, or quarrels with them."),
    "mahatmya": ("Greatness and magnanimity; dignity of character and stature.",
                 "Rise in standing, acts of generosity, being looked up to."),
    "asha": ("Hopes, wishes and expectations.",
             "Whether the wishes held at the start of the year are fulfilled."),
    "samartha": ("Ability and capacity: the strength to carry out what one sets out to do.",
                 "Competence and enterprise, the success of efforts and initiatives."),
    "bhratru": ("Brothers and sisters.",
                "The welfare of siblings and relations with them during the year."),
    "gaurava": ("Respect and esteem; the weight a person carries with others.",
                "Honour from elders and authorities, being treated with regard."),
    "pitru": ("The father and paternal matters.",
              "The father's health and fortunes, and support from him."),
    "raja": ("Kingship: authority, office and status; dealings with government.",
             "Promotion, position, favour or disfavour from those in power."),
    "matru": ("The mother and maternal matters.",
              "The mother's health and fortunes, and comfort from her."),
    "putra": ("Children and progeny.",
              "Birth of children, their welfare and the joy or worry they bring."),
    "jeeva": ("Life and vitality: the continuance of life itself.",
              "General health and longevity in the year, read with the 8th house and the Mṛtyu saham."),
    "karma": ("Work, action and profession; the duties one performs.",
              "Career events, new undertakings and success in occupation."),
    "roga": ("Illness and bodily disorder.",
             "Periods and kinds of sickness; afflicted, it warns of health trouble that year."),
    "kali": ("Strife and conflict; quarrels and the troubles of a difficult age.",
             "Disputes, litigation and discord in the year."),
    "sastra": ("Sacred learning and the śāstras; in some manuals also weapons.",
               "Study of scripture and traditional knowledge; in the weapon sense, danger from sharp instruments."),
    "bandhu": ("Relatives and kin beyond the immediate family.",
               "Help or trouble from relatives, family gatherings and ties."),
    "mrityu": ("Death and deadly danger.",
               "Read with great caution, together with the 8th house and the Jīva saham, for risk to life."),
    "paradesa": ("Foreign lands and living away from home.",
                 "Long journeys, travel abroad and residence in another country."),
    "artha": ("Wealth and money.",
              "Income, savings and financial gain or loss in the year."),
    "paradara": ("Another's spouse: illicit or extra-marital relations.",
                 "Temptation, scandal or secret liaisons; read for moral risk rather than as a forecast."),
    "vanik": ("Trade and commerce, merchants and dealing.",
              "Buying and selling, profit from trade."),
    "karyasiddhi": ("Accomplishment of undertakings: work brought to completion.",
                    "Whether the projects begun in the year succeed."),
    "vivaha": ("Marriage and the married partnership.",
               "Marriage or engagement in the year, and the state of the marriage."),
    "santapa": ("Sorrow, grief and mental distress.",
                "Periods of anxiety, grief or heartache."),
    "sraddha": ("Faith and devotion; sincere commitment.",
                "Religious practice, devotion and trust placed in others."),
    "preeti": ("Love and affection.",
               "Affection given and received, romance and fondness."),
    "jadya": ("Dullness and chronic illness: sluggishness of body or mind.",
              "Lingering ailments, lethargy and loss of mental sharpness."),
    "vyapara": ("Business and occupation, commercial activity.",
                "Business ventures and the dealings of the trade."),
    "satru": ("Enemies and opponents.",
              "Open opposition, rivals and their success or defeat."),
    "jalapathana": ("Crossing water: sea voyages, and danger by water.",
                    "Journeys over water and the risk of accidents involving water."),
    "bandhana": ("Bondage and confinement.",
                 "Restraint, imprisonment or being held back by circumstances."),
    "apamrityu": ("Untimely or accidental death; sudden danger.",
                  "Accidents and sudden threats; read with caution alongside the Mṛtyu saham."),
    "prasava": ("Childbirth and pregnancy.",
                "Pregnancy and delivery in the year, for oneself or one's spouse."),
    "labha": ("Gains and income of every kind.",
              "Profits, gains and the fulfilment of desires through receipt."),
}

# ── pañca pakṣī: the five birds and the five activities (Tamil tradition, paraphrased) ─────────────────────────
PAKSHI_SOURCES = ["Tamil pañca pakṣī śāstra, traditionally attributed to the Siddhas",
                  "Tamil almanac (pañcāṅgam) practice"]
PAKSHI = dict(
    what="A Tamil timing system. Day and night are each cut into five yāmas; in each yāma every one of five birds "
         "(Vulture, Owl, Crow, Cock, Peacock) is doing one of five things. A person is ruled by one bird, fixed at "
         "birth, and the activity of that bird at a moment says how favourable the moment is for that person.",
    birth_bird="The birth bird is read from the Moon's nakṣatra at birth, which falls in one of five groups, and "
               "from the pakṣa: the order of the birds is reversed for a birth in the dark half. It stays the "
               "person's bird for life.",
    relations="Each bird has two friends and two enemies. Practitioners prefer times when a friendly bird is strong "
              "and avoid those when an enemy bird rules or eats; in contests the two parties' birds are compared.",
    ruling="The day's ruling bird holds the first yāma of the day as ruler; the day is reckoned favourable for "
           "people of that bird.",
    dying="On each weekday (and pakṣa) one bird is said to be dying all day long. Someone whose own bird is the "
          "dying bird is advised to keep that day for routine matters and not to begin anything important.",
    activities={
        "ruling": ("Arasu", "Ruling: the bird is at full power. The best time to begin, to ask, to compete; "
                   "results come readily."),
        "eating": ("Ūṇ", "Eating: the bird is nourished and strong. Good for gains, earning and for most work."),
        "walking": ("Naḍai", "Walking: the bird is on the move. Moderate results, gained with effort; suited to "
                    "travel and routine."),
        "sleeping": ("Tuyil", "Sleeping: the bird is inactive. Results are slow or do not come; better to wait."),
        "dying": ("Sāvu", "Dying: the bird is at its weakest. Losses and failure are feared; avoid beginnings."),
    },
    strength="The activities rank from strongest to weakest as Ruling, Eating, Walking, Sleeping, Dying. Each yāma "
             "is split into five sub-periods that take the activities in turn, so a strong sub-period inside a weak "
             "yāma (or the reverse) softens the judgement: the strongest moment is a ruling sub-period inside a "
             "ruling yāma, the weakest a dying one inside a dying yāma.",
)

META = {
    "epistemic": "interpretation",
    "attribution": "Paraphrased summaries of classical texts and common practice, for orientation. ESHANA computes "
                   "whether conditions hold; what they mean belongs to the tradition named, not to the engine.",
}


def main() -> None:
    OUT.write_text(json.dumps({"meta": META, "yogas": YOGAS, "points": POINTS, "numerology": {"numbers": NUMBERS, "roles": ROLES, "karmic_debt": KARMIC_DEBT, "lessons": LESSONS, "arrows": ARROWS, "compound": COMPOUND, "kua": KUA, "sources": NUM_SOURCES + ["Cheiro, Book of Numbers (1926), for compound numbers", "Eight Mansions feng shui practice, for the Kua number"]},
                                "sahams": {"general": SAHAM_GENERAL, "sources": SAHAM_SOURCES,
                                           "items": {k: {"signifies": s, "used_for": u} for k, (s, u) in SAHAMS.items()}},
                                "pakshi": dict(PAKSHI, sources=PAKSHI_SOURCES)}, ensure_ascii=False, indent=1) + "\n",
                   encoding="utf-8")
    print(f"wrote {OUT.relative_to(OUT.parents[5])}: {len(YOGAS)} yogas and doshas, {len(POINTS)} points, "
          f"{len(SAHAMS)} sahams, pañca pakṣī")


if __name__ == "__main__":
    main()
